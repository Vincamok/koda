use std::collections::HashMap;
use serde_json::Value;
use crate::{config::Config, connectors::ConnectorRegistry, secret::SecretResolver};

/// Gère les sessions MCP actives et route les appels vers les connecteurs.
pub struct SessionManager {
    config:    Config,
    registry:  ConnectorRegistry,
    secrets:   SecretResolver,
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
        // Écoute les appels depuis l'API via Redis Streams `jobs:mcp`
        let client = redis::Client::open(self.config.redis_url.as_str())?;
        let mut conn = client.get_async_connection().await?;
        tracing::info!("MCP Gateway démarrée — en attente de jobs:mcp");

        loop {
            let results: Vec<(String, Vec<(String, HashMap<String, String>)>)> =
                redis::cmd("XREADGROUP")
                    .arg("GROUP").arg("mcp-gateway").arg("worker-1")
                    .arg("COUNT").arg(10)
                    .arg("BLOCK").arg(2000)
                    .arg("STREAMS").arg("jobs:mcp").arg(">")
                    .query_async(&mut conn)
                    .await
                    .unwrap_or_default();

            for (_, messages) in results {
                for (id, fields) in messages {
                    if let Err(e) = self.handle_message(&fields, &mut conn).await {
                        tracing::error!("Erreur MCP job {id}: {e}");
                    } else {
                        let _: () = redis::cmd("XACK").arg("jobs:mcp").arg("mcp-gateway").arg(&id).query_async(&mut conn).await?;
                    }
                }
            }
        }
    }

    async fn handle_message(
        &self,
        fields: &HashMap<String, String>,
        _conn: &mut redis::aio::Connection,
    ) -> anyhow::Result<()> {
        let connector_id = fields.get("connector_id").ok_or_else(|| anyhow::anyhow!("connector_id manquant"))?;
        let tool_name    = fields.get("tool_name").ok_or_else(|| anyhow::anyhow!("tool_name manquant"))?;
        let arguments: HashMap<String, Value> = fields.get("arguments")
            .map(|s| serde_json::from_str(s))
            .transpose()?.unwrap_or_default();
        let binding_id = fields.get("binding_id").ok_or_else(|| anyhow::anyhow!("binding_id manquant"))?;

        // Résolution des credentials depuis SecretRef
        let config = self.secrets.resolve_binding_config(binding_id).await?;

        let connector = self.registry.get(connector_id)
            .ok_or_else(|| anyhow::anyhow!("Connecteur inconnu : {connector_id}"))?;

        let result = connector.call_tool(tool_name, arguments, &config).await?;

        tracing::debug!("MCP call {connector_id}/{tool_name}: is_error={}", result.is_error);
        // TODO: publier le résultat dans Redis pour que l'API le retourne au client SSE
        Ok(())
    }
}
