use std::collections::HashMap;
use async_trait::async_trait;
use serde_json::Value;
use super::{McpConnector, McpResult};

pub struct JiraConnector;

#[async_trait]
impl McpConnector for JiraConnector {
    fn id(&self) -> &'static str { "jira" }

    fn list_tools(&self) -> Vec<&'static str> {
        vec!["jira_search_issues", "jira_get_issue", "jira_create_issue", "jira_transition_issue"]
    }

    async fn call_tool(
        &self,
        tool_name: &str,
        arguments: HashMap<String, Value>,
        config: &HashMap<String, Value>,
    ) -> anyhow::Result<McpResult> {
        let base_url = config["base_url"].as_str().ok_or_else(|| anyhow::anyhow!("base_url manquant"))?;
        let email   = config["email"].as_str().ok_or_else(|| anyhow::anyhow!("email manquant"))?;
        let token   = config["api_token"].as_str().ok_or_else(|| anyhow::anyhow!("api_token manquant"))?;
        let client  = jira_client(email, token)?;

        let result = match tool_name {
            "jira_search_issues" => {
                let jql   = arguments["jql"].as_str().ok_or_else(|| anyhow::anyhow!("jql manquant"))?;
                let limit = arguments.get("limit").and_then(|v| v.as_u64()).unwrap_or(20);
                let url   = format!("{base_url}/rest/api/3/search?jql={}&maxResults={limit}", urlencoding::encode(jql));
                client.get(url).send().await?.json::<Value>().await?
            }
            "jira_get_issue" => {
                let key = arguments["issue_key"].as_str().ok_or_else(|| anyhow::anyhow!("issue_key manquant"))?;
                client.get(format!("{base_url}/rest/api/3/issue/{key}")).send().await?.json::<Value>().await?
            }
            "jira_create_issue" => {
                let project_key  = arguments["project_key"].as_str().ok_or_else(|| anyhow::anyhow!("project_key manquant"))?;
                let summary      = arguments["summary"].as_str().ok_or_else(|| anyhow::anyhow!("summary manquant"))?;
                let issue_type   = arguments.get("issue_type").and_then(|v| v.as_str()).unwrap_or("Task");
                let description  = arguments.get("description").and_then(|v| v.as_str()).unwrap_or("");
                let body = serde_json::json!({
                    "fields": {
                        "project":   { "key": project_key },
                        "summary":   summary,
                        "issuetype": { "name": issue_type },
                        "description": {
                            "type":    "doc",
                            "version": 1,
                            "content": [{ "type": "paragraph", "content": [{ "type": "text", "text": description }] }]
                        }
                    }
                });
                client.post(format!("{base_url}/rest/api/3/issue")).json(&body).send().await?.json::<Value>().await?
            }
            "jira_transition_issue" => {
                let key           = arguments["issue_key"].as_str().ok_or_else(|| anyhow::anyhow!("issue_key manquant"))?;
                let transition_id = arguments["transition_id"].as_str().ok_or_else(|| anyhow::anyhow!("transition_id manquant"))?;
                let body = serde_json::json!({ "transition": { "id": transition_id } });
                client.post(format!("{base_url}/rest/api/3/issue/{key}/transitions")).json(&body).send().await?.json::<Value>().await?
            }
            _ => anyhow::bail!("Outil inconnu : {tool_name}"),
        };

        Ok(McpResult { content: result, is_error: false })
    }

    async fn read_resource(&self, _uri: &str, _config: &HashMap<String, Value>) -> anyhow::Result<McpResult> {
        anyhow::bail!("read_resource non supporté pour Jira")
    }
}

fn jira_client(email: &str, token: &str) -> anyhow::Result<reqwest::Client> {
    use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE, ACCEPT};
    let creds   = super::base64_encode(&format!("{email}:{token}"));
    let mut headers = HeaderMap::new();
    headers.insert(AUTHORIZATION, HeaderValue::from_str(&format!("Basic {creds}"))?);
    headers.insert(CONTENT_TYPE,  HeaderValue::from_static("application/json"));
    headers.insert(ACCEPT,        HeaderValue::from_static("application/json"));
    Ok(reqwest::Client::builder().default_headers(headers).build()?)
}
