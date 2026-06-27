use std::collections::HashMap;
use serde_json::Value;
use crate::config::Config;

/// Résoud les SecretRef en valeurs claires pour injection dans la config du connecteur.
/// En production, appelle le secret store (Vault ou équivalent).
/// Les valeurs ne sont jamais loggées ni stockées après usage.
pub struct SecretResolver {
    #[allow(dead_code)]
    config: Config,
}

impl SecretResolver {
    pub fn new(config: &Config) -> Self {
        Self { config: config.clone() }
    }

    /// Charge la config d'un WorkspaceMCPBinding depuis la DB et résoud les SecretRef.
    pub async fn resolve_binding_config(
        &self,
        binding_id: &str,
    ) -> anyhow::Result<HashMap<String, Value>> {
        // TODO: implémenter la résolution réelle depuis DB + secret store
        // Pour l'instant, stub pour compilation
        tracing::debug!("Résolution config pour binding {binding_id}");
        Ok(HashMap::new())
    }
}
