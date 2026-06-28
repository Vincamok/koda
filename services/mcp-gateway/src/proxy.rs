/// Community MCP connector proxy — spawns a user-defined stdio process and routes
/// MCP JSON-RPC calls to it. This activates community connectors from the marketplace.
///
/// Each binding with `connector_type = 'stdio'` specifies a `command` in its config.
/// The gateway spawns it on demand, sends JSON-RPC over stdin, and reads stdout.
use std::collections::HashMap;

use serde_json::Value;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};

use crate::connectors::McpResult;

pub struct StdioConnector {
    /// The command to execute (e.g. `["npx", "-y", "@modelcontextprotocol/server-github"]`)
    pub command: Vec<String>,
    /// Environment variables passed to the subprocess
    pub env: HashMap<String, String>,
}

impl StdioConnector {
    pub fn from_config(config: &HashMap<String, Value>) -> anyhow::Result<Self> {
        let cmd = config
            .get("command")
            .and_then(|v| v.as_array())
            .ok_or_else(|| anyhow::anyhow!("stdio connector config missing 'command' array"))?;

        let command: Vec<String> = cmd
            .iter()
            .filter_map(|v| v.as_str().map(str::to_string))
            .collect();

        if command.is_empty() {
            anyhow::bail!("stdio connector 'command' must be non-empty");
        }

        let env: HashMap<String, String> = config
            .get("env")
            .and_then(|v| v.as_object())
            .map(|obj| {
                obj.iter()
                    .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                    .collect()
            })
            .unwrap_or_default();

        Ok(Self { command, env })
    }

    pub async fn call_tool(
        &self,
        tool_name: &str,
        arguments: HashMap<String, Value>,
        request_id: u64,
    ) -> anyhow::Result<McpResult> {
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": request_id,
            "method": "tools/call",
            "params": {
                "name": tool_name,
                "arguments": arguments,
            }
        });

        let response = self.roundtrip(request).await?;
        let is_error = response
            .get("error")
            .map(|e| !e.is_null())
            .unwrap_or(false);

        let content = response
            .get("result")
            .cloned()
            .unwrap_or(response.clone());

        Ok(McpResult { content, is_error })
    }

    pub async fn read_resource(
        &self,
        uri: &str,
        request_id: u64,
    ) -> anyhow::Result<McpResult> {
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": request_id,
            "method": "resources/read",
            "params": { "uri": uri }
        });

        let response = self.roundtrip(request).await?;
        let is_error = response.get("error").map(|e| !e.is_null()).unwrap_or(false);
        let content = response.get("result").cloned().unwrap_or(response);

        Ok(McpResult { content, is_error })
    }

    /// Spawn the process, send one JSON-RPC request on stdin, read one response from stdout.
    async fn roundtrip(&self, request: Value) -> anyhow::Result<Value> {
        let mut child = self.spawn()?;
        let stdin = child.stdin.as_mut().ok_or_else(|| anyhow::anyhow!("no stdin on child process"))?;
        let stdout = child.stdout.take().ok_or_else(|| anyhow::anyhow!("no stdout on child process"))?;

        let line = serde_json::to_string(&request)? + "\n";
        stdin.write_all(line.as_bytes()).await?;
        stdin.flush().await?;

        let mut reader = BufReader::new(stdout);
        let mut response_line = String::new();

        // Read until we get a non-empty line (JSON-RPC response)
        let timeout = tokio::time::timeout(
            std::time::Duration::from_secs(30),
            async {
                loop {
                    response_line.clear();
                    reader.read_line(&mut response_line).await?;
                    let trimmed = response_line.trim();
                    if !trimmed.is_empty() {
                        return Ok::<String, anyhow::Error>(trimmed.to_string());
                    }
                }
            },
        )
        .await
        .map_err(|_| anyhow::anyhow!("stdio connector timed out after 30s"))??;

        // Clean up the process
        let _ = child.kill().await;

        let value: Value = serde_json::from_str(&timeout)
            .map_err(|e| anyhow::anyhow!("invalid JSON response from stdio connector: {e}"))?;

        Ok(value)
    }

    fn spawn(&self) -> anyhow::Result<Child> {
        let (program, args) = self.command.split_first()
            .ok_or_else(|| anyhow::anyhow!("empty command"))?;

        let mut cmd = Command::new(program);
        cmd.args(args)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::null())
            .envs(&self.env)
            // Run in a tmpdir with no access to the host filesystem beyond what's mounted
            .current_dir("/tmp");

        Ok(cmd.spawn()?)
    }
}
