use std::time::Duration;

use sqlx::PgPool;

const CHECK_INTERVAL_SECS: u64 = 300; // 5 minutes

pub struct AlertWatcher {
    pub pool: PgPool,
    pub http: reqwest::Client,
}

impl AlertWatcher {
    pub async fn run(self) -> anyhow::Result<()> {
        tracing::info!("alert watcher started — checking every {}s", CHECK_INTERVAL_SECS);
        loop {
            if let Err(e) = self.tick().await {
                tracing::error!(error = %e, "alert watcher tick error");
            }
            tokio::time::sleep(Duration::from_secs(CHECK_INTERVAL_SECS)).await;
        }
    }

    async fn tick(&self) -> anyhow::Result<()> {
        self.check_quota_limits().await?;
        self.check_pipeline_failures().await?;
        self.check_stuck_workspaces().await?;
        Ok(())
    }

    /// Alert when an org is at ≥ 80% of their workspace quota.
    async fn check_quota_limits(&self) -> anyhow::Result<()> {
        let rows = sqlx::query!(
            r#"
            SELECT q.organization_id,
                   q.max_workspaces,
                   COUNT(w.id)::BIGINT AS used_workspaces
            FROM organization_quotas q
            LEFT JOIN workspaces w ON w.organization_id = q.organization_id AND w.status != 'closed'
            GROUP BY q.organization_id, q.max_workspaces
            HAVING COUNT(w.id)::BIGINT >= (q.max_workspaces * 0.8)::BIGINT
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        for row in rows {
            let used = row.used_workspaces.unwrap_or(0);
            let pct = (used as f64 / row.max_workspaces as f64 * 100.0) as i32;

            // Check if we already alerted in the last 24h
            let recent: Option<i64> = sqlx::query_scalar!(
                r#"SELECT COUNT(*) FROM alert_events
                   WHERE organization_id = $1 AND message LIKE '%quota%'
                     AND created_at > NOW() - INTERVAL '24 hours'"#,
                row.organization_id,
            )
            .fetch_one(&self.pool)
            .await
            .ok()
            .flatten();

            if recent.unwrap_or(0) > 0 {
                continue;
            }

            self.emit_alert(
                row.organization_id,
                None,
                "quota_near_limit",
                "high",
                &format!(
                    "Organisation at {}% workspace quota ({}/{} workspaces used)",
                    pct, used, row.max_workspaces
                ),
                serde_json::json!({
                    "used_workspaces": used,
                    "max_workspaces": row.max_workspaces,
                    "percent": pct,
                }),
            )
            .await?;
        }

        Ok(())
    }

    /// Alert when pipelines fail repeatedly (≥ 3 failures in 1 hour for same workspace).
    async fn check_pipeline_failures(&self) -> anyhow::Result<()> {
        let rows = sqlx::query!(
            r#"
            SELECT cp.organization_id,
                   cp.workspace_id,
                   COUNT(j.id)::BIGINT AS failed_count
            FROM jobs j
            JOIN cicd_pipelines cp ON cp.id::TEXT = (j.payload->>'pipeline_id')
            WHERE j.status = 'failed'
              AND j.updated_at > NOW() - INTERVAL '1 hour'
            GROUP BY cp.organization_id, cp.workspace_id
            HAVING COUNT(j.id) >= 3
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        for row in rows {
            let recent: Option<i64> = sqlx::query_scalar!(
                r#"SELECT COUNT(*) FROM alert_events
                   WHERE workspace_id = $1 AND message LIKE '%pipeline%fail%'
                     AND created_at > NOW() - INTERVAL '2 hours'"#,
                row.workspace_id,
            )
            .fetch_one(&self.pool)
            .await
            .ok()
            .flatten();

            if recent.unwrap_or(0) > 0 {
                continue;
            }

            self.emit_alert(
                row.organization_id,
                Some(row.workspace_id),
                "pipeline_failed",
                "high",
                &format!(
                    "{} pipeline failures in the last hour",
                    row.failed_count.unwrap_or(0)
                ),
                serde_json::json!({ "failed_count": row.failed_count }),
            )
            .await?;
        }

        Ok(())
    }

    /// Alert when a workspace has been in 'starting' or 'cloning' for more than 15 minutes.
    async fn check_stuck_workspaces(&self) -> anyhow::Result<()> {
        let rows = sqlx::query!(
            r#"
            SELECT id, organization_id, status
            FROM workspaces
            WHERE status IN ('starting', 'cloning')
              AND updated_at < NOW() - INTERVAL '15 minutes'
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        for row in rows {
            let recent: Option<i64> = sqlx::query_scalar!(
                r#"SELECT COUNT(*) FROM alert_events
                   WHERE workspace_id = $1 AND message LIKE '%stuck%'
                     AND created_at > NOW() - INTERVAL '4 hours'"#,
                row.id,
            )
            .fetch_one(&self.pool)
            .await
            .ok()
            .flatten();

            if recent.unwrap_or(0) > 0 {
                continue;
            }

            self.emit_alert(
                row.organization_id,
                Some(row.id),
                "workspace_stuck",
                "medium",
                &format!("Workspace stuck in '{}' for more than 15 minutes", row.status),
                serde_json::json!({ "status": row.status }),
            )
            .await?;
        }

        Ok(())
    }

    async fn emit_alert(
        &self,
        org_id: uuid::Uuid,
        workspace_id: Option<uuid::Uuid>,
        rule_type: &str,
        severity: &str,
        message: &str,
        metadata: serde_json::Value,
    ) -> anyhow::Result<()> {
        tracing::warn!(
            org_id = %org_id,
            rule_type = rule_type,
            severity = severity,
            message = message,
            "alert triggered"
        );

        // Find matching alert rules
        let rules = sqlx::query!(
            "SELECT id, webhook_url FROM alert_rules
             WHERE organization_id = $1 AND rule_type = $2 AND enabled = true",
            org_id,
            rule_type,
        )
        .fetch_all(&self.pool)
        .await?;

        for rule in &rules {
            // Store alert event
            let event_id = sqlx::query_scalar!(
                r#"INSERT INTO alert_events (rule_id, organization_id, workspace_id, severity, message, metadata)
                   VALUES ($1, $2, $3, $4, $5, $6)
                   RETURNING id"#,
                rule.id,
                org_id,
                workspace_id,
                severity,
                message,
                metadata,
            )
            .fetch_one(&self.pool)
            .await?;

            // Send webhook notification if configured
            if let Some(ref url) = rule.webhook_url {
                let payload = serde_json::json!({
                    "alert_id": event_id,
                    "rule_type": rule_type,
                    "severity": severity,
                    "message": message,
                    "organization_id": org_id,
                    "workspace_id": workspace_id,
                    "metadata": metadata,
                });

                match self.http.post(url).json(&payload).send().await {
                    Ok(_) => {
                        let _ = sqlx::query!(
                            "UPDATE alert_events SET notified_at = NOW() WHERE id = $1",
                            event_id
                        )
                        .execute(&self.pool)
                        .await;
                    }
                    Err(e) => {
                        tracing::warn!(alert_id = %event_id, error = %e, "failed to send alert webhook");
                    }
                }
            }
        }

        // If no rules match, still log to alert_events with a default severity
        if rules.is_empty() {
            let _ = sqlx::query!(
                r#"INSERT INTO alert_events (rule_id, organization_id, workspace_id, severity, message, metadata)
                   SELECT id, $2, $3, $4, $5, $6 FROM alert_rules
                   WHERE organization_id = $2 AND rule_type = $1 AND enabled = true
                   LIMIT 1"#,
                rule_type,
                org_id,
                workspace_id,
                severity,
                message,
                metadata,
            )
            .execute(&self.pool)
            .await;
        }

        Ok(())
    }
}
