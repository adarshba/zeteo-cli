use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub call_type: String,
    pub function: FunctionCall,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCall {
    pub name: String,
    pub arguments: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    #[serde(rename = "type")]
    pub tool_type: String,
    pub function: ToolFunction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolFunction {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRequest {
    pub messages: Vec<Message>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Tool>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    pub content: String,
    pub model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
}

#[async_trait::async_trait]
pub trait AiProvider: Send + Sync {
    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse>;
    #[allow(dead_code)]
    fn provider_name(&self) -> &str;
    #[allow(dead_code)]
    fn supports_tools(&self) -> bool {
        true
    }
}

pub mod azure;
pub mod google;
pub mod openai;
pub mod vertex;

pub use azure::AzureProvider;
pub use google::GoogleProvider;
pub use openai::OpenAiProvider;
pub use vertex::VertexProvider;

/// Create the log query tools definition for AI function calling
pub fn create_log_tools() -> Vec<Tool> {
    vec![
        Tool {
            tool_type: "function".to_string(),
            function: ToolFunction {
                name: "query_logs".to_string(),
                description: "Search and query logs from the observability backend. Use this to find errors, investigate issues, analyze patterns, or get log data for a specific service, time range, or search term.".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "query": {
                            "type": "string",
                            "description": "Search query string. Use '*' for all logs, or specific terms like 'error', 'payment failed', 'timeout', etc."
                        },
                        "max_results": {
                            "type": "integer",
                            "description": "Maximum number of log entries to return (default: 50, max: 200)",
                            "default": 50
                        },
                        "level": {
                            "type": "string",
                            "description": "Filter by log level: ERROR, WARN, INFO, DEBUG",
                            "enum": ["ERROR", "WARN", "INFO", "DEBUG"]
                        },
                        "service": {
                            "type": "string",
                            "description": "Filter by service name"
                        },
                        "start_time": {
                            "type": "string",
                            "description": "Start time in ISO 8601 format (e.g., 2024-01-01T00:00:00Z) or relative time like '1h' (1 hour ago), '30m' (30 minutes ago)"
                        },
                        "end_time": {
                            "type": "string",
                            "description": "End time in ISO 8601 format. Defaults to now."
                        }
                    },
                    "required": ["query"]
                }),
            },
        },
        Tool {
            tool_type: "function".to_string(),
            function: ToolFunction {
                name: "list_services".to_string(),
                description: "List available services in the log backend".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {},
                    "required": []
                }),
            },
        },
        Tool {
            tool_type: "function".to_string(),
            function: ToolFunction {
                name: "get_log_stats".to_string(),
                description: "Get aggregated statistics about logs: counts by level, by service, time distribution".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "start_time": {
                            "type": "string",
                            "description": "Start time for stats calculation"
                        },
                        "end_time": {
                            "type": "string",
                            "description": "End time for stats calculation"
                        }
                    },
                    "required": []
                }),
            },
        },
    ]
}
