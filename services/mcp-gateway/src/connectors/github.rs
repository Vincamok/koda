use std::collections::HashMap;
use async_trait::async_trait;
use serde_json::Value;
use super::{McpConnector, McpResult};

const BASE: &str = "https://api.github.com";

pub struct GitHubConnector;

#[async_trait]
impl McpConnector for GitHubConnector {
    fn id(&self) -> &'static str { "github" }

    fn list_tools(&self) -> Vec<&'static str> {
        vec!["github_list_issues", "github_get_pr", "github_search_code", "github_create_issue", "github_comment_pr"]
    }

    async fn call_tool(
        &self,
        tool_name: &str,
        arguments: HashMap<String, Value>,
        config: &HashMap<String, Value>,
    ) -> anyhow::Result<McpResult> {
        let token  = config["token"].as_str().ok_or_else(|| anyhow::anyhow!("token manquant"))?;
        let owner  = config["owner"].as_str().ok_or_else(|| anyhow::anyhow!("owner manquant"))?;
        let repo   = config.get("repo").and_then(|v| v.as_str()).unwrap_or("");
        let client = gh_client(token)?;

        let result = match tool_name {
            "github_list_issues" => {
                let state = arguments.get("state").and_then(|v| v.as_str()).unwrap_or("open");
                let limit = arguments.get("limit").and_then(|v| v.as_u64()).unwrap_or(20);
                let url = format!("{BASE}/repos/{owner}/{repo}/issues?state={state}&per_page={limit}");
                client.get(&url).send().await?.json::<Value>().await?
            }
            "github_get_pr" => {
                let number = arguments["number"].as_u64().ok_or_else(|| anyhow::anyhow!("number manquant"))?;
                client.get(format!("{BASE}/repos/{owner}/{repo}/pulls/{number}")).send().await?.json::<Value>().await?
            }
            "github_search_code" => {
                let query = arguments["query"].as_str().ok_or_else(|| anyhow::anyhow!("query manquant"))?;
                let q = if repo.is_empty() { format!("{query} user:{owner}") } else { format!("{query} repo:{owner}/{repo}") };
                client.get(format!("{BASE}/search/code?q={}", urlencoding::encode(&q))).send().await?.json::<Value>().await?
            }
            "github_create_issue" => {
                let body = serde_json::json!({
                    "title": arguments.get("title").and_then(|v| v.as_str()).unwrap_or(""),
                    "body":  arguments.get("body").and_then(|v| v.as_str()).unwrap_or(""),
                    "labels": arguments.get("labels").cloned().unwrap_or(Value::Array(vec![])),
                });
                client.post(format!("{BASE}/repos/{owner}/{repo}/issues")).json(&body).send().await?.json::<Value>().await?
            }
            "github_comment_pr" => {
                let number = arguments["number"].as_u64().ok_or_else(|| anyhow::anyhow!("number manquant"))?;
                let body   = serde_json::json!({ "body": arguments["body"].as_str().unwrap_or("") });
                client.post(format!("{BASE}/repos/{owner}/{repo}/issues/{number}/comments")).json(&body).send().await?.json::<Value>().await?
            }
            _ => anyhow::bail!("Outil inconnu : {tool_name}"),
        };

        Ok(McpResult { content: result, is_error: false })
    }

    async fn read_resource(&self, uri: &str, config: &HashMap<String, Value>) -> anyhow::Result<McpResult> {
        let token  = config["token"].as_str().ok_or_else(|| anyhow::anyhow!("token manquant"))?;
        let client = gh_client(token)?;
        // uri format: github://{owner}/{repo}/blob/{branch}/{path}
        let api_url = uri_to_api_url(uri)?;
        let content = client.get(&api_url).send().await?.json::<Value>().await?;
        Ok(McpResult { content, is_error: false })
    }
}

fn gh_client(token: &str) -> anyhow::Result<reqwest::Client> {
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert("Authorization",        format!("Bearer {token}").parse()?);
    headers.insert("Accept",               "application/vnd.github+json".parse()?);
    headers.insert("X-GitHub-Api-Version", "2022-11-28".parse()?);
    headers.insert("User-Agent",           "koda-mcp-gateway/0.1".parse()?);
    Ok(reqwest::Client::builder().default_headers(headers).build()?)
}

fn uri_to_api_url(uri: &str) -> anyhow::Result<String> {
    // github://{owner}/{repo}/blob/{branch}/{path} → /repos/{owner}/{repo}/contents/{path}?ref={branch}
    let path = uri.strip_prefix("github://").unwrap_or(uri);
    let parts: Vec<&str> = path.splitn(5, '/').collect();
    if parts.len() < 5 || parts[2] != "blob" {
        anyhow::bail!("URI GitHub invalide : {uri}");
    }
    let (owner, repo, branch, file_path) = (parts[0], parts[1], parts[3], parts[4]);
    Ok(format!("{BASE}/repos/{owner}/{repo}/contents/{file_path}?ref={branch}"))
}
