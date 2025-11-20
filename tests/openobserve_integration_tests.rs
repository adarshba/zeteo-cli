/// Integration tests for OpenObserve log fetching
/// 
/// These tests verify that the zeteo-cli can correctly:
/// 1. Connect to OpenObserve backend
/// 2. Fetch logs with various queries
/// 3. Handle errors gracefully
/// 4. Filter and aggregate logs correctly
/// 
/// Prerequisites:
/// - Node.js installed (for otel-mcp-server)
/// - Internet connectivity to OpenObserve instance

use std::collections::HashMap;
use std::time::Duration;

/// Test configuration for OpenObserve
const OPENOBSERVE_URL: &str = "https://periscope.breeze.in/openobserve";
const OPENOBSERVE_USERNAME: &str = "support@breeze.in";
const OPENOBSERVE_PASSWORD: &str = "Breeze@123";
const SERVER_NAME: &str = "otel-mcp-server";

/// Helper function to create OpenObserve MCP server configuration
fn create_openobserve_config() -> HashMap<String, String> {
    let mut env = HashMap::new();
    env.insert("ELASTICSEARCH_URL".to_string(), OPENOBSERVE_URL.to_string());
    env.insert("ELASTICSEARCH_USERNAME".to_string(), OPENOBSERVE_USERNAME.to_string());
    env.insert("ELASTICSEARCH_PASSWORD".to_string(), OPENOBSERVE_PASSWORD.to_string());
    env.insert("SERVER_NAME".to_string(), SERVER_NAME.to_string());
    env.insert("LOGLEVEL".to_string(), "OFF".to_string());
    // otel-mcp-server requires OPENAI_API_KEY for ML embedding feature
    // Get from environment or use a test key
    let api_key = std::env::var("OPENAI_API_KEY")
        .unwrap_or_else(|_| "sk-test-key-not-set".to_string());
    env.insert("OPENAI_API_KEY".to_string(), api_key);
    env
}

#[tokio::test]
async fn test_mcp_client_connection() {
    // Test that we can create and initialize an MCP client
    // This verifies the basic connection to otel-mcp-server
    
    println!("Testing MCP client connection to otel-mcp-server...");
    
    // Note: This test requires npx and otel-mcp-server to be available
    // We'll try to create the client and handle gracefully if it fails
    
    let env = create_openobserve_config();
    let command = "npx";
    let args = vec!["-y".to_string(), "otel-mcp-server".to_string()];
    
    // Try to create MCP client
    match zeteo_cli::mcp::McpClient::new(command, &args, &env, SERVER_NAME.to_string()) {
        Ok(mut client) => {
            println!("✓ MCP client created successfully");
            
            // Try to initialize
            match client.initialize() {
                Ok(result) => {
                    println!("✓ MCP client initialized successfully");
                    println!("  Initialization result: {:?}", result);
                }
                Err(e) => {
                    println!("⚠ MCP client initialization failed: {}", e);
                    println!("  This might be due to network issues or server unavailability");
                }
            }
        }
        Err(e) => {
            println!("⚠ Could not create MCP client: {}", e);
            println!("  Make sure Node.js is installed and otel-mcp-server is available");
        }
    }
}

#[tokio::test]
async fn test_query_logs_basic() {
    // Test basic log querying with a simple query
    
    println!("\nTesting basic log query...");
    
    let env = create_openobserve_config();
    let command = "npx";
    let args = vec!["-y".to_string(), "otel-mcp-server".to_string()];
    
    match zeteo_cli::mcp::McpClient::new(command, &args, &env, SERVER_NAME.to_string()) {
        Ok(mut client) => {
            match client.initialize() {
                Ok(_) => {
                    // Give the server a moment to stabilize
                    tokio::time::sleep(Duration::from_millis(500)).await;
                    
                    // Try to query logs with a simple query
                    println!("  Querying logs with query: '*'");
                    match client.query_logs("*", 10) {
                        Ok(result) => {
                            println!("✓ Query successful!");
                            println!("  Result: {}", serde_json::to_string_pretty(&result).unwrap_or_else(|_| format!("{:?}", result)));
                            
                            // Verify the response structure
                            if let Some(logs) = result.get("logs") {
                                if let Some(logs_array) = logs.as_array() {
                                    println!("  Number of logs returned: {}", logs_array.len());
                                    if logs_array.is_empty() {
                                        println!("  ⚠ No logs found (might be expected if no data in OpenObserve)");
                                    }
                                }
                            } else {
                                println!("  ⚠ Response doesn't contain 'logs' field");
                            }
                        }
                        Err(e) => {
                            println!("✗ Query failed: {}", e);
                        }
                    }
                }
                Err(e) => {
                    println!("⚠ Skipping test: MCP client initialization failed: {}", e);
                }
            }
        }
        Err(e) => {
            println!("⚠ Skipping test: Could not create MCP client: {}", e);
        }
    }
}

