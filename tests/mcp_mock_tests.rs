/// Mock tests for MCP client functionality
/// 
/// These tests verify the MCP client implementation without requiring
/// an actual otel-mcp-server to be running. They test:
/// - Serialization/deserialization of MCP messages
/// - Request/response correlation
/// - Error handling
/// - Protocol compliance

use serde_json::json;

#[test]
fn test_mcp_request_serialization() {
    let request = zeteo::mcp::McpRequest {
        jsonrpc: "2.0".to_string(),
        id: 1,
        method: "initialize".to_string(),
        params: Some(json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {"tools": {}},
            "clientInfo": {"name": "zeteo-cli", "version": "0.1.0"}
        })),
    };
    
    let serialized = serde_json::to_string(&request).unwrap();
    assert!(serialized.contains("\"jsonrpc\":\"2.0\""));
    assert!(serialized.contains("\"method\":\"initialize\""));
    assert!(serialized.contains("\"id\":1"));
}

#[test]
fn test_mcp_response_deserialization_success() {
    let json = r#"{
        "jsonrpc": "2.0",
        "id": 1,
        "result": {
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "tools": {}
            },
            "serverInfo": {
                "name": "otel-mcp-server",
                "version": "0.1.0"
            }
        }
    }"#;
    
    let response: zeteo::mcp::McpResponse = serde_json::from_str(json).unwrap();
    assert_eq!(response.jsonrpc, "2.0");
    assert_eq!(response.id, 1);
    assert!(response.result.is_some());
    assert!(response.error.is_none());
    
    let result = response.result.unwrap();
    assert_eq!(result["protocolVersion"], "2024-11-05");
}

#[test]
fn test_mcp_response_deserialization_error() {
    let json = r#"{
        "jsonrpc": "2.0",
        "id": 1,
        "error": {
            "code": -32600,
            "message": "Invalid request"
        }
    }"#;
    
    let response: zeteo::mcp::McpResponse = serde_json::from_str(json).unwrap();
    assert_eq!(response.jsonrpc, "2.0");
    assert_eq!(response.id, 1);
    assert!(response.result.is_none());
    assert!(response.error.is_some());
    
    let error = response.error.unwrap();
    assert_eq!(error.code, -32600);
    assert_eq!(error.message, "Invalid request");
}

#[test]
fn test_tool_call_request_format() {
    let request = zeteo::mcp::McpRequest {
        jsonrpc: "2.0".to_string(),
        id: 2,
        method: "tools/call".to_string(),
        params: Some(json!({
            "name": "query_logs",
            "arguments": {
                "query": "error",
                "maxResults": 10
            }
        })),
    };
    
    let serialized = serde_json::to_string(&request).unwrap();
    assert!(serialized.contains("\"method\":\"tools/call\""));
    assert!(serialized.contains("\"name\":\"query_logs\""));
    assert!(serialized.contains("\"query\":\"error\""));
    assert!(serialized.contains("\"maxResults\":10"));
}

#[test]
fn test_log_entry_parsing() {
    let log_json = json!({
        "timestamp": "2024-01-01T00:00:00Z",
        "level": "ERROR",
        "message": "Test error message",
        "service": "test-service",
        "trace_id": "abc123",
        "labels": {
            "environment": "production",
            "region": "us-east-1"
        }
    });
    
    let log: zeteo::logs::LogEntry = serde_json::from_value(log_json).unwrap();
    assert_eq!(log.timestamp, "2024-01-01T00:00:00Z");
    assert_eq!(log.level, "ERROR");
    assert_eq!(log.message, "Test error message");
    assert_eq!(log.service, Some("test-service".to_string()));
    assert_eq!(log.trace_id, Some("abc123".to_string()));
    assert_eq!(log.labels.get("environment").unwrap(), "production");
}

