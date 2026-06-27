use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;

pub mod github;
pub mod http;
pub mod jira;
pub mod notion;
pub mod postgres;
pub mod slack;

/// Encodage Base64 standard (RFC 4648) sans padding conditionnel.
/// Partagé entre jira.rs (Basic Auth) et http.rs (auth Basic générique).
pub(super) fn base64_encode(input: &str) -> String {
    const TABLE: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let bytes = input.as_bytes();
    let mut out = Vec::with_capacity((bytes.len() + 2) / 3 * 4);
    let mut i = 0;
    while i + 2 < bytes.len() {
        let n = ((bytes[i] as u32) << 16) | ((bytes[i + 1] as u32) << 8) | (bytes[i + 2] as u32);
        out.push(TABLE[((n >> 18) & 63) as usize]);
        out.push(TABLE[((n >> 12) & 63) as usize]);
        out.push(TABLE[((n >> 6) & 63) as usize]);
        out.push(TABLE[(n & 63) as usize]);
        i += 3;
    }
    match bytes.len() - i {
        1 => {
            let n = (bytes[i] as u32) << 16;
            out.extend_from_slice(&[TABLE[((n >> 18) & 63) as usize], TABLE[((n >> 12) & 63) as usize], b'=', b'=']);
        }
        2 => {
            let n = ((bytes[i] as u32) << 16) | ((bytes[i + 1] as u32) << 8);
            out.extend_from_slice(&[TABLE[((n >> 18) & 63) as usize], TABLE[((n >> 12) & 63) as usize], TABLE[((n >> 6) & 63) as usize], b'=']);
        }
        _ => {}
    }
    String::from_utf8(out).unwrap()
}

/// Résultat d'un appel MCP tool ou resource.
#[derive(Debug)]
pub struct McpResult {
    pub content: Value,
    pub is_error: bool,
}

/// Trait implémenté par chaque connecteur built-in.
/// Les connecteurs custom (community) appellent un process stdio externe.
#[async_trait]
pub trait McpConnector: Send + Sync {
    fn id(&self) -> &'static str;

    async fn call_tool(
        &self,
        tool_name: &str,
        arguments: HashMap<String, Value>,
        config: &HashMap<String, Value>,
    ) -> anyhow::Result<McpResult>;

    async fn read_resource(
        &self,
        uri: &str,
        config: &HashMap<String, Value>,
    ) -> anyhow::Result<McpResult>;

    fn list_tools(&self) -> Vec<&'static str>;
}

/// Registre Rust des connecteurs built-in.
pub struct ConnectorRegistry {
    connectors: HashMap<&'static str, Box<dyn McpConnector>>,
}

impl ConnectorRegistry {
    pub fn new() -> Self {
        let mut reg = Self { connectors: HashMap::new() };
        reg.register(Box::new(github::GitHubConnector));
        reg.register(Box::new(jira::JiraConnector));
        reg.register(Box::new(notion::NotionConnector));
        reg.register(Box::new(postgres::PostgresConnector));
        reg.register(Box::new(http::HttpConnector));
        reg.register(Box::new(slack::SlackConnector));
        reg
    }

    pub fn register(&mut self, connector: Box<dyn McpConnector>) {
        self.connectors.insert(connector.id(), connector);
    }

    pub fn get(&self, id: &str) -> Option<&dyn McpConnector> {
        self.connectors.get(id).map(|c| c.as_ref())
    }
}
