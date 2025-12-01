pub mod elasticsearch;
pub mod kibana;
pub mod openobserve;

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogQuery {
    pub query: String,
    pub max_results: usize,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub level: Option<String>,
    pub service: Option<String>,
    #[serde(default)]
    pub index_pattern: Option<String>,
}

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

#[async_trait]
pub trait LogBackendClient: Send + Sync {
    async fn query_logs(&self, query: &LogQuery) -> Result<Vec<LogEntry>>;
    #[allow(dead_code)]
    async fn health_check(&self) -> Result<bool>;
    #[allow(dead_code)]
    fn backend_name(&self) -> &str;
}
