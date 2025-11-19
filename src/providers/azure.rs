use super::{AiProvider, ChatRequest, ChatResponse};
use anyhow::Result;

#[derive(Clone)]
pub struct AzureProvider {
    api_key: String,
    endpoint: String,
    deployment: String,
}

impl AzureProvider {
    pub fn new(api_key: String, endpoint: String, deployment: String) -> Self {
        AzureProvider {
            api_key,
            endpoint,
            deployment,
        }
    }
}

#[async_trait::async_trait]
impl AiProvider for AzureProvider {
    async fn chat(&self, _request: ChatRequest) -> Result<ChatResponse> {
        // Placeholder implementation
        // Full implementation would use Azure OpenAI API
        Ok(ChatResponse {
            content: "Azure OpenAI provider not yet fully implemented".to_string(),
            model: self.deployment.clone(),
        })
    }
    
    fn provider_name(&self) -> &str {
        "Azure OpenAI"
    }
}
