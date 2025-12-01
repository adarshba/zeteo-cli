use super::{AiProvider, ChatRequest, ChatResponse, FunctionCall, Tool, ToolCall};
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
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<Tool>>,
}

#[derive(Serialize, Deserialize, Debug)]
struct OpenAiMessage {
    role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_calls: Option<Vec<OpenAiToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_call_id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct OpenAiToolCall {
    id: String,
    #[serde(rename = "type")]
    call_type: String,
    function: OpenAiFunctionCall,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct OpenAiFunctionCall {
    name: String,
    arguments: String,
}

#[derive(Deserialize, Debug)]
struct OpenAiResponse {
    choices: Vec<OpenAiChoice>,
}

#[derive(Deserialize, Debug)]
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
            .map(|m| {
                // OpenAI requires content to be non-null for user and tool messages
                // Only assistant messages with tool_calls can have null content
                let content =
                    if m.role == "assistant" && m.tool_calls.is_some() && m.content.is_empty() {
                        None
                    } else if m.content.is_empty() {
                        Some(String::new()) // Send empty string instead of null
                    } else {
                        Some(m.content.clone())
                    };

                OpenAiMessage {
                    role: m.role.clone(),
                    content,
                    tool_calls: m.tool_calls.as_ref().map(|tcs| {
                        tcs.iter()
                            .map(|tc| OpenAiToolCall {
                                id: tc.id.clone(),
                                call_type: tc.call_type.clone(),
                                function: OpenAiFunctionCall {
                                    name: tc.function.name.clone(),
                                    arguments: tc.function.arguments.clone(),
                                },
                            })
                            .collect()
                    }),
                    tool_call_id: m.tool_call_id.clone(),
                }
            })
            .collect();

        let openai_request = OpenAiRequest {
            model: self.model.clone(),
            messages,
            temperature: request.temperature,
            max_tokens: request.max_tokens,
            tools: request.tools,
        };

        let response = self
            .client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&openai_request)
            .send()
            .await
            .context("Failed to send request to OpenAI")?;

        if !response.status().is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            anyhow::bail!("OpenAI API error: {}", error_text);
        }

        let openai_response: OpenAiResponse = response
            .json()
            .await
            .context("Failed to parse OpenAI response")?;

        let choice = openai_response
            .choices
            .first()
            .context("No choices in OpenAI response")?;

        let content = choice.message.content.clone().unwrap_or_default();

        let tool_calls = choice.message.tool_calls.as_ref().map(|tcs| {
            tcs.iter()
                .map(|tc| ToolCall {
                    id: tc.id.clone(),
                    call_type: tc.call_type.clone(),
                    function: FunctionCall {
                        name: tc.function.name.clone(),
                        arguments: tc.function.arguments.clone(),
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
        "OpenAI"
    }
}