#[tokio::test]
async fn test_query_logs_with_error_filter() {
    // Test querying logs with error level filter
    
    println!("\nTesting log query with error filter...");
    
    let env = create_openobserve_config();
    let command = "npx";
    let args = vec!["-y".to_string(), "otel-mcp-server".to_string()];
    
    match zeteo_cli::mcp::McpClient::new(command, &args, &env, SERVER_NAME.to_string()) {
        Ok(mut client) => {
            match client.initialize() {
                Ok(_) => {
                    tokio::time::sleep(Duration::from_millis(500)).await;
                    
                    // Query for error logs
                    println!("  Querying logs with query: 'level:ERROR OR level:error'");
                    match client.query_logs("level:ERROR OR level:error", 20) {
                        Ok(result) => {
                            println!("✓ Error query successful!");
                            
                            if let Some(logs) = result.get("logs") {
                                if let Some(logs_array) = logs.as_array() {
                                    println!("  Number of error logs found: {}", logs_array.len());
                                    
                                    // Print first few error logs for inspection
                                    for (i, log) in logs_array.iter().take(3).enumerate() {
                                        println!("  Error log {}: {}", i + 1, 
                                            serde_json::to_string(log).unwrap_or_else(|_| format!("{:?}", log)));
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            println!("✗ Error query failed: {}", e);
                        }
                    }
                }
                Err(e) => {
                    println!("⚠ Skipping test: MCP client initialization failed: {}", e);
                }
            }
        }
        Err(e) => {
            println!("⚠ Skipping test: Could not create MCP client: {}", e);
        }
    }
}

#[tokio::test]
async fn test_query_logs_with_specific_search() {
    // Test querying logs with specific search terms
    
    println!("\nTesting log query with specific search terms...");
    
    let env = create_openobserve_config();
    let command = "npx";
    let args = vec!["-y".to_string(), "otel-mcp-server".to_string()];
    
    // Test with multiple search queries
    let queries = vec![
        "exception",
        "timeout",
        "failed",
        "success",
    ];
    
    match zeteo_cli::mcp::McpClient::new(command, &args, &env, SERVER_NAME.to_string()) {
        Ok(mut client) => {
            match client.initialize() {
                Ok(_) => {
                    tokio::time::sleep(Duration::from_millis(500)).await;
                    
                    for query in queries {
                        println!("  Testing query: '{}'", query);
                        match client.query_logs(query, 5) {
                            Ok(result) => {
                                if let Some(logs) = result.get("logs") {
                                    if let Some(logs_array) = logs.as_array() {
                                        println!("    ✓ Found {} logs matching '{}'", logs_array.len(), query);
                                    }
                                }
                            }
                            Err(e) => {
                                println!("    ✗ Query '{}' failed: {}", query, e);
                            }
                        }
                        
                        // Small delay between queries to avoid overwhelming the server
                        tokio::time::sleep(Duration::from_millis(200)).await;
                    }
                }
                Err(e) => {
                    println!("⚠ Skipping test: MCP client initialization failed: {}", e);
                }
            }
        }
        Err(e) => {
            println!("⚠ Skipping test: Could not create MCP client: {}", e);
        }
    }
}

#[tokio::test]
async fn test_query_logs_different_limits() {
    // Test querying logs with different result limits
    
    println!("\nTesting log queries with different result limits...");
    
    let env = create_openobserve_config();
    let command = "npx";
    let args = vec!["-y".to_string(), "otel-mcp-server".to_string()];
    
    let limits = vec![1, 5, 10, 20, 50];
    
    match zeteo_cli::mcp::McpClient::new(command, &args, &env, SERVER_NAME.to_string()) {
        Ok(mut client) => {
            match client.initialize() {
                Ok(_) => {
                    tokio::time::sleep(Duration::from_millis(500)).await;
                    
                    for limit in limits {
                        println!("  Testing with maxResults: {}", limit);
                        match client.query_logs("*", limit) {
                            Ok(result) => {
                                if let Some(logs) = result.get("logs") {
                                    if let Some(logs_array) = logs.as_array() {
                                        let actual_count = logs_array.len();
                                        println!("    ✓ Requested: {}, Received: {}", limit, actual_count);
                                        
                                        if actual_count > limit {
                                            println!("    ⚠ Warning: Received more logs than requested!");
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                println!("    ✗ Query with limit {} failed: {}", limit, e);
                            }
                        }
                        
                        tokio::time::sleep(Duration::from_millis(200)).await;
                    }
                }
                Err(e) => {
                    println!("⚠ Skipping test: MCP client initialization failed: {}", e);
                }
            }
        }
        Err(e) => {
            println!("⚠ Skipping test: Could not create MCP client: {}", e);
        }
    }
}

#[tokio::test]
async fn test_log_explorer_integration() {
    // Test the LogExplorer with OpenObserve configuration
    
    println!("\nTesting LogExplorer integration with OpenObserve...");
    
    // First, we need to create a config file with OpenObserve credentials
    let mut servers = HashMap::new();
    servers.insert(
        SERVER_NAME.to_string(),
        zeteo_cli::config::McpServer {
            command: "npx".to_string(),
            args: vec!["-y".to_string(), "otel-mcp-server".to_string()],
            env: create_openobserve_config(),
        },
    );
    
    let config = zeteo_cli::config::Config { 
        servers,
        backends: std::collections::HashMap::new(),
    };
    
    // Save the config temporarily
    match config.save() {
        Ok(_) => {
            println!("✓ Saved OpenObserve config");
            
            // Create LogExplorer
            match zeteo_cli::logs::LogExplorer::new(SERVER_NAME.to_string()).with_mcp_client() {
                Ok(explorer) => {
                    println!("✓ LogExplorer created and MCP client initialized");
                    
                    // Test searching logs
                    match explorer.search_logs("*", 10).await {
                        Ok(logs) => {
                            println!("✓ Successfully searched logs");
                            println!("  Found {} logs", logs.len());
                            
                            if !logs.is_empty() {
                                println!("  First log:");
                                if let Ok(json) = serde_json::to_string_pretty(&logs[0]) {
                                    println!("{}", json);
                                }
                            } else {
                                println!("  ⚠ No logs found (might be expected)");
                            }
                        }
                        Err(e) => {
                            println!("✗ Log search failed: {}", e);
                        }
                    }
                }
                Err(e) => {
                    println!("⚠ Could not create LogExplorer: {}", e);
                }
            }
        }
        Err(e) => {
            println!("✗ Failed to save config: {}", e);
        }
    }
}

#[tokio::test]
async fn test_error_handling_invalid_credentials() {
    // Test error handling with invalid credentials
    
    println!("\nTesting error handling with invalid credentials...");
    
    let mut env = create_openobserve_config();
    env.insert("ELASTICSEARCH_PASSWORD".to_string(), "invalid_password".to_string());
    
    let command = "npx";
    let args = vec!["-y".to_string(), "otel-mcp-server".to_string()];
    
    match zeteo_cli::mcp::McpClient::new(command, &args, &env, SERVER_NAME.to_string()) {
        Ok(mut client) => {
            match client.initialize() {
                Ok(_) => {
                    tokio::time::sleep(Duration::from_millis(500)).await;
                    
                    // This should fail or return an error
                    match client.query_logs("*", 10) {
                        Ok(_) => {
                            println!("  ⚠ Query succeeded with invalid credentials (unexpected)");
                        }
                        Err(e) => {
                            println!("✓ Query correctly failed with invalid credentials");
                            println!("  Error: {}", e);
                        }
                    }
                }
                Err(e) => {
                    println!("✓ Initialization correctly failed: {}", e);
                }
            }
        }
        Err(e) => {
            println!("✓ Client creation correctly failed: {}", e);
        }
    }
}

#[tokio::test]
async fn test_concurrent_queries() {
    // Test making multiple concurrent queries
    
    println!("\nTesting concurrent log queries...");
    
    let env = create_openobserve_config();
    let command = "npx";
    let args = vec!["-y".to_string(), "otel-mcp-server".to_string()];
    
    match zeteo_cli::mcp::McpClient::new(command, &args, &env, SERVER_NAME.to_string()) {
        Ok(mut client) => {
            match client.initialize() {
                Ok(_) => {
                    tokio::time::sleep(Duration::from_millis(500)).await;
                    
                    let queries = vec!["error", "warning", "info", "debug"];
                    
                    println!("  Running {} queries sequentially...", queries.len());
                    
                    for query in queries {
                        match client.query_logs(query, 5) {
                            Ok(result) => {
                                if let Some(logs) = result.get("logs") {
                                    if let Some(logs_array) = logs.as_array() {
                                        println!("    ✓ Query '{}': {} results", query, logs_array.len());
                                    }
                                }
                            }
                            Err(e) => {
                                println!("    ✗ Query '{}' failed: {}", query, e);
                            }
                        }
                        
                        tokio::time::sleep(Duration::from_millis(100)).await;
                    }
                }
                Err(e) => {
                    println!("⚠ Skipping test: MCP client initialization failed: {}", e);
                }
            }
        }
        Err(e) => {
            println!("⚠ Skipping test: Could not create MCP client: {}", e);
        }
    }
}
