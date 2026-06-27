use std::collections::HashMap;
use serde_json::Value;
use crate::{config::Config, connectors::ConnectorRegistry, secret::SecretResolver};

pub struct SessionManager {
    config:   Config,
    registry: ConnectorRegistry,
    secrets:  SecretResolver,
}

impl SessionManager {
    pub fn new(config: Config) -> Self {
        Self {
            secrets:  SecretResolver::new(&config),
            registry: ConnectorRegistry::new(),
            config,
        }
    }

    pub async fn run(self) -> anyhow::Result<()> {
        let client = redis::Client::open(self.config.redis_url.as_str())?;
        let mut conn = client.get_async_connection().await?;

        // Crée le consumer group si inexistant. MKSTREAM crée le stream s'il n'existe pas.
        let _: redis::RedisResult<()> = redis::cmd("XGROUP")
            .arg("CREATE").arg("jobs:mcp").arg("mcp-gateway").arg("$").arg("MKSTREAM")
            .query_async(&mut conn).await;
        // Ignorer l'erreur BUSYGROUP (groupe déjà existant)

        tracing::info!(worker_id = %self.config.worker_id, "MCP Gateway démarrée — en attente de jobs:mcp");

        // Compteur de tentatives par message ID (réinitialisé à chaque redémarrage du process).
        let mut failure_counts: HashMap<String, u8> = HashMap::new();

        loop {
            let results: Vec<(String, Vec<(String, HashMap<String, String>)>)> =
                redis::cmd("XREADGROUP")
                    .arg("GROUP").arg("mcp-gateway").arg(&self.config.worker_id)
                    .arg("COUNT").arg(10)
                    .arg("BLOCK").arg(2000)
                    .arg("STREAMS").arg("jobs:mcp").arg(">")
                    .query_async(&mut conn)
                    .await
                    .unwrap_or_default();

            for (_, messages) in results {
                for (id, fields) in messages {
                    match self.handle_message(&fields).await {
                        Ok(_) => {
                            let _: () = redis::cmd("XACK")
                                .arg("jobs:mcp").arg("mcp-gateway").arg(&id)
                                .query_async(&mut conn).await?;
                            failure_counts.remove(&id);
                        }
                        Err(e) => {
                            let attempts = {
                                let count = failure_counts.entry(id.clone()).or_insert(0);
                                *count += 1;
                                *count
                            };

                            if attempts >= self.config.dead_letter_max_attempts {
                                tracing::error!(
                                    job_id = %id, attempts, error = %e,
                                    "Job MCP déplacé en dead_letter après {} échecs", attempts
                                );
                                let _: () = redis::cmd("XADD")
                                    .arg("jobs:dead_letter").arg("*")
                                    .arg("original_stream").arg("jobs:mcp")
                                    .arg("original_id").arg(&id)
                                    .arg("error").arg(e.to_string())
                                    .arg("attempts").arg(attempts.to_string())
                                    .query_async(&mut conn).await?;
                                let _: () = redis::cmd("XACK")
                                    .arg("jobs:mcp").arg("mcp-gateway").arg(&id)
                                    .query_async(&mut conn).await?;
                                failure_counts.remove(&id);
                            } else {
                                tracing::warn!(
                                    job_id = %id, attempts,
                                    max = self.config.dead_letter_max_attempts,
                                    "Erreur MCP job (tentative {}/{}): {e}",
                                    attempts, self.config.dead_letter_max_attempts
                                );
                                // Pas d'ACK — Redis relivrera le message
                            }
                        }
                    }
                }
            }
        }
    }

    async fn handle_message(
        &self,
        fields: &HashMap<String, String>,
    ) -> anyhow::Result<()> {
        let connector_id = fields.get("connector_id").ok_or_else(|| anyhow::anyhow!("connector_id manquant"))?;
        let tool_name    = fields.get("tool_name").ok_or_else(|| anyhow::anyhow!("tool_name manquant"))?;
        let arguments: HashMap<String, Value> = fields.get("arguments")
            .map(|s| serde_json::from_str(s))
            .transpose()?.unwrap_or_default();
        let binding_id = fields.get("binding_id").ok_or_else(|| anyhow::anyhow!("binding_id manquant"))?;

        let config = self.secrets.resolve_binding_config(binding_id).await?;

        let connector = self.registry.get(connector_id)
            .ok_or_else(|| anyhow::anyhow!("Connecteur inconnu : {connector_id}"))?;

        let result = connector.call_tool(tool_name, arguments, &config).await?;

        tracing::debug!(connector = %connector_id, tool = %tool_name, is_error = result.is_error, "MCP call terminé");

        // TODO KODA-D02: publier le résultat dans Redis (clé reply_to fournie par l'API)
        // pour que l'API Axum retourne la réponse au client SSE.
        let _ = result;
        Ok(())
    }
}
