use sqlx::PgPool;
use std::time::Duration;
use time::OffsetDateTime;
use uuid::Uuid;

const TICK_INTERVAL_SECS: u64 = 60;

#[derive(sqlx::FromRow)]
struct TriggerRow {
    id: Uuid,
    workspace_id: Uuid,
    pipeline_id: Uuid,
    schedule_cron: Option<String>,
    organization_id: Uuid,
}

pub struct CronScheduler {
    pub pool: PgPool,
}

impl CronScheduler {
    pub async fn run(self) -> anyhow::Result<()> {
        tracing::info!("cron scheduler started — checking every {}s", TICK_INTERVAL_SECS);
        loop {
            if let Err(e) = self.tick().await {
                tracing::error!(error = %e, "cron scheduler tick error");
            }
            tokio::time::sleep(Duration::from_secs(TICK_INTERVAL_SECS)).await;
        }
    }

    async fn tick(&self) -> anyhow::Result<()> {
        // Fetch all active schedule triggers
        let triggers = sqlx::query_as::<_, TriggerRow>(
            r#"SELECT at.id, at.workspace_id, at.pipeline_id, at.schedule_cron,
                      cp.organization_id
               FROM automation_triggers at
               JOIN cicd_pipelines cp ON cp.id = at.pipeline_id
               WHERE at.trigger_type = 'schedule' AND at.is_active = true
                 AND at.schedule_cron IS NOT NULL"#,
        )
        .fetch_all(&self.pool)
        .await?;

        let now = OffsetDateTime::now_utc();

        for trigger in triggers {
            let Some(cron_expr) = trigger.schedule_cron else { continue };

            if !should_run_now(&cron_expr, now) {
                continue;
            }

            // Enqueue pipeline job
            let payload = serde_json::json!({
                "type": "run_pipeline",
                "pipeline_id": trigger.pipeline_id,
                "workspace_id": trigger.workspace_id,
                "org_id": trigger.organization_id,
                "trigger": "schedule",
                "trigger_id": trigger.id,
            });

            if let Err(e) = sqlx::query!(
                "INSERT INTO jobs (job_type, payload, status) VALUES ('pipeline', $1, 'pending')",
                payload,
            )
            .execute(&self.pool)
            .await
            {
                tracing::error!(trigger_id = %trigger.id, error = %e, "failed to enqueue scheduled pipeline");
            } else {
                tracing::info!(
                    trigger_id = %trigger.id,
                    pipeline_id = %trigger.pipeline_id,
                    cron = %cron_expr,
                    "scheduled pipeline enqueued"
                );
            }
        }

        Ok(())
    }
}

/// Minimal cron check: matches if the current minute is consistent with the expression.
/// Supports 5-field standard cron: min hour dom month dow
fn should_run_now(expr: &str, now: OffsetDateTime) -> bool {
    let parts: Vec<&str> = expr.split_whitespace().collect();
    if parts.len() != 5 {
        return false;
    }

    let fields = [
        (parts[0], now.minute() as u32),
        (parts[1], now.hour() as u32),
        (parts[2], now.day() as u32),
        (parts[3], now.month() as u32),
        (parts[4], now.weekday().number_days_from_sunday() as u32),
    ];

    fields.iter().all(|(expr, val)| matches_cron_field(expr, *val))
}

fn matches_cron_field(expr: &str, val: u32) -> bool {
    if expr == "*" {
        return true;
    }

    // Handle */n step
    if let Some(step_str) = expr.strip_prefix("*/") {
        if let Ok(step) = step_str.parse::<u32>() {
            return step > 0 && val % step == 0;
        }
    }

    // Handle comma-separated list
    for part in expr.split(',') {
        // Handle range a-b
        if let Some((a, b)) = part.split_once('-') {
            if let (Ok(a), Ok(b)) = (a.parse::<u32>(), b.parse::<u32>()) {
                if val >= a && val <= b {
                    return true;
                }
            }
        } else if let Ok(n) = part.parse::<u32>() {
            if n == val {
                return true;
            }
        }
    }

    false
}
