use async_trait::async_trait;
use futures::Stream;
use serde::{Deserialize, Serialize};
use std::pin::Pin;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone)]
pub struct AiContext {
    /// Ordered context layers (platform → org → lang packs → framework packs → workspace → personal)
    pub system_layers: Vec<String>,
    pub messages: Vec<ChatMessage>,
    pub tools: Vec<serde_json::Value>,
}

pub type StreamChunk = Result<String, anyhow::Error>;
pub type ChatStream = Pin<Box<dyn Stream<Item = StreamChunk> + Send>>;

#[async_trait]
pub trait AiProviderAdapter: Send + Sync {
    fn provider_id(&self) -> &str;
    async fn chat_stream(&self, context: AiContext) -> anyhow::Result<ChatStream>;
}
