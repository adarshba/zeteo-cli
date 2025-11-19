use anyhow::Result;
use colored::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub message: String,
    pub service: Option<String>,
    pub trace_id: Option<String>,
}

pub struct LogExplorer {
    mcp_server: String,
}

impl LogExplorer {
    pub fn new(mcp_server: String) -> Self {
        LogExplorer { mcp_server }
    }
    
    pub async fn search_logs(&self, query: &str, max_results: usize) -> Result<Vec<LogEntry>> {
        // This would use the MCP client to query logs
        // For now, returning placeholder data
        println!("Searching logs with query: {}", query.cyan());
        println!("MCP Server: {}", self.mcp_server.green());
        println!("Max results: {}", max_results);
        
        Ok(vec![])
    }
    
    pub fn display_logs(&self, logs: &[LogEntry]) {
        if logs.is_empty() {
            println!("{}", "No logs found.".yellow());
            return;
        }
        
        for log in logs {
            let level_colored = match log.level.as_str() {
                "ERROR" | "error" => log.level.red().bold(),
                "WARN" | "warn" => log.level.yellow().bold(),
                "INFO" | "info" => log.level.green().bold(),
                "DEBUG" | "debug" => log.level.blue().bold(),
                _ => log.level.normal(),
            };
            
            println!("[{}] {} {}", 
                log.timestamp.dimmed(),
                level_colored,
                log.message
            );
            
            if let Some(service) = &log.service {
                println!("  Service: {}", service.cyan());
            }
            
            if let Some(trace_id) = &log.trace_id {
                println!("  Trace ID: {}", trace_id.magenta());
            }
        }
    }
    
    pub async fn interactive_mode(&self) -> Result<()> {
        println!("{}", "=== Interactive Log Explorer ===".green().bold());
        println!("Type your search queries or 'quit' to exit\n");
        
        loop {
            let query = dialoguer::Input::<String>::new()
                .with_prompt("Search")
                .interact_text()?;
            
            if query.trim().eq_ignore_ascii_case("quit") || query.trim().eq_ignore_ascii_case("exit") {
                break;
            }
            
            let logs = self.search_logs(&query, 50).await?;
            self.display_logs(&logs);
            println!();
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_explorer_creation() {
        let explorer = LogExplorer::new("test-server".to_string());
        assert_eq!(explorer.mcp_server, "test-server");
    }

    #[test]
    fn test_log_entry_serialization() {
        let log = LogEntry {
            timestamp: "2024-01-01T00:00:00Z".to_string(),
            level: "INFO".to_string(),
            message: "Test message".to_string(),
            service: Some("test-service".to_string()),
            trace_id: Some("abc123".to_string()),
        };

        let json = serde_json::to_string(&log).unwrap();
        let deserialized: LogEntry = serde_json::from_str(&json).unwrap();

        assert_eq!(log.timestamp, deserialized.timestamp);
        assert_eq!(log.level, deserialized.level);
        assert_eq!(log.message, deserialized.message);
    }

    #[tokio::test]
    async fn test_search_logs_returns_empty() {
        let explorer = LogExplorer::new("test-server".to_string());
        let result = explorer.search_logs("test query", 10).await;
        assert!(result.is_ok());
        let logs = result.unwrap();
        assert_eq!(logs.len(), 0);
    }
}

