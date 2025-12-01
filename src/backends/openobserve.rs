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

        if !query.query.is_empty() && query.query != "*" {
            conditions.push(format!(
                "(body LIKE '%{}%' OR payload LIKE '%{}%' OR service_name LIKE '%{}%')",
                query.query, query.query, query.query
            ));
        }

        if let Some(level) = &query.level {
            let severity_condition = match level.to_uppercase().as_str() {
                "ERROR" | "ERR" => "severity >= 17 AND severity <= 20",
                "WARN" | "WARNING" => "severity >= 13 AND severity <= 16",
                "INFO" => "severity >= 9 AND severity <= 12",
                "DEBUG" => "severity >= 5 AND severity <= 8",
                "TRACE" => "severity >= 1 AND severity <= 4",
                "FATAL" | "CRITICAL" => "severity >= 21",
                _ => "severity >= 1",
            };
            conditions.push(severity_condition.to_string());
        }

        if let Some(service) = &query.service {
            conditions.push(format!("service_name LIKE '%{}%'", service));
        }

        let where_clause = if conditions.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", conditions.join(" AND "))
        };

        let stream = query.index_pattern.as_deref().unwrap_or(&self.stream);

        format!(
            "SELECT * FROM \"{}\" {} ORDER BY _timestamp DESC LIMIT {}",
            stream, where_clause, query.max_results
        )
    }

    fn parse_log_entry(&self, record: &serde_json::Value) -> Option<LogEntry> {
        let severity_num = record
            .get("severity")
            .and_then(|v| {
                v.as_str()
                    .and_then(|s| s.parse::<i64>().ok())
                    .or_else(|| v.as_i64())
            })
            .unwrap_or(9);

        let level = match severity_num {
            1..=4 => "TRACE",
            5..=8 => "DEBUG",
            9..=12 => "INFO",
            13..=16 => "WARN",
            17..=20 => "ERROR",
            21..=24 => "FATAL",
            _ => "INFO",
        }
        .to_string();

        let body = record.get("body").and_then(|v| v.as_str()).unwrap_or("");
        let event = record.get("event").and_then(|v| v.as_str()).unwrap_or("");
        let payload = record.get("payload").and_then(|v| v.as_str());

        let message = if !body.is_empty() && body != "analytics" {
            body.to_string()
        } else if let Some(p) = payload {
            if p.len() > 300 {
                format!("[{}] {}...", event, &p[..300])
            } else {
                format!("[{}] {}", event, p)
            }
        } else {
            format!("[{}] {}", event, body)
        };

        Some(LogEntry {
            timestamp: record
                .get("_timestamp")
                .and_then(|v| v.as_i64())
                .map(|ts| {
                    chrono::DateTime::from_timestamp_micros(ts)
                        .map(|dt| dt.to_rfc3339())
                        .unwrap_or_default()
                })
                .unwrap_or_default(),
            level,
            message,
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
        let search_url = format!("{}/api/{}/_search?type=logs", self.url, self.organization);

        let sql_query = self.build_sql_query(query);

        let now = chrono::Utc::now();
        let start_time = query
            .start_time
            .as_ref()
            .and_then(|t| chrono::DateTime::parse_from_rfc3339(t).ok())
            .map(|dt| dt.timestamp_micros())
            .unwrap_or_else(|| (now - chrono::Duration::hours(1)).timestamp_micros());
        let end_time = query
            .end_time
            .as_ref()
            .and_then(|t| chrono::DateTime::parse_from_rfc3339(t).ok())
            .map(|dt| dt.timestamp_micros())
            .unwrap_or_else(|| now.timestamp_micros());

        let body = json!({
            "query": {
                "sql": sql_query,
                "start_time": start_time,
                "end_time": end_time,
                "from": 0,
                "size": query.max_results
            }
        });

        let response = self
            .client
            .post(&search_url)
            .basic_auth(&self.username, Some(&self.password))
            .header(
                "Referer",
                format!("{}/web/logs?org_identifier={}", self.url, self.organization),
            )
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

        let empty_vec = vec![];
        let hits = result
            .get("hits")
            .and_then(|h| h.as_array())
            .unwrap_or(&empty_vec);

        Ok(hits
            .iter()
            .filter_map(|hit| self.parse_log_entry(hit))
            .collect())
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
            index_pattern: None,
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
            index_pattern: None,
        };

        let sql = client.build_sql_query(&query);
        assert!(sql.contains("severity >= 17 AND severity <= 20"));
        assert!(sql.contains("service_name LIKE '%api-service%'"));
        assert!(sql.contains("LIMIT 50"));
    }
}
