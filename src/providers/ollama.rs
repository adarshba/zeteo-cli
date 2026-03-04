use super::{AiProvider, ChatRequest, ChatResponse, FunctionCall, Tool, ToolCall};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct OllamaProvider {
    model: String,
    base_url: String,
    client: reqwest::Client,
}

#[derive(Serialize)]
struct OllamaRequest {
    model: String,
    messages: Vec<OllamaMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<Tool>>,
    stream: bool,
}

#[derive(Serialize, Deserialize, Debug)]
struct OllamaMessage {
    role: String,
    content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_calls: Option<Vec<OllamaToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_call_id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct OllamaToolCall {
    id: String,
    #[serde(rename = "type")]
    call_type: String,
    function: OllamaFunctionCall,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct OllamaFunctionCall {
    name: String,
    arguments: serde_json::Value,
}

#[derive(Deserialize, Debug)]
struct OllamaResponse {
    message: OllamaResponseMessage,
}

#[derive(Deserialize, Debug)]
struct OllamaResponseMessage {
    #[allow(dead_code)]
    role: String,
    content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_calls: Option<Vec<OllamaToolCall>>,
}

impl OllamaProvider {
    pub fn new(model: Option<String>, base_url: Option<String>) -> Self {
        OllamaProvider {
            model: model.unwrap_or_else(|| "llama3".to_string()),
            base_url: base_url.unwrap_or_else(|| "http://localhost:11434".to_string()),
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait::async_trait]
impl AiProvider for OllamaProvider {
    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse> {
        let messages: Vec<OllamaMessage> = request
            .messages
            .iter()
            .map(|m| {
                let content = if m.content.is_empty() {
                    String::new()
                } else {
                    m.content.clone()
                };

                OllamaMessage {
                    role: m.role.clone(),
                    content,
                    tool_calls: m.tool_calls.as_ref().map(|tcs| {
                        tcs.iter()
                            .map(|tc| OllamaToolCall {
                                id: tc.id.clone(),
                                call_type: tc.call_type.clone(),
                                function: OllamaFunctionCall {
                                    name: tc.function.name.clone(),
                                    arguments: serde_json::from_str(&tc.function.arguments)
                                        .unwrap_or(serde_json::Value::Object(
                                            serde_json::Map::new(),
                                        )),
                                },
                            })
                            .collect()
                    }),
                    tool_call_id: m.tool_call_id.clone(),
                }
            })
            .collect();

        let ollama_request = OllamaRequest {
            model: self.model.clone(),
            messages,
            temperature: request.temperature,
            max_tokens: request.max_tokens,
            tools: request.tools,
            stream: false,
        };

        let url = format!("{}/api/chat", self.base_url);
        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&ollama_request)
            .send()
            .await
            .context("Failed to send request to Ollama")?;

        if !response.status().is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            anyhow::bail!("Ollama API error: {}", error_text);
        }

        let ollama_response: OllamaResponse = response
            .json()
            .await
            .context("Failed to parse Ollama response")?;

        let content = ollama_response.message.content;

        let tool_calls = ollama_response.message.tool_calls.as_ref().map(|tcs| {
            tcs.iter()
                .map(|tc| ToolCall {
                    id: tc.id.clone(),
                    call_type: tc.call_type.clone(),
                    function: FunctionCall {
                        name: tc.function.name.clone(),
                        arguments: serde_json::to_string(&tc.function.arguments).unwrap_or_default(),
                    },
                })
                .collect()
        });

        Ok(ChatResponse {
            content,
            model: self.model.clone(),
            tool_calls,
        })
    }

    fn provider_name(&self) -> &str {
        "Ollama"
    }

    fn supports_tools(&self) -> bool {
        // Ollama has varying tool support depending on the model
        false
    }
}
