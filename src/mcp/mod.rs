use anyhow::{Context, Result, anyhow};
use serde::{Deserialize, Serialize};
use std::process::{Child, Command, Stdio, ChildStdin, ChildStdout};
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write, BufWriter};
use std::sync::{Arc, Mutex};

#[derive(Debug, Serialize, Deserialize)]
pub struct McpRequest {
    pub jsonrpc: String,
    pub id: u64,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct McpResponse {
    pub jsonrpc: String,
    pub id: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<McpError>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct McpError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ToolInfo {
    pub name: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_schema: Option<serde_json::Value>,
}

pub struct McpClient {
    stdin: Arc<Mutex<BufWriter<ChildStdin>>>,
    stdout: Arc<Mutex<BufReader<ChildStdout>>>,
    process: Option<Child>,
    #[allow(dead_code)]
    server_name: String,
    request_id: Arc<Mutex<u64>>,
    initialized: bool,
}

impl McpClient {
    pub fn new(
        command: &str,
        args: &[String],
        env: &HashMap<String, String>,
        server_name: String,
    ) -> Result<Self> {
        let mut cmd = Command::new(command);
        cmd.args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        
        for (key, value) in env {
            cmd.env(key, value);
        }
        
        let mut process = cmd.spawn()
            .context("Failed to start MCP server process")?;
        
        let stdin = process.stdin.take()
            .context("Failed to get stdin handle")?;
        let stdout = process.stdout.take()
            .context("Failed to get stdout handle")?;
        
        Ok(McpClient {
            stdin: Arc::new(Mutex::new(BufWriter::new(stdin))),
            stdout: Arc::new(Mutex::new(BufReader::new(stdout))),
            process: Some(process),
            server_name,
            request_id: Arc::new(Mutex::new(0)),
            initialized: false,
        })
    }
    
    fn next_request_id(&self) -> u64 {
        let mut id = self.request_id.lock().unwrap();
        *id += 1;
        *id
    }
    
    pub fn initialize(&mut self) -> Result<serde_json::Value> {
        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: self.next_request_id(),
            method: "initialize".to_string(),
            params: Some(serde_json::json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "tools": {}
                },
                "clientInfo": {
                    "name": "zeteo-cli",
                    "version": env!("CARGO_PKG_VERSION")
                }
            })),
        };
        
        let response = self.send_request(&request)?;
        
        if let Some(error) = response.error {
            return Err(anyhow!("Initialize failed: {}", error.message));
        }
        
        self.initialized = true;
        
        // Send initialized notification
        self.send_notification("notifications/initialized", None)?;
        
        response.result.ok_or_else(|| anyhow!("No result in initialize response"))
    }
    
    #[allow(dead_code)]
    pub fn list_tools(&self) -> Result<Vec<ToolInfo>> {
        if !self.initialized {
            return Err(anyhow!("Client not initialized. Call initialize() first"));
        }
        
        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: self.next_request_id(),
            method: "tools/list".to_string(),
            params: None,
        };
        
        let response = self.send_request(&request)?;
        
        if let Some(error) = response.error {
            return Err(anyhow!("List tools failed: {}", error.message));
        }
        
        let result = response.result.ok_or_else(|| anyhow!("No result in list tools response"))?;
        
        let tools = result.get("tools")
            .ok_or_else(|| anyhow!("No 'tools' field in response"))?;
        
        serde_json::from_value(tools.clone())
            .context("Failed to parse tools list")
    }
    
    pub fn call_tool(&self, tool_name: &str, arguments: serde_json::Value) -> Result<serde_json::Value> {
        if !self.initialized {
            return Err(anyhow!("Client not initialized. Call initialize() first"));
        }
        
        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: self.next_request_id(),
            method: "tools/call".to_string(),
            params: Some(serde_json::json!({
                "name": tool_name,
                "arguments": arguments
            })),
        };
        
        let response = self.send_request(&request)?;
        
        if let Some(error) = response.error {
            return Err(anyhow!("Tool call failed: {}", error.message));
        }
        
        response.result.ok_or_else(|| anyhow!("No result in tool call response"))
    }
    
    pub fn query_logs(&self, query: &str, max_results: usize) -> Result<serde_json::Value> {
        let arguments = serde_json::json!({
            "query": query,
            "maxResults": max_results
        });
        
        self.call_tool("query_logs", arguments)
    }
    
    fn send_request(&self, request: &McpRequest) -> Result<McpResponse> {
        // Serialize request to JSON
        let json_request = serde_json::to_string(request)?;
        
        // Write to stdin
        {
            let mut stdin = self.stdin.lock().unwrap();
            writeln!(stdin, "{}", json_request)?;
            stdin.flush()?;
        }
        
        // Read response from stdout, skipping non-JSON lines
        let response_line = {
            let mut stdout = self.stdout.lock().unwrap();
            let mut line = String::new();
            let mut attempts = 0;
            const MAX_ATTEMPTS: usize = 10;
            
            // Try to read a valid JSON line, skipping non-JSON output
            loop {
                line.clear();
                stdout.read_line(&mut line)
                    .context("Failed to read response from MCP server")?;
                
                if line.trim().is_empty() {
                    attempts += 1;
                    if attempts >= MAX_ATTEMPTS {
                        return Err(anyhow!("No response from MCP server after {} attempts", MAX_ATTEMPTS));
                    }
                    continue;
                }
                
                // Try to parse as JSON - if it succeeds, we have our response
                if line.trim().starts_with('{') {
                    break;
                }
                
                // Otherwise, it's probably a log line from the server - skip it
                eprintln!("MCP server output: {}", line.trim());
                attempts += 1;
                if attempts >= MAX_ATTEMPTS {
                    return Err(anyhow!("No valid JSON response from MCP server after {} lines", MAX_ATTEMPTS));
                }
            }
            
            line
        };
        
        // Parse response
        let response: McpResponse = serde_json::from_str(&response_line)
            .context("Failed to parse MCP response")?;
        
        // Verify response ID matches request ID
        if response.id != request.id {
            return Err(anyhow!("Response ID mismatch: expected {}, got {}", request.id, response.id));
        }
        
        Ok(response)
    }
    
    fn send_notification(&self, method: &str, params: Option<serde_json::Value>) -> Result<()> {
        let notification = serde_json::json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params
        });
        
        let json_notification = serde_json::to_string(&notification)?;
        
        let mut stdin = self.stdin.lock().unwrap();
        writeln!(stdin, "{}", json_notification)?;
        stdin.flush()?;
        
        Ok(())
    }
    
    #[allow(dead_code)]
    pub fn is_alive(&mut self) -> bool {
        if let Some(process) = &mut self.process {
            // Try to check if process is still running (non-blocking)
            // This is a simple check - process might be alive but unresponsive
            match process.try_wait() {
                Ok(Some(_)) => false, // Process has exited
                Ok(None) => true,     // Process is still running
                Err(_) => false,      // Error checking, assume dead
            }
        } else {
            false
        }
    }
}

