use anyhow::Result;
use colored::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::config::Config;
use crate::mcp::McpClient;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub message: String,
    pub service: Option<String>,
    pub trace_id: Option<String>,
    #[serde(default)]
    pub labels: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct LogFilter {
    pub level: Option<String>,
    pub service: Option<String>,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub contains: Option<String>,
}

impl Default for LogFilter {
    fn default() -> Self {
        Self {
            level: None,
            service: None,
            start_time: None,
            end_time: None,
            contains: None,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct LogAggregation {
    pub total_count: usize,
    pub level_counts: HashMap<String, usize>,
    pub service_counts: HashMap<String, usize>,
    pub time_range: Option<(String, String)>,
}

pub struct LogExplorer {
    mcp_server: String,
    mcp_client: Option<McpClient>,
}

impl LogExplorer {
    pub fn new(mcp_server: String) -> Self {
        LogExplorer { 
            mcp_server,
            mcp_client: None,
        }
    }
    
    /// Initialize the MCP client connection
    pub fn with_mcp_client(mut self) -> Result<Self> {
        let config = Config::load()?;
        
        if let Some(server_config) = config.servers.get(&self.mcp_server) {
            let mut client = McpClient::new(
                &server_config.command,
                &server_config.args,
                &server_config.env,
                self.mcp_server.clone(),
            )?;
            
            // Initialize the MCP client
            client.initialize()?;
            
            self.mcp_client = Some(client);
        }
        
        Ok(self)
    }
    
    pub async fn search_logs(&self, query: &str, max_results: usize) -> Result<Vec<LogEntry>> {
        // Try to use MCP client if available
        if let Some(client) = &self.mcp_client {
            match client.query_logs(query, max_results) {
                Ok(result) => {
                    // Parse the result into LogEntry structs
                    if let Some(logs_array) = result.get("logs").and_then(|v| v.as_array()) {
                        let logs: Vec<LogEntry> = logs_array
                            .iter()
                            .filter_map(|log| serde_json::from_value(log.clone()).ok())
                            .collect();
                        return Ok(logs);
                    }
                }
                Err(e) => {
                    eprintln!("{} {}", "⚠ MCP query failed:".yellow(), e);
                }
            }
        }
        
        // Fallback: return placeholder data
        println!("Searching logs with query: {}", query.cyan());
        println!("MCP Server: {}", self.mcp_server.green());
        println!("Max results: {}", max_results);
        
        Ok(vec![])
    }

    pub async fn search_logs_with_filter(&self, query: &str, max_results: usize, filter: &LogFilter) -> Result<Vec<LogEntry>> {
        let mut logs = self.search_logs(query, max_results).await?;
        
        // Apply filters
        logs.retain(|log| {
            if let Some(level) = &filter.level {
                if !log.level.eq_ignore_ascii_case(level) {
                    return false;
                }
            }
            
            if let Some(service) = &filter.service {
                if log.service.as_ref() != Some(service) {
                    return false;
                }
            }
            
            if let Some(contains) = &filter.contains {
                if !log.message.to_lowercase().contains(&contains.to_lowercase()) {
                    return false;
                }
            }
            
            true
        });
        
        Ok(logs)
    }

    pub async fn stream_logs<F>(&self, query: &str, callback: F) -> Result<()>
    where
        F: Fn(&LogEntry) -> bool,
    {
        // This would use MCP to stream logs in real-time
        // For now, simulate streaming with batch fetching
        println!("{}", "Starting log stream...".cyan());
        println!("Query: {}", query.green());
        println!("Press Ctrl+C to stop streaming");
        
        let mut _offset = 0;
        let batch_size = 10;
        
        loop {
            let logs = self.search_logs(query, batch_size).await?;
            
            if logs.is_empty() {
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                continue;
            }
            
            for log in &logs {
                if !callback(log) {
                    return Ok(());
                }
                self.display_single_log(log);
            }
            
            _offset += logs.len();
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        }
    }

    pub fn aggregate_logs(&self, logs: &[LogEntry]) -> LogAggregation {
        let mut level_counts = HashMap::new();
        let mut service_counts = HashMap::new();
        let mut timestamps = Vec::new();
        
        for log in logs {
            *level_counts.entry(log.level.clone()).or_insert(0) += 1;
            
            if let Some(service) = &log.service {
                *service_counts.entry(service.clone()).or_insert(0) += 1;
            }
            
            timestamps.push(log.timestamp.clone());
        }
        
        let time_range = if !timestamps.is_empty() {
            timestamps.sort();
            Some((timestamps.first().unwrap().clone(), timestamps.last().unwrap().clone()))
        } else {
            None
        };
        
        LogAggregation {
            total_count: logs.len(),
            level_counts,
            service_counts,
            time_range,
        }
    }

    pub fn export_logs_json(&self, logs: &[LogEntry], filename: &str) -> Result<()> {
        use std::fs::File;
        use std::io::Write;
        
        let json = serde_json::to_string_pretty(logs)?;
        let mut file = File::create(filename)?;
        file.write_all(json.as_bytes())?;
        
        println!("{}", format!("Exported {} logs to {}", logs.len(), filename).green());
        Ok(())
    }

    pub fn export_logs_csv(&self, logs: &[LogEntry], filename: &str) -> Result<()> {
        use std::fs::File;
        use std::io::Write;
        
        let mut file = File::create(filename)?;
        
        // Write header
        writeln!(file, "timestamp,level,message,service,trace_id")?;
        
        // Write data
        for log in logs {
            let service = log.service.as_deref().unwrap_or("");
            let trace_id = log.trace_id.as_deref().unwrap_or("");
            let message = log.message.replace(",", ";").replace("\n", " ");
            
            writeln!(file, "{},{},{},{},{}",
                log.timestamp,
                log.level,
                message,
                service,
                trace_id
            )?;
        }
        
        println!("{}", format!("Exported {} logs to {}", logs.len(), filename).green());
        Ok(())
    }

    fn display_single_log(&self, log: &LogEntry) {
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
    
    pub fn display_logs(&self, logs: &[LogEntry]) {
        if logs.is_empty() {
            println!("{}", "No logs found.".yellow());
            return;
        }
        
        for log in logs {
            self.display_single_log(log);
        }
    }

    pub fn display_aggregation(&self, agg: &LogAggregation) {
        println!("\n{}", "=== Log Aggregation ===".cyan().bold());
        println!("Total logs: {}", agg.total_count.to_string().green().bold());
        
        if !agg.level_counts.is_empty() {
            println!("\n{}", "By Level:".bold());
            let mut levels: Vec<_> = agg.level_counts.iter().collect();
            levels.sort_by(|a, b| b.1.cmp(a.1));
            for (level, count) in levels {
                let level_colored = match level.as_str() {
                    "ERROR" | "error" => level.red().bold(),
                    "WARN" | "warn" => level.yellow().bold(),
                    "INFO" | "info" => level.green().bold(),
                    "DEBUG" | "debug" => level.blue().bold(),
                    _ => level.normal(),
                };
                println!("  {}: {}", level_colored, count);
            }
        }
        
        if !agg.service_counts.is_empty() {
            println!("\n{}", "By Service:".bold());
            let mut services: Vec<_> = agg.service_counts.iter().collect();
            services.sort_by(|a, b| b.1.cmp(a.1));
            for (service, count) in services {
                println!("  {}: {}", service.cyan(), count);
            }
        }
        
        if let Some((start, end)) = &agg.time_range {
            println!("\n{}", "Time Range:".bold());
            println!("  From: {}", start.dimmed());
            println!("  To:   {}", end.dimmed());
        }
        println!();
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
            labels: std::collections::HashMap::new(),
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

