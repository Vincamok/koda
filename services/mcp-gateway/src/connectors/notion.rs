use std::collections::HashMap;
use async_trait::async_trait;
use serde_json::Value;
use super::{McpConnector, McpResult};

const BASE: &str = "https://api.notion.com/v1";

pub struct NotionConnector;

#[async_trait]
impl McpConnector for NotionConnector {
    fn id(&self) -> &'static str { "notion" }

    fn list_tools(&self) -> Vec<&'static str> {
        vec!["notion_search", "notion_get_page", "notion_create_page", "notion_append_block"]
    }

    async fn call_tool(
        &self,
        tool_name: &str,
        arguments: HashMap<String, Value>,
        config: &HashMap<String, Value>,
    ) -> anyhow::Result<McpResult> {
        let token  = config["token"].as_str().ok_or_else(|| anyhow::anyhow!("token manquant"))?;
        let client = notion_client(token)?;

        let result = match tool_name {
            "notion_search" => {
                let body = serde_json::json!({
                    "query":        arguments.get("query").and_then(|v| v.as_str()).unwrap_or(""),
                    "page_size":    arguments.get("limit").and_then(|v| v.as_u64()).unwrap_or(10),
                    "filter": if let Some(t) = arguments.get("filter") { serde_json::json!({"value": t, "property": "object"}) } else { Value::Null }
                });
                client.post(format!("{BASE}/search")).json(&body).send().await?.json::<Value>().await?
            }
            "notion_get_page" => {
                let id = arguments["page_id"].as_str().ok_or_else(|| anyhow::anyhow!("page_id manquant"))?;
                let blocks = client.get(format!("{BASE}/blocks/{id}/children?page_size=100")).send().await?.json::<Value>().await?;
                serde_json::json!({ "blocks": blocks })
            }
            "notion_create_page" => {
                let parent_id = arguments["parent_id"].as_str().ok_or_else(|| anyhow::anyhow!("parent_id manquant"))?;
                let title     = arguments["title"].as_str().unwrap_or("");
                let body = serde_json::json!({
                    "parent": { "page_id": parent_id },
                    "properties": { "title": { "title": [{ "text": { "content": title } }] } }
                });
                client.post(format!("{BASE}/pages")).json(&body).send().await?.json::<Value>().await?
            }
            "notion_append_block" => {
                let page_id = arguments["page_id"].as_str().ok_or_else(|| anyhow::anyhow!("page_id manquant"))?;
                let content = arguments.get("content").and_then(|v| v.as_str()).unwrap_or("");
                let body = serde_json::json!({
                    "children": [{ "object": "block", "type": "paragraph", "paragraph": { "rich_text": [{ "type": "text", "text": { "content": content } }] } }]
                });
                client.patch(format!("{BASE}/blocks/{page_id}/children")).json(&body).send().await?.json::<Value>().await?
            }
            _ => anyhow::bail!("Outil inconnu : {tool_name}"),
        };

        Ok(McpResult { content: result, is_error: false })
    }

    async fn read_resource(&self, uri: &str, config: &HashMap<String, Value>) -> anyhow::Result<McpResult> {
        let page_id = uri.strip_prefix("notion://").unwrap_or(uri);
        let token   = config["token"].as_str().ok_or_else(|| anyhow::anyhow!("token manquant"))?;
        let client  = notion_client(token)?;
        let content = client.get(format!("{BASE}/blocks/{page_id}/children?page_size=100")).send().await?.json::<Value>().await?;
        Ok(McpResult { content, is_error: false })
    }
}

fn notion_client(token: &str) -> anyhow::Result<reqwest::Client> {
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert("Authorization",    format!("Bearer {token}").parse()?);
    headers.insert("Notion-Version",   "2022-06-28".parse()?);
    headers.insert("Content-Type",     "application/json".parse()?);
    Ok(reqwest::Client::builder().default_headers(headers).build()?)
}
