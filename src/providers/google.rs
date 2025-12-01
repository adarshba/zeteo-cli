use super::{AiProvider, ChatRequest, ChatResponse, Tool, ToolCall, FunctionCall};
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
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<GoogleTool>>,
}

#[derive(Serialize)]
struct GoogleContent {
    role: String,
    parts: Vec<GooglePart>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
enum GooglePart {
    Text { text: String },
    FunctionCall { function_call: GoogleFunctionCall },
    FunctionResponse { function_response: GoogleFunctionResponse },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct GoogleFunctionCall {
    name: String,
    args: serde_json::Value,
}

#[derive(Serialize, Deserialize, Debug)]
struct GoogleFunctionResponse {
    name: String,
    response: serde_json::Value,
}

#[derive(Serialize)]
struct GoogleTool {
    function_declarations: Vec<GoogleFunctionDeclaration>,
}

#[derive(Serialize)]
struct GoogleFunctionDeclaration {
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
struct GoogleResponse {
    candidates: Vec<GoogleCandidate>,
}

#[derive(Deserialize, Debug)]
struct GoogleCandidate {
    content: GoogleResponseContent,
}

#[derive(Deserialize, Debug)]
struct GoogleResponseContent {
    parts: Vec<GoogleResponsePart>,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
enum GoogleResponsePart {
    Text { text: String },
    FunctionCall { function_call: GoogleFunctionCall },
}

impl GoogleProvider {
    pub fn new(api_key: String, model: Option<String>) -> Self {
        GoogleProvider {
            api_key,
            model: model.unwrap_or_else(|| "gemini-pro".to_string()),
            client: reqwest::Client::new(),
        }
    }
    
    fn convert_tools(tools: &[Tool]) -> Vec<GoogleTool> {
        vec![GoogleTool {
            function_declarations: tools.iter().map(|t| GoogleFunctionDeclaration {
                name: t.function.name.clone(),
                description: t.function.description.clone(),
                parameters: t.function.parameters.clone(),
            }).collect(),
        }]
    }
}

#[async_trait::async_trait]
impl AiProvider for GoogleProvider {
    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse> {
        let contents: Vec<GoogleContent> = request
            .messages
            .iter()
            .filter(|m| m.role != "system") // Google doesn't support system role directly
            .map(|m| {
                let role = match m.role.as_str() {
                    "assistant" => "model".to_string(),
                    "tool" => "function".to_string(),
                    _ => m.role.clone(),
                };
                
                let parts = if m.role == "tool" {
                    // Tool response
                    vec![GooglePart::FunctionResponse {
                        function_response: GoogleFunctionResponse {
                            name: m.tool_call_id.clone().unwrap_or_default(),
                            response: serde_json::from_str(&m.content).unwrap_or(serde_json::json!({"result": m.content})),
                        }
                    }]
                } else if let Some(tool_calls) = &m.tool_calls {
                    // Assistant with function calls
                    tool_calls.iter().map(|tc| {
                        GooglePart::FunctionCall {
                            function_call: GoogleFunctionCall {
                                name: tc.function.name.clone(),
                                args: serde_json::from_str(&tc.function.arguments).unwrap_or_default(),
                            }
                        }
                    }).collect()
                } else {
                    vec![GooglePart::Text { text: m.content.clone() }]
                };
                
                GoogleContent { role, parts }
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
        
        let google_request = GoogleRequest {
            contents,
            generation_config,
            tools,
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
        
        let candidate = google_response
            .candidates
            .first()
            .context("No candidates in Google AI response")?;
        
        // Extract text content and function calls
        let mut content = String::new();
        let mut tool_calls = Vec::new();
        
        for (idx, part) in candidate.content.parts.iter().enumerate() {
            match part {
                GoogleResponsePart::Text { text } => {
                    content.push_str(text);
                }
                GoogleResponsePart::FunctionCall { function_call } => {
                    tool_calls.push(ToolCall {
                        id: format!("call_{}", idx),
                        call_type: "function".to_string(),
                        function: FunctionCall {
                            name: function_call.name.clone(),
                            arguments: serde_json::to_string(&function_call.args).unwrap_or_default(),
                        },
                    });
                }
            }
        }
        
        Ok(ChatResponse {
            content,
            model: self.model.clone(),
            tool_calls: if tool_calls.is_empty() { None } else { Some(tool_calls) },
        })
    }
    
    fn provider_name(&self) -> &str {
        "Google AI"
    }
}