#[test]
fn test_log_aggregation() {
    let logs = vec![
        zeteo::logs::LogEntry {
            timestamp: "2024-01-01T00:00:00Z".to_string(),
            level: "ERROR".to_string(),
            message: "Error 1".to_string(),
            service: Some("api".to_string()),
            trace_id: None,
            labels: std::collections::HashMap::new(),
        },
        zeteo::logs::LogEntry {
            timestamp: "2024-01-01T00:01:00Z".to_string(),
            level: "ERROR".to_string(),
            message: "Error 2".to_string(),
            service: Some("api".to_string()),
            trace_id: None,
            labels: std::collections::HashMap::new(),
        },
        zeteo::logs::LogEntry {
            timestamp: "2024-01-01T00:02:00Z".to_string(),
            level: "WARN".to_string(),
            message: "Warning 1".to_string(),
            service: Some("worker".to_string()),
            trace_id: None,
            labels: std::collections::HashMap::new(),
        },
    ];
    
    let explorer = zeteo::logs::LogExplorer::new("test-server".to_string());
    let agg = explorer.aggregate_logs(&logs);
    
    assert_eq!(agg.total_count, 3);
    assert_eq!(*agg.level_counts.get("ERROR").unwrap(), 2);
    assert_eq!(*agg.level_counts.get("WARN").unwrap(), 1);
    assert_eq!(*agg.service_counts.get("api").unwrap(), 2);
    assert_eq!(*agg.service_counts.get("worker").unwrap(), 1);
    assert!(agg.time_range.is_some());
}

#[test]
fn test_config_serialization() {
    let mut servers = std::collections::HashMap::new();
    servers.insert(
        "test-server".to_string(),
        zeteo::config::McpServer {
            command: "npx".to_string(),
            args: vec!["-y".to_string(), "test-server".to_string()],
            env: {
                let mut env = std::collections::HashMap::new();
                env.insert("TEST_VAR".to_string(), "test_value".to_string());
                env
            },
        },
    );
    
    let config = zeteo::config::Config { 
        servers,
        backends: std::collections::HashMap::new(),
    };
    let json = serde_json::to_string(&config).unwrap();
    
    assert!(json.contains("\"test-server\""));
    assert!(json.contains("\"command\":\"npx\""));
    assert!(json.contains("\"TEST_VAR\""));
    assert!(json.contains("\"test_value\""));
}

#[tokio::test]
async fn test_log_filter_application() {
    let logs = vec![
        zeteo::logs::LogEntry {
            timestamp: "2024-01-01T00:00:00Z".to_string(),
            level: "ERROR".to_string(),
            message: "Database connection failed".to_string(),
            service: Some("api".to_string()),
            trace_id: None,
            labels: std::collections::HashMap::new(),
        },
        zeteo::logs::LogEntry {
            timestamp: "2024-01-01T00:01:00Z".to_string(),
            level: "INFO".to_string(),
            message: "Request completed successfully".to_string(),
            service: Some("api".to_string()),
            trace_id: None,
            labels: std::collections::HashMap::new(),
        },
        zeteo::logs::LogEntry {
            timestamp: "2024-01-01T00:02:00Z".to_string(),
            level: "ERROR".to_string(),
            message: "Timeout occurred".to_string(),
            service: Some("worker".to_string()),
            trace_id: None,
            labels: std::collections::HashMap::new(),
        },
    ];
    
    // Test level filter
    let filtered: Vec<_> = logs.iter()
        .filter(|log| log.level == "ERROR")
        .collect();
    assert_eq!(filtered.len(), 2);
    
    // Test service filter
    let filtered: Vec<_> = logs.iter()
        .filter(|log| log.service.as_deref() == Some("api"))
        .collect();
    assert_eq!(filtered.len(), 2);
    
    // Test message contains
    let filtered: Vec<_> = logs.iter()
        .filter(|log| log.message.to_lowercase().contains("failed"))
        .collect();
    assert_eq!(filtered.len(), 1);
}
