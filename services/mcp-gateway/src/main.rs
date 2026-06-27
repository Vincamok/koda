//! MCP Gateway — proxy entre le LLM (via l'API Koda) et les connecteurs MCP actifs.
//!
//! Responsabilités :
//! - Maintient une session MCP par WorkspaceMCPBinding actif
//! - Route les tool_calls et resource_reads vers le bon connecteur
//! - Injecte les credentials (SecretRef) avant l'appel
//! - Retourne les résultats au format MCP au caller (API Axum)
//!
//! Transport : HTTP/SSE (spec MCP) ou stdio pour les connecteurs locaux.

use anyhow::Result;

mod config;
mod connectors;
mod proxy;
mod secret;
mod session;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let cfg = config::Config::from_env()?;
    session::SessionManager::new(cfg).run().await
}
