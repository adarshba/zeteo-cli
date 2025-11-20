use anyhow::{Context, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde_json::json;
use std::collections::HashMap;

use super::{LogBackendClient, LogEntry, LogQuery};

pub struct ElasticsearchClient {
    url: String,
    username: Option<String>,
    password: Option<String>,
    index_pattern: String,
    client: Client,
}

impl ElasticsearchClient {
    pub fn new(
        url: String,
        username: Option<String>,
        password: Option<String>,
        index_pattern: String,
        verify_ssl: bool,
    ) -> Result<Self> {
        let client = Client::builder()
            .danger_accept_invalid_certs(!verify_ssl)
            .build()
            .context("Failed to create HTTP client")?;

        Ok(ElasticsearchClient {
            url,
            username,
            password,
            index_pattern,
            client,
        })
    }

    fn build_query(&self, query: &LogQuery) -> serde_json::Value {
        let mut must = vec![];

        // Add query string query if present
        if !query.query.is_empty() && query.query != "*" {
            must.push(json!({
                "query_string": {
                    "query": query.query
                }
            }));
        }

        // Add level filter
        if let Some(level) = &query.level {
            must.push(json!({
                "term": {
                    "level": level.to_lowercase()
                }
            }));
        }

        // Add service filter
        if let Some(service) = &query.service {
            must.push(json!({
                "term": {
                    "service.name": service
                }
            }));
        }

        // Add time range filter
        if query.start_time.is_some() || query.end_time.is_some() {
            let mut range_query = json!({});
            if let Some(start) = &query.start_time {
                range_query["gte"] = json!(start);
            }
            if let Some(end) = &query.end_time {
                range_query["lte"] = json!(end);
            }
            must.push(json!({
                "range": {
                    "@timestamp": range_query
                }
            }));
        }

        let bool_query = if must.is_empty() {
            json!({
                "match_all": {}
            })
        } else {
            json!({
                "bool": {
                    "must": must
                }
            })
        };

        json!({
            "query": bool_query,
            "size": query.max_results,
            "sort": [
                {
                    "@timestamp": {
                        "order": "desc"
                    }
                }
            ]
        })
    }

    fn parse_log_entry(&self, hit: &serde_json::Value) -> Option<LogEntry> {
        let source = hit.get("_source")?;

        Some(LogEntry {
            timestamp: source
                .get("@timestamp")
                .or_else(|| source.get("timestamp"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            level: source
                .get("level")
                .or_else(|| source.get("severity"))
                .and_then(|v| v.as_str())
                .unwrap_or("INFO")
                .to_uppercase(),
            message: source
                .get("message")
                .or_else(|| source.get("body"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            service: source
                .get("service")
                .and_then(|s| s.get("name"))
                .or_else(|| source.get("service_name"))
                .and_then(|v| v.as_str())
                .map(String::from),
            trace_id: source
                .get("trace_id")
                .or_else(|| source.get("traceId"))
                .and_then(|v| v.as_str())
                .map(String::from),
            labels: HashMap::new(),
        })
    }
}

#[async_trait]
impl LogBackendClient for ElasticsearchClient {
    async fn query_logs(&self, query: &LogQuery) -> Result<Vec<LogEntry>> {
        let search_url = format!("{}{}/_search", self.url, self.index_pattern);
        let body = self.build_query(query);

        let mut request = self.client.post(&search_url).json(&body);

        if let (Some(username), Some(password)) = (&self.username, &self.password) {
            request = request.basic_auth(username, Some(password));
        }

        let response = request
            .send()
            .await
            .context("Failed to send query to Elasticsearch")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!(
                "Elasticsearch query failed with status {}: {}",
                status,
                error_text
            ));
        }

        let result: serde_json::Value = response
            .json()
            .await
            .context("Failed to parse Elasticsearch response")?;

        let hits = result
            .get("hits")
            .and_then(|h| h.get("hits"))
            .and_then(|h| h.as_array())
            .context("Invalid Elasticsearch response format")?;

        Ok(hits.iter().filter_map(|hit| self.parse_log_entry(hit)).collect())
    }

    async fn health_check(&self) -> Result<bool> {
        let health_url = format!("{}/_cluster/health", self.url);
        let mut request = self.client.get(&health_url);

        if let (Some(username), Some(password)) = (&self.username, &self.password) {
            request = request.basic_auth(username, Some(password));
        }

        let response = request.send().await;
        Ok(response.is_ok() && response.unwrap().status().is_success())
    }

    fn backend_name(&self) -> &str {
        "Elasticsearch"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_query_simple() {
        let client = ElasticsearchClient::new(
            "http://localhost:9200".to_string(),
            None,
            None,
            "logs-*".to_string(),
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

        let es_query = client.build_query(&query);
        assert!(es_query.get("query").is_some());
        assert_eq!(es_query.get("size").unwrap().as_u64().unwrap(), 10);
    }

    #[test]
    fn test_build_query_with_filters() {
        let client = ElasticsearchClient::new(
            "http://localhost:9200".to_string(),
            None,
            None,
            "logs-*".to_string(),
            false,
        )
        .unwrap();

        let query = LogQuery {
            query: "*".to_string(),
            max_results: 50,
            start_time: Some("2024-01-01T00:00:00Z".to_string()),
            end_time: Some("2024-01-02T00:00:00Z".to_string()),
            level: Some("ERROR".to_string()),
            service: Some("api-service".to_string()),
        };

        let es_query = client.build_query(&query);
        assert!(es_query.get("query").is_some());
        assert_eq!(es_query.get("size").unwrap().as_u64().unwrap(), 50);
    }
}
