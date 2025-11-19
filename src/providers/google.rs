use super::{AiProvider, ChatRequest, ChatResponse};
use anyhow::Result;

#[derive(Clone)]
pub struct GoogleProvider {
    api_key: String,
    model: String,
}

impl GoogleProvider {
    pub fn new(api_key: String, model: Option<String>) -> Self {
        GoogleProvider {
            api_key,
            model: model.unwrap_or_else(|| "gemini-pro".to_string()),
        }
    }
}

#[async_trait::async_trait]
impl AiProvider for GoogleProvider {
    async fn chat(&self, _request: ChatRequest) -> Result<ChatResponse> {
        // Placeholder implementation
        // Full implementation would use Google AI API
        Ok(ChatResponse {
            content: "Google AI provider not yet fully implemented".to_string(),
            model: self.model.clone(),
        })
    }
    
    fn provider_name(&self) -> &str {
        "Google AI"
    }
}
