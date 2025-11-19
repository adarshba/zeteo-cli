use super::{AiProvider, ChatRequest, ChatResponse};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct AzureProvider {
    api_key: String,
    endpoint: String,
    deployment: String,
    client: reqwest::Client,
}

#[derive(Serialize)]
struct AzureRequest {
    messages: Vec<AzureMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
}

#[derive(Serialize, Deserialize)]
struct AzureMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct AzureResponse {
    choices: Vec<AzureChoice>,
}

#[derive(Deserialize)]
struct AzureChoice {
    message: AzureMessage,
}

impl AzureProvider {
    pub fn new(api_key: String, endpoint: String, deployment: String) -> Self {
        AzureProvider {
            api_key,
            endpoint,
            deployment,
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait::async_trait]
impl AiProvider for AzureProvider {
    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse> {
        let messages: Vec<AzureMessage> = request
            .messages
            .iter()
            .map(|m| AzureMessage {
                role: m.role.clone(),
                content: m.content.clone(),
            })
            .collect();
        
        let azure_request = AzureRequest {
            messages,
            temperature: request.temperature,
            max_tokens: request.max_tokens,
        };
        
        // Azure OpenAI endpoint format:
        // https://{resource-name}.openai.azure.com/openai/deployments/{deployment-id}/chat/completions?api-version=2024-02-15-preview
        let url = format!(
            "{}/openai/deployments/{}/chat/completions?api-version=2024-02-15-preview",
            self.endpoint.trim_end_matches('/'),
            self.deployment
        );
        
        let response = self.client
            .post(&url)
            .header("api-key", &self.api_key)
            .header("Content-Type", "application/json")
            .json(&azure_request)
            .send()
            .await
            .context("Failed to send request to Azure OpenAI")?;
        
        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            anyhow::bail!("Azure OpenAI API error: {}", error_text);
        }
        
        let azure_response: AzureResponse = response
            .json()
            .await
            .context("Failed to parse Azure OpenAI response")?;
        
        let content = azure_response
            .choices
            .first()
            .context("No choices in Azure OpenAI response")?
            .message
            .content
            .clone();
        
        Ok(ChatResponse {
            content,
            model: self.deployment.clone(),
        })
    }
    
    fn provider_name(&self) -> &str {
        "Azure OpenAI"
    }
}

