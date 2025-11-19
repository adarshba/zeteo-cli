use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRequest {
    pub messages: Vec<Message>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    pub content: String,
    pub model: String,
}

#[async_trait::async_trait]
pub trait AiProvider: Send + Sync {
    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse>;
    fn provider_name(&self) -> &str;
}

pub mod openai;
pub mod vertex;
pub mod google;
pub mod azure;

pub use openai::OpenAiProvider;
pub use vertex::VertexProvider;
pub use google::GoogleProvider;
pub use azure::AzureProvider;