impl Drop for McpClient {
    fn drop(&mut self) {
        if let Some(mut process) = self.process.take() {
            let _ = process.kill();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_serialization() {
        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: 1,
            method: "test".to_string(),
            params: Some(serde_json::json!({"key": "value"})),
        };
        
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"jsonrpc\":\"2.0\""));
        assert!(json.contains("\"method\":\"test\""));
    }
    
    #[test]
    fn test_response_deserialization() {
        let json = r#"{"jsonrpc":"2.0","id":1,"result":{"data":"test"}}"#;
        let response: McpResponse = serde_json::from_str(json).unwrap();
        
        assert_eq!(response.jsonrpc, "2.0");
        assert_eq!(response.id, 1);
        assert!(response.result.is_some());
        assert!(response.error.is_none());
    }
    
    #[test]
    fn test_error_response_deserialization() {
        let json = r#"{"jsonrpc":"2.0","id":1,"error":{"code":-32600,"message":"Invalid request"}}"#;
        let response: McpResponse = serde_json::from_str(json).unwrap();
        
        assert_eq!(response.jsonrpc, "2.0");
        assert_eq!(response.id, 1);
        assert!(response.result.is_none());
        assert!(response.error.is_some());
        
        let error = response.error.unwrap();
        assert_eq!(error.code, -32600);
        assert_eq!(error.message, "Invalid request");
    }
}
