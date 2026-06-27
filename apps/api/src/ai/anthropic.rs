use async_trait::async_trait;
use futures::{stream, StreamExt};
use serde_json::json;

use super::provider::{AiContext, AiProviderAdapter, ChatStream};

const ANTHROPIC_API: &str = "https://api.anthropic.com/v1/messages";
const ANTHROPIC_VERSION: &str = "2023-06-01";

pub struct AnthropicAdapter {
    api_key: String,
    model: String,
    http: reqwest::Client,
}

impl AnthropicAdapter {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            model: "claude-sonnet-4-6".into(),
            http: reqwest::Client::new(),
        }
    }

    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }
}

#[async_trait]
impl AiProviderAdapter for AnthropicAdapter {
    fn provider_id(&self) -> &str {
        "anthropic"
    }

    async fn chat_stream(&self, context: AiContext) -> anyhow::Result<ChatStream> {
        let system = context.system_layers.join("\n\n---\n\n");

        let mut body = json!({
            "model": self.model,
            "max_tokens": 8192,
            "stream": true,
            "system": system,
            "messages": context.messages,
        });

        if !context.tools.is_empty() {
            body["tools"] = json!(context.tools);
        }

        let response = self
            .http
            .post(ANTHROPIC_API)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", ANTHROPIC_VERSION)
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            let err = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("Anthropic API error: {err}"));
        }

        let byte_stream = response.bytes_stream();

        // Parse SSE: data: {"type":"content_block_delta","delta":{"text":"..."}}
        let text_stream = byte_stream
            .map(|chunk| {
                let bytes = chunk.map_err(|e| anyhow::anyhow!(e))?;
                let text = String::from_utf8_lossy(&bytes).to_string();
                let mut collected = String::new();
                for line in text.lines() {
                    if let Some(data) = line.strip_prefix("data: ") {
                        if data == "[DONE]" {
                            break;
                        }
                        if let Ok(val) = serde_json::from_str::<serde_json::Value>(data) {
                            if val["type"] == "content_block_delta" {
                                if let Some(t) = val["delta"]["text"].as_str() {
                                    collected.push_str(t);
                                }
                            }
                        }
                    }
                }
                Ok(collected)
            })
            .filter(|r| {
                let keep = match r {
                    Ok(s) => !s.is_empty(),
                    Err(_) => true,
                };
                futures::future::ready(keep)
            });

        Ok(Box::pin(text_stream))
    }
}
