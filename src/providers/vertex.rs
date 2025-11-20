use super::{AiProvider, ChatRequest, ChatResponse};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct VertexProvider {
    project_id: String,
    location: String,
    model: String,
    client: reqwest::Client,
}

#[derive(Serialize)]
struct VertexRequest {
    contents: Vec<VertexContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    generation_config: Option<GenerationConfig>,
}

#[derive(Serialize)]
struct VertexContent {
    role: String,
    parts: Vec<VertexPart>,
}

#[derive(Serialize)]
struct VertexPart {
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
struct VertexResponse {
    candidates: Vec<VertexCandidate>,
}

#[derive(Deserialize)]
struct VertexCandidate {
    content: VertexResponseContent,
}

#[derive(Deserialize)]
struct VertexResponseContent {
    parts: Vec<VertexResponsePart>,
}

#[derive(Deserialize)]
struct VertexResponsePart {
    text: String,
}

impl VertexProvider {
    pub fn new(project_id: String, location: String, model: Option<String>) -> Self {
        VertexProvider {
            project_id,
            location,
            model: model.unwrap_or_else(|| "gemini-pro".to_string()),
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait::async_trait]
impl AiProvider for VertexProvider {
    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse> {
        // Get access token from gcloud
        let token = Self::get_access_token().await?;
        
        let contents: Vec<VertexContent> = request
            .messages
            .iter()
            .map(|m| VertexContent {
                role: if m.role == "assistant" { "model".to_string() } else { m.role.clone() },
                parts: vec![VertexPart {
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
        
        let vertex_request = VertexRequest {
            contents,
            generation_config,
        };
        
        let url = format!(
            "https://{}-aiplatform.googleapis.com/v1/projects/{}/locations/{}/publishers/google/models/{}:generateContent",
            self.location, self.project_id, self.location, self.model
        );
        
        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .json(&vertex_request)
            .send()
            .await
            .context("Failed to send request to Vertex AI")?;
        
        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            anyhow::bail!("Vertex AI API error: {}", error_text);
        }
        
        let vertex_response: VertexResponse = response
            .json()
            .await
            .context("Failed to parse Vertex AI response")?;
        
        let content = vertex_response
            .candidates
            .first()
            .and_then(|c| c.content.parts.first())
            .map(|p| p.text.clone())
            .context("No content in Vertex AI response")?;
        
        Ok(ChatResponse {
            content,
            model: self.model.clone(),
        })
    }
    
    fn provider_name(&self) -> &str {
        "Vertex AI"
    }
}

impl VertexProvider {
    async fn get_access_token() -> Result<String> {
        // Try to get access token from gcloud CLI
        let output = tokio::process::Command::new("gcloud")
            .args(["auth", "print-access-token"])
            .output()
            .await;
        
        match output {
            Ok(output) if output.status.success() => {
                let token = String::from_utf8(output.stdout)
                    .context("Invalid UTF-8 in gcloud output")?
                    .trim()
                    .to_string();
                Ok(token)
            }
            _ => {
                anyhow::bail!(
                    "Failed to get access token. Please run 'gcloud auth application-default login'"
                )
            }
        }
    }
}

