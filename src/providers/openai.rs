use super::{AiProvider, ChatRequest, ChatResponse};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct OpenAiProvider {
    api_key: String,
    model: String,
    client: reqwest::Client,
}

#[derive(Serialize)]
struct OpenAiRequest {
    model: String,
    messages: Vec<OpenAiMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
}

#[derive(Serialize, Deserialize)]
struct OpenAiMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct OpenAiResponse {
    choices: Vec<OpenAiChoice>,
}

#[derive(Deserialize)]
struct OpenAiChoice {
    message: OpenAiMessage,
}

impl OpenAiProvider {
    pub fn new(api_key: String, model: Option<String>) -> Self {
        OpenAiProvider {
            api_key,
            model: model.unwrap_or_else(|| "gpt-4o".to_string()),
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait::async_trait]
impl AiProvider for OpenAiProvider {
    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse> {
        let messages: Vec<OpenAiMessage> = request
            .messages
            .iter()
            .map(|m| OpenAiMessage {
                role: m.role.clone(),
                content: m.content.clone(),
            })
            .collect();
        
        let openai_request = OpenAiRequest {
            model: self.model.clone(),
            messages,
            temperature: request.temperature,
            max_tokens: request.max_tokens,
        };
        
        let response = self.client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&openai_request)
            .send()
            .await
            .context("Failed to send request to OpenAI")?;
        
        let openai_response: OpenAiResponse = response
            .json()
            .await
            .context("Failed to parse OpenAI response")?;
        
        let content = openai_response
            .choices
            .first()
            .context("No choices in OpenAI response")?
            .message
            .content
            .clone();
        
        Ok(ChatResponse {
            content,
            model: self.model.clone(),
        })
    }
    
    fn provider_name(&self) -> &str {
        "OpenAI"
    }
}
