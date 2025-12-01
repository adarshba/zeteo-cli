use super::{AiProvider, ChatRequest, ChatResponse, FunctionCall, Tool, ToolCall};
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
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<VertexTool>>,
}

#[derive(Serialize)]
struct VertexContent {
    role: String,
    parts: Vec<VertexPart>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
enum VertexPart {
    Text {
        text: String,
    },
    FunctionCall {
        function_call: VertexFunctionCall,
    },
    FunctionResponse {
        function_response: VertexFunctionResponse,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct VertexFunctionCall {
    name: String,
    args: serde_json::Value,
}

#[derive(Serialize, Deserialize, Debug)]
struct VertexFunctionResponse {
    name: String,
    response: serde_json::Value,
}

#[derive(Serialize)]
struct VertexTool {
    function_declarations: Vec<VertexFunctionDeclaration>,
}

#[derive(Serialize)]
struct VertexFunctionDeclaration {
    name: String,
    description: String,
    parameters: serde_json::Value,
}

#[derive(Serialize)]
struct GenerationConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_output_tokens: Option<u32>,
}

#[derive(Deserialize, Debug)]
struct VertexResponse {
    candidates: Vec<VertexCandidate>,
}

#[derive(Deserialize, Debug)]
struct VertexCandidate {
    content: VertexResponseContent,
}

#[derive(Deserialize, Debug)]
struct VertexResponseContent {
    parts: Vec<VertexResponsePart>,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
enum VertexResponsePart {
    Text { text: String },
    FunctionCall { function_call: VertexFunctionCall },
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

    fn convert_tools(tools: &[Tool]) -> Vec<VertexTool> {
        vec![VertexTool {
            function_declarations: tools
                .iter()
                .map(|t| VertexFunctionDeclaration {
                    name: t.function.name.clone(),
                    description: t.function.description.clone(),
                    parameters: t.function.parameters.clone(),
                })
                .collect(),
        }]
    }
}

#[async_trait::async_trait]
impl AiProvider for VertexProvider {
    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse> {
        let token = Self::get_access_token().await?;

        let contents: Vec<VertexContent> = request
            .messages
            .iter()
            .filter(|m| m.role != "system")
            .map(|m| {
                let role = match m.role.as_str() {
                    "assistant" => "model".to_string(),
                    "tool" => "function".to_string(),
                    _ => m.role.clone(),
                };

                let parts = if m.role == "tool" {
                    vec![VertexPart::FunctionResponse {
                        function_response: VertexFunctionResponse {
                            name: m.tool_call_id.clone().unwrap_or_default(),
                            response: serde_json::from_str(&m.content)
                                .unwrap_or(serde_json::json!({"result": m.content})),
                        },
                    }]
                } else if let Some(tool_calls) = &m.tool_calls {
                    tool_calls
                        .iter()
                        .map(|tc| VertexPart::FunctionCall {
                            function_call: VertexFunctionCall {
                                name: tc.function.name.clone(),
                                args: serde_json::from_str(&tc.function.arguments)
                                    .unwrap_or_default(),
                            },
                        })
                        .collect()
                } else {
                    vec![VertexPart::Text {
                        text: m.content.clone(),
                    }]
                };

                VertexContent { role, parts }
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

        let tools = request.tools.as_ref().map(|t| Self::convert_tools(t));

        let vertex_request = VertexRequest {
            contents,
            generation_config,
            tools,
        };

        let url = format!(
            "https://{}-aiplatform.googleapis.com/v1/projects/{}/locations/{}/publishers/google/models/{}:generateContent",
            self.location, self.project_id, self.location, self.model
        );

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .json(&vertex_request)
            .send()
            .await
            .context("Failed to send request to Vertex AI")?;

        if !response.status().is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            anyhow::bail!("Vertex AI API error: {}", error_text);
        }

        let vertex_response: VertexResponse = response
            .json()
            .await
            .context("Failed to parse Vertex AI response")?;

        let candidate = vertex_response
            .candidates
            .first()
            .context("No candidates in Vertex AI response")?;

        let mut content = String::new();
        let mut tool_calls = Vec::new();

        for (idx, part) in candidate.content.parts.iter().enumerate() {
            match part {
                VertexResponsePart::Text { text } => {
                    content.push_str(text);
                }
                VertexResponsePart::FunctionCall { function_call } => {
                    tool_calls.push(ToolCall {
                        id: format!("call_{}", idx),
                        call_type: "function".to_string(),
                        function: FunctionCall {
                            name: function_call.name.clone(),
                            arguments: serde_json::to_string(&function_call.args)
                                .unwrap_or_default(),
                        },
                    });
                }
            }
        }

        Ok(ChatResponse {
            content,
            model: self.model.clone(),
            tool_calls: if tool_calls.is_empty() {
                None
            } else {
                Some(tool_calls)
            },
        })
    }

    fn provider_name(&self) -> &str {
        "Vertex AI"
    }
}

impl VertexProvider {
    async fn get_access_token() -> Result<String> {
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
