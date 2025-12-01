use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use chrono::{Utc, Duration};

use crate::backends::{LogBackendClient, LogQuery};

/// Tool execution request parsed from AI function call
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryLogsArgs {
    pub query: String,
    #[serde(default = "default_max_results")]
    pub max_results: usize,
    pub level: Option<String>,
    pub service: Option<String>,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
}

fn default_max_results() -> usize {
    50
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogStatsArgs {
    pub start_time: Option<String>,
    pub end_time: Option<String>,
}

/// Result of log query for AI consumption
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogQueryResult {
    pub total_count: usize,
    pub logs: Vec<LogEntrySummary>,
    pub level_distribution: std::collections::HashMap<String, usize>,
    pub services: Vec<String>,
    pub time_range: Option<TimeRange>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntrySummary {
    pub timestamp: String,
    pub level: String,
    pub message: String,
    pub service: Option<String>,
    pub trace_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeRange {
    pub start: String,
    pub end: String,
}

/// Tool executor that handles log-related function calls
pub struct ToolExecutor {
    backend: Arc<dyn LogBackendClient>,
}

impl ToolExecutor {
    pub fn new(backend: Arc<dyn LogBackendClient>) -> Self {
        Self { backend }
    }
    
    /// Execute a tool call and return the result as a JSON string
    pub async fn execute(&self, tool_name: &str, arguments: &str) -> Result<String> {
        match tool_name {
            "query_logs" => {
                let args: QueryLogsArgs = serde_json::from_str(arguments)
                    .context("Failed to parse query_logs arguments")?;
                let result = self.query_logs(args).await?;
                serde_json::to_string_pretty(&result)
                    .context("Failed to serialize query result")
            }
            "list_services" => {
                let result = self.list_services().await?;
                serde_json::to_string_pretty(&result)
                    .context("Failed to serialize services list")
            }
            "get_log_stats" => {
                let args: LogStatsArgs = serde_json::from_str(arguments)
                    .context("Failed to parse get_log_stats arguments")?;
                let result = self.get_log_stats(args).await?;
                serde_json::to_string_pretty(&result)
                    .context("Failed to serialize stats")
            }
            _ => anyhow::bail!("Unknown tool: {}", tool_name),
        }
    }
    
    /// Parse relative time strings like "1h", "30m", "2d" into ISO timestamps
    fn parse_time(&self, time_str: &str) -> Option<String> {
        let time_str = time_str.trim();
        
        // Check if it's already an ISO timestamp
        if time_str.contains('T') || time_str.contains('-') {
            return Some(time_str.to_string());
        }
        
        // Parse relative time
        let now = Utc::now();
        let duration = if time_str.ends_with('h') {
            let hours: i64 = time_str.trim_end_matches('h').parse().ok()?;
            Duration::hours(hours)
        } else if time_str.ends_with('m') {
            let minutes: i64 = time_str.trim_end_matches('m').parse().ok()?;
            Duration::minutes(minutes)
        } else if time_str.ends_with('d') {
            let days: i64 = time_str.trim_end_matches('d').parse().ok()?;
            Duration::days(days)
        } else {
            return None;
        };
        
        Some((now - duration).to_rfc3339())
    }
    
    async fn query_logs(&self, args: QueryLogsArgs) -> Result<LogQueryResult> {
        let max_results = args.max_results.min(200);
        
        let start_time = args.start_time.as_ref().and_then(|t| self.parse_time(t));
        let end_time = args.end_time.as_ref().and_then(|t| self.parse_time(t));
        
        let query = LogQuery {
            query: args.query,
            max_results,
            start_time,
            end_time,
            level: args.level,
            service: args.service,
        };
        
        let logs = self.backend.query_logs(&query).await?;
        
        // Build level distribution
        let mut level_distribution = std::collections::HashMap::new();
        let mut services_set = std::collections::HashSet::new();
        let mut timestamps = Vec::new();
        
        for log in &logs {
            *level_distribution.entry(log.level.clone()).or_insert(0) += 1;
            if let Some(svc) = &log.service {
                services_set.insert(svc.clone());
            }
            timestamps.push(log.timestamp.clone());
        }
        
        let time_range = if !timestamps.is_empty() {
            timestamps.sort();
            Some(TimeRange {
                start: timestamps.first().unwrap().clone(),
                end: timestamps.last().unwrap().clone(),
            })
        } else {
            None
        };
        
        let log_summaries: Vec<LogEntrySummary> = logs.iter().map(|log| LogEntrySummary {
            timestamp: log.timestamp.clone(),
            level: log.level.clone(),
            message: truncate_message(&log.message, 500),
            service: log.service.clone(),
            trace_id: log.trace_id.clone(),
        }).collect();
        
        Ok(LogQueryResult {
            total_count: logs.len(),
            logs: log_summaries,
            level_distribution,
            services: services_set.into_iter().collect(),
            time_range,
        })
    }
    
    async fn list_services(&self) -> Result<Vec<String>> {
        // Query recent logs to discover services
        let query = LogQuery {
            query: "*".to_string(),
            max_results: 100,
            start_time: Some((Utc::now() - Duration::hours(1)).to_rfc3339()),
            end_time: None,
            level: None,
            service: None,
        };
        
        let logs = self.backend.query_logs(&query).await?;
        
        let services: std::collections::HashSet<String> = logs
            .iter()
            .filter_map(|log| log.service.clone())
            .collect();
        
        Ok(services.into_iter().collect())
    }
    
    async fn get_log_stats(&self, args: LogStatsArgs) -> Result<serde_json::Value> {
        let start_time = args.start_time
            .as_ref()
            .and_then(|t| self.parse_time(t))
            .unwrap_or_else(|| (Utc::now() - Duration::hours(1)).to_rfc3339());
        
        let query = LogQuery {
            query: "*".to_string(),
            max_results: 200,
            start_time: Some(start_time),
            end_time: args.end_time.as_ref().and_then(|t| self.parse_time(t)),
            level: None,
            service: None,
        };
        
        let logs = self.backend.query_logs(&query).await?;
        
        let mut level_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
        let mut service_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
        
        for log in &logs {
            *level_counts.entry(log.level.clone()).or_insert(0) += 1;
            if let Some(svc) = &log.service {
                *service_counts.entry(svc.clone()).or_insert(0) += 1;
            }
        }
        
        Ok(serde_json::json!({
            "total_logs": logs.len(),
            "level_distribution": level_counts,
            "service_distribution": service_counts,
            "error_count": level_counts.get("ERROR").unwrap_or(&0),
            "warn_count": level_counts.get("WARN").unwrap_or(&0),
        }))
    }
}

fn truncate_message(msg: &str, max_len: usize) -> String {
    if msg.len() <= max_len {
        msg.to_string()
    } else {
        format!("{}...", &msg[..max_len])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_query_logs_args() {
        let json = r#"{"query": "error", "max_results": 100, "level": "ERROR"}"#;
        let args: QueryLogsArgs = serde_json::from_str(json).unwrap();
        assert_eq!(args.query, "error");
        assert_eq!(args.max_results, 100);
        assert_eq!(args.level, Some("ERROR".to_string()));
    }
    
    #[test]
    fn test_default_max_results() {
        let json = r#"{"query": "test"}"#;
        let args: QueryLogsArgs = serde_json::from_str(json).unwrap();
        assert_eq!(args.max_results, 50);
    }
}
