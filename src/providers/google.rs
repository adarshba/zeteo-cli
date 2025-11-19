use super::{AiProvider, ChatRequest, ChatResponse};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct GoogleProvider {
    api_key: String,
    model: String,
    client: reqwest::Client,
}

#[derive(Serialize)]
struct GoogleRequest {
    contents: Vec<GoogleContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    generation_config: Option<GenerationConfig>,
}

#[derive(Serialize)]
struct GoogleContent {
    role: String,
    parts: Vec<GooglePart>,
}

#[derive(Serialize)]
struct GooglePart {
    text: String,
}

#[derive(Serialize)]
struct GenerationConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_output_tokens: Option<u32>,
}

#[derive(Deserialize)]
struct GoogleResponse {
    candidates: Vec<GoogleCandidate>,
}

#[derive(Deserialize)]
struct GoogleCandidate {
    content: GoogleResponseContent,
}

#[derive(Deserialize)]
struct GoogleResponseContent {
    parts: Vec<GoogleResponsePart>,
}

#[derive(Deserialize)]
struct GoogleResponsePart {
    text: String,
}

impl GoogleProvider {
    pub fn new(api_key: String, model: Option<String>) -> Self {
        GoogleProvider {
            api_key,
            model: model.unwrap_or_else(|| "gemini-pro".to_string()),
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait::async_trait]
impl AiProvider for GoogleProvider {
    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse> {
        let contents: Vec<GoogleContent> = request
            .messages
            .iter()
            .map(|m| GoogleContent {
                role: if m.role == "assistant" { "model".to_string() } else { m.role.clone() },
                parts: vec![GooglePart {
                    text: m.content.clone(),
                }],
            })
            .collect();
        
        let generation_config = if request.temperature.is_some() || request.max_tokens.is_some() {
            Some(GenerationConfig {
                temperature: request.temperature,
                max_output_tokens: request.max_tokens,
            })
        } else {
            None
        };
        
        let google_request = GoogleRequest {
            contents,
            generation_config,
        };
        
        let url = format!(
            "https://generativelanguage.googleapis.com/v1/models/{}:generateContent?key={}",
            self.model, self.api_key
        );
        
        let response = self.client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&google_request)
            .send()
            .await
            .context("Failed to send request to Google AI")?;
        
        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            anyhow::bail!("Google AI API error: {}", error_text);
        }
        
        let google_response: GoogleResponse = response
            .json()
            .await
            .context("Failed to parse Google AI response")?;
        
        let content = google_response
            .candidates
            .first()
            .and_then(|c| c.content.parts.first())
            .map(|p| p.text.clone())
            .context("No content in Google AI response")?;
        
        Ok(ChatResponse {
            content,
            model: self.model.clone(),
        })
    }
    
    fn provider_name(&self) -> &str {
        "Google AI"
    }
}

