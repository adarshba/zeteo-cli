use anyhow::{Context, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde_json::json;
use std::collections::HashMap;

use super::{LogBackendClient, LogEntry, LogQuery};

pub struct OpenObserveClient {
    url: String,
    username: String,
    password: String,
    organization: String,
    stream: String,
    client: Client,
}

impl OpenObserveClient {
    pub fn new(
        url: String,
        username: String,
        password: String,
        organization: String,
        stream: String,
        verify_ssl: bool,
    ) -> Result<Self> {
        let client = Client::builder()
            .danger_accept_invalid_certs(!verify_ssl)
            .build()
            .context("Failed to create HTTP client")?;

        Ok(OpenObserveClient {
            url,
            username,
            password,
            organization,
            stream,
            client,
        })
    }

    fn build_sql_query(&self, query: &LogQuery) -> String {
        let mut conditions = vec![];

        // Add query text filter
        if !query.query.is_empty() && query.query != "*" {
            // OpenObserve supports SQL WHERE clauses
            conditions.push(format!("({} LIKE '%{}%')", "log", query.query));
        }

        // Add level filter
        if let Some(level) = &query.level {
            conditions.push(format!("level = '{}'", level));
        }

        // Add service filter
        if let Some(service) = &query.service {
            conditions.push(format!("service_name = '{}'", service));
        }

        // Build time range condition
        if query.start_time.is_some() || query.end_time.is_some() {
            if let Some(start) = &query.start_time {
                conditions.push(format!("_timestamp >= {}", Self::parse_timestamp(start)));
            }
            if let Some(end) = &query.end_time {
                conditions.push(format!("_timestamp <= {}", Self::parse_timestamp(end)));
            }
        }

        let where_clause = if conditions.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", conditions.join(" AND "))
        };

        format!(
            "SELECT * FROM \"{}\" {} ORDER BY _timestamp DESC LIMIT {}",
            self.stream, where_clause, query.max_results
        )
    }

    fn parse_timestamp(timestamp: &str) -> i64 {
        // Try to parse ISO 8601 timestamp and convert to microseconds
        // This is a simplified implementation
        chrono::DateTime::parse_from_rfc3339(timestamp)
            .map(|dt| dt.timestamp_micros())
            .unwrap_or(0)
    }

    fn parse_log_entry(&self, record: &serde_json::Value) -> Option<LogEntry> {
        Some(LogEntry {
            timestamp: record
                .get("_timestamp")
                .or_else(|| record.get("timestamp"))
                .and_then(|v| v.as_i64())
                .map(|ts| {
                    chrono::DateTime::from_timestamp_micros(ts)
                        .map(|dt| dt.to_rfc3339())
                        .unwrap_or_default()
                })
                .unwrap_or_default(),
            level: record
                .get("level")
                .and_then(|v| v.as_str())
                .unwrap_or("INFO")
                .to_uppercase(),
            message: record
                .get("log")
                .or_else(|| record.get("message"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            service: record
                .get("service_name")
                .or_else(|| record.get("service"))
                .and_then(|v| v.as_str())
                .map(String::from),
            trace_id: record
                .get("trace_id")
                .and_then(|v| v.as_str())
                .map(String::from),
            labels: HashMap::new(),
        })
    }
}

#[async_trait]
impl LogBackendClient for OpenObserveClient {
    async fn query_logs(&self, query: &LogQuery) -> Result<Vec<LogEntry>> {
        let search_url = format!(
            "{}/api/{}/{}/_search",
            self.url, self.organization, self.stream
        );

        let sql_query = self.build_sql_query(query);
        let body = json!({
            "query": {
                "sql": sql_query
            }
        });

        let response = self
            .client
            .post(&search_url)
            .basic_auth(&self.username, Some(&self.password))
            .json(&body)
            .send()
            .await
            .context("Failed to send query to OpenObserve")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!(
                "OpenObserve query failed with status {}: {}",
                status,
                error_text
            ));
        }

        let result: serde_json::Value = response
            .json()
            .await
            .context("Failed to parse OpenObserve response")?;

        let hits = result
            .get("hits")
            .and_then(|h| h.as_array())
            .context("Invalid OpenObserve response format")?;

        Ok(hits.iter().filter_map(|hit| self.parse_log_entry(hit)).collect())
    }

    async fn health_check(&self) -> Result<bool> {
        let health_url = format!("{}/healthz", self.url);
        let response = self.client.get(&health_url).send().await;
        Ok(response.is_ok() && response.unwrap().status().is_success())
    }

    fn backend_name(&self) -> &str {
        "OpenObserve"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_sql_query_simple() {
        let client = OpenObserveClient::new(
            "http://localhost:5080".to_string(),
            "admin".to_string(),
            "pass".to_string(),
            "default".to_string(),
            "logs".to_string(),
            false,
        )
        .unwrap();

        let query = LogQuery {
            query: "error".to_string(),
            max_results: 10,
            start_time: None,
            end_time: None,
            level: None,
            service: None,
        };

        let sql = client.build_sql_query(&query);
        assert!(sql.contains("SELECT * FROM"));
        assert!(sql.contains("error"));
        assert!(sql.contains("LIMIT 10"));
    }

    #[test]
    fn test_build_sql_query_with_filters() {
        let client = OpenObserveClient::new(
            "http://localhost:5080".to_string(),
            "admin".to_string(),
            "pass".to_string(),
            "default".to_string(),
            "logs".to_string(),
            false,
        )
        .unwrap();

        let query = LogQuery {
            query: "*".to_string(),
            max_results: 50,
            start_time: None,
            end_time: None,
            level: Some("ERROR".to_string()),
            service: Some("api-service".to_string()),
        };

        let sql = client.build_sql_query(&query);
        assert!(sql.contains("level = 'ERROR'"));
        assert!(sql.contains("service_name = 'api-service'"));
        assert!(sql.contains("LIMIT 50"));
    }
}
