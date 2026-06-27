use std::collections::HashMap;
use async_trait::async_trait;
use serde_json::Value;
use super::{McpConnector, McpResult};

const BASE: &str = "https://slack.com/api";

pub struct SlackConnector;

#[async_trait]
impl McpConnector for SlackConnector {
    fn id(&self) -> &'static str { "slack" }

    fn list_tools(&self) -> Vec<&'static str> {
        vec!["slack_post_message", "slack_search_messages", "slack_list_channels"]
    }

    async fn call_tool(
        &self,
        tool_name: &str,
        arguments: HashMap<String, Value>,
        config: &HashMap<String, Value>,
    ) -> anyhow::Result<McpResult> {
        let token  = config["bot_token"].as_str().ok_or_else(|| anyhow::anyhow!("bot_token manquant"))?;
        let client = slack_client(token)?;

        let result = match tool_name {
            "slack_post_message" => {
                let body = serde_json::json!({
                    "channel":   arguments.get("channel").and_then(|v| v.as_str()).unwrap_or(""),
                    "text":      arguments.get("text").and_then(|v| v.as_str()).unwrap_or(""),
                    "thread_ts": arguments.get("thread_ts"),
                });
                client.post(format!("{BASE}/chat.postMessage")).json(&body).send().await?.json::<Value>().await?
            }
            "slack_search_messages" => {
                let query = arguments["query"].as_str().ok_or_else(|| anyhow::anyhow!("query manquant"))?;
                let limit = arguments.get("limit").and_then(|v| v.as_u64()).unwrap_or(20);
                client.get(format!("{BASE}/search.messages?query={}&count={limit}", urlencoding::encode(query)))
                    .send().await?.json::<Value>().await?
            }
            "slack_list_channels" => {
                let limit = arguments.get("limit").and_then(|v| v.as_u64()).unwrap_or(50);
                client.get(format!("{BASE}/conversations.list?limit={limit}&exclude_archived=true"))
                    .send().await?.json::<Value>().await?
            }
            _ => anyhow::bail!("Outil inconnu : {tool_name}"),
        };

        Ok(McpResult { content: result, is_error: false })
    }

    async fn read_resource(&self, _uri: &str, _config: &HashMap<String, Value>) -> anyhow::Result<McpResult> {
        anyhow::bail!("read_resource non supporté pour Slack")
    }
}

fn slack_client(token: &str) -> anyhow::Result<reqwest::Client> {
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert("Authorization", format!("Bearer {token}").parse()?);
    headers.insert("Content-Type",  "application/json".parse()?);
    Ok(reqwest::Client::builder().default_headers(headers).build()?)
}
