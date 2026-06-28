use std::time::Duration;

use sqlx::PgPool;

const SHIP_INTERVAL_SECS: u64 = 30;
const BATCH_SIZE: i64 = 500;

/// Ships logs from the `log_entries` table to a configured Loki instance.
/// Workers and the API write structured log rows; this shipper forwards them
/// to Loki using the JSON push API and marks them as shipped.
pub struct LokiShipper {
    pub pool: PgPool,
    pub http: reqwest::Client,
    pub loki_url: String,
}

impl LokiShipper {
    pub async fn run(self) -> anyhow::Result<()> {
        if self.loki_url.is_empty() {
            tracing::info!("Loki shipper disabled — LOKI_URL not set");
            return futures::future::pending::<anyhow::Result<()>>().await;
        }

        tracing::info!(loki_url = %self.loki_url, "Loki shipper started — shipping every {}s", SHIP_INTERVAL_SECS);
        loop {
            if let Err(e) = self.tick().await {
                tracing::error!(error = %e, "Loki shipper tick error");
            }
            tokio::time::sleep(Duration::from_secs(SHIP_INTERVAL_SECS)).await;
        }
    }

    async fn tick(&self) -> anyhow::Result<()> {
        // Read unshipped log rows
        let rows = sqlx::query!(
            r#"SELECT id, ts, level, service, message, fields
               FROM log_entries
               WHERE shipped_at IS NULL
               ORDER BY ts ASC
               LIMIT $1"#,
            BATCH_SIZE,
        )
        .fetch_all(&self.pool)
        .await?;

        if rows.is_empty() {
            return Ok(());
        }

        // Build Loki push payload — group by service label
        let mut streams: std::collections::HashMap<String, Vec<[String; 2]>> = std::collections::HashMap::new();

        for row in &rows {
            let labels = format!(
                r#"{{service="{}",level="{}"}}"#,
                row.service, row.level
            );
            let ts_ns = row.ts
                .unix_timestamp_nanos()
                .to_string();
            let line = format!(
                "{} {}",
                row.message,
                row.fields.as_ref()
                    .map(|f| f.to_string())
                    .unwrap_or_default()
            );
            streams.entry(labels).or_default().push([ts_ns, line]);
        }

        let stream_entries: Vec<serde_json::Value> = streams
            .into_iter()
            .map(|(labels, values)| {
                // Parse labels string back to object for JSON
                serde_json::json!({
                    "stream": { "raw": labels },
                    "values": values,
                })
            })
            .collect();

        let push_body = serde_json::json!({ "streams": stream_entries });

        let push_url = format!("{}/loki/api/v1/push", self.loki_url.trim_end_matches('/'));

        let resp = self.http
            .post(&push_url)
            .json(&push_body)
            .timeout(Duration::from_secs(15))
            .send()
            .await?;

        if resp.status().is_success() || resp.status().as_u16() == 204 {
            // Mark rows as shipped
            let ids: Vec<uuid::Uuid> = rows.iter().map(|r| r.id).collect();
            sqlx::query!(
                "UPDATE log_entries SET shipped_at = NOW() WHERE id = ANY($1)",
                &ids,
            )
            .execute(&self.pool)
            .await?;

            tracing::debug!(count = ids.len(), "Loki: shipped log entries");
        } else {
            let status = resp.status().as_u16();
            tracing::warn!(status, "Loki push returned non-success");
        }

        Ok(())
    }
}
