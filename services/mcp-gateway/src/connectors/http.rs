use std::collections::HashMap;
use async_trait::async_trait;
use serde_json::Value;
use super::{McpConnector, McpResult};

pub struct HttpConnector;

#[async_trait]
impl McpConnector for HttpConnector {
    fn id(&self) -> &'static str { "http" }

    fn list_tools(&self) -> Vec<&'static str> {
        vec!["http_get", "http_post", "http_patch", "http_delete"]
    }

    async fn call_tool(
        &self,
        tool_name: &str,
        arguments: HashMap<String, Value>,
        config: &HashMap<String, Value>,
    ) -> anyhow::Result<McpResult> {
        let base_url = config["base_url"].as_str().unwrap_or("").trim_end_matches('/');
        let path = arguments.get("path").and_then(|v| v.as_str()).unwrap_or("/");
        let url = format!("{base_url}{path}");

        let client = build_client(config)?;

        let response = match tool_name {
            "http_get" => {
                let mut req = client.get(&url);
                if let Some(params) = arguments.get("params").and_then(|v| v.as_object()) {
                    req = req.query(&params.iter().map(|(k, v)| (k, v.to_string())).collect::<Vec<_>>());
                }
                req.send().await?
            }
            "http_post" | "http_patch" => {
                let body = arguments.get("body").cloned().unwrap_or(Value::Null);
                let req = if tool_name == "http_post" { client.post(&url) } else { client.patch(&url) };
                req.json(&body).send().await?
            }
            "http_delete" => client.delete(&url).send().await?,
            _ => anyhow::bail!("Outil inconnu : {tool_name}"),
        };

        let status = response.status();
        let body: Value = response.json().await.unwrap_or(Value::Null);

        Ok(McpResult {
            content: serde_json::json!({ "status": status.as_u16(), "body": body }),
            is_error: !status.is_success(),
        })
    }

    async fn read_resource(&self, _uri: &str, _config: &HashMap<String, Value>) -> anyhow::Result<McpResult> {
        anyhow::bail!("read_resource non supporté pour le connecteur HTTP générique")
    }
}

fn build_client(config: &HashMap<String, Value>) -> anyhow::Result<reqwest::Client> {
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse()?);

    let auth_type = config.get("auth_type").and_then(|v| v.as_str()).unwrap_or("none");
    match auth_type {
        "bearer" => {
            if let Some(token) = config.get("auth_value").and_then(|v| v.as_str()) {
                headers.insert("Authorization", format!("Bearer {token}").parse()?);
            }
        }
        "apikey-header" => {
            let header_name = config.get("auth_header").and_then(|v| v.as_str()).unwrap_or("X-Api-Key");
            if let Some(key) = config.get("auth_value").and_then(|v| v.as_str()) {
                headers.insert(header_name, key.parse()?);
            }
        }
        "basic" => {
            if let Some(val) = config.get("auth_value").and_then(|v| v.as_str()) {
                let encoded = super::base64_encode(val);
                headers.insert("Authorization", format!("Basic {encoded}").parse()?);
            }
        }
        _ => {}
    }

    Ok(reqwest::Client::builder().default_headers(headers).build()?)
}
