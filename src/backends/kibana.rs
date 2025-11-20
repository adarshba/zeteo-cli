use anyhow::{Context, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde_json::json;
use std::collections::HashMap;

use super::{LogBackendClient, LogEntry, LogQuery};

pub struct KibanaClient {
    url: String,
    auth_token: Option<String>,
    index_pattern: String,
    version: String,
    client: Client,
}

impl KibanaClient {
    pub fn new(
        url: String,
        auth_token: Option<String>,
        index_pattern: String,
        version: String,
        verify_ssl: bool,
    ) -> Result<Self> {
        let client = Client::builder()
            .danger_accept_invalid_certs(!verify_ssl)
            .build()
            .context("Failed to create HTTP client")?;

        Ok(KibanaClient {
            url,
            auth_token,
            index_pattern,
            version,
            client,
        })
    }

    fn build_kql_query(&self, query: &LogQuery) -> String {
        let mut kql_parts = vec![];

        // Add main query
        if !query.query.is_empty() && query.query != "*" {
            kql_parts.push(query.query.clone());
        }

        // Add level filter
        if let Some(level) = &query.level {
            kql_parts.push(format!("level: \"{}\"", level));
        }

        // Add service filter
        if let Some(service) = &query.service {
            kql_parts.push(format!("service.name: \"{}\"", service));
        }

        if kql_parts.is_empty() {
            "*".to_string()
        } else {
            kql_parts.join(" AND ")
        }
    }

    fn build_search_body(&self, query: &LogQuery) -> serde_json::Value {
        let kql = self.build_kql_query(query);

        let mut body = json!({
            "params": {
                "index": self.index_pattern,
                "body": {
                    "version": true,
                    "size": query.max_results,
                    "sort": [
                        {
                            "@timestamp": {
                                "order": "desc"
                            }
                        }
                    ],
                    "query": {
                        "bool": {
                            "must": [],
                            "filter": [
                                {
                                    "query_string": {
                                        "query": kql,
                                        "analyze_wildcard": true
                                    }
                                }
                            ]
                        }
                    }
                }
            }
        });

        // Add time range if specified
        if query.start_time.is_some() || query.end_time.is_some() {
            let mut range = json!({});
            if let Some(start) = &query.start_time {
                range["gte"] = json!(start);
            }
            if let Some(end) = &query.end_time {
                range["lte"] = json!(end);
            }

            if let Some(filter) = body["params"]["body"]["query"]["bool"]["filter"].as_array_mut() {
                filter.push(json!({
                    "range": {
                        "@timestamp": range
                    }
                }));
            }
        }

        body
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
impl LogBackendClient for KibanaClient {
    async fn query_logs(&self, query: &LogQuery) -> Result<Vec<LogEntry>> {
        // Kibana uses internal Elasticsearch API
        let search_url = format!("{}/_plugin/kibana/api/console/proxy", self.url);
        let body = self.build_search_body(query);

        let mut request = self
            .client
            .post(&search_url)
            .header("kbn-xsrf", "true")
            .header("kbn-version", &self.version)
            .header("Content-Type", "application/json")
            .json(&body);

        // Add authentication if token is present
        if let Some(token) = &self.auth_token {
            request = request.header("Authorization", format!("Bearer {}", token));
        }

        let response = request
            .send()
            .await
            .context("Failed to send query to Kibana")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!(
                "Kibana query failed with status {}: {}",
                status,
                error_text
            ));
        }

        let result: serde_json::Value = response
            .json()
            .await
            .context("Failed to parse Kibana response")?;

        // Kibana wraps the Elasticsearch response
        let hits = result
            .get("rawResponse")
            .or_else(|| result.get("response"))
            .and_then(|r| r.get("hits"))
            .and_then(|h| h.get("hits"))
            .and_then(|h| h.as_array())
            .context("Invalid Kibana response format")?;

        Ok(hits.iter().filter_map(|hit| self.parse_log_entry(hit)).collect())
    }

    async fn health_check(&self) -> Result<bool> {
        let health_url = format!("{}/api/status", self.url);
        let mut request = self.client.get(&health_url);

        if let Some(token) = &self.auth_token {
            request = request.header("Authorization", format!("Bearer {}", token));
        }

        let response = request.send().await;
        Ok(response.is_ok() && response.unwrap().status().is_success())
    }

    fn backend_name(&self) -> &str {
        "Kibana"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_kql_query_simple() {
        let client = KibanaClient::new(
            "http://localhost:5601".to_string(),
            None,
            "logs-*".to_string(),
            "7.10.2".to_string(),
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

        let kql = client.build_kql_query(&query);
        assert_eq!(kql, "error");
    }

    #[test]
    fn test_build_kql_query_with_filters() {
        let client = KibanaClient::new(
            "http://localhost:5601".to_string(),
            None,
            "logs-*".to_string(),
            "7.10.2".to_string(),
            false,
        )
        .unwrap();

        let query = LogQuery {
            query: "payment".to_string(),
            max_results: 50,
            start_time: None,
            end_time: None,
            level: Some("ERROR".to_string()),
            service: Some("api-service".to_string()),
        };

        let kql = client.build_kql_query(&query);
        assert!(kql.contains("payment"));
        assert!(kql.contains("level: \"ERROR\""));
        assert!(kql.contains("service.name: \"api-service\""));
    }

    #[test]
    fn test_build_search_body() {
        let client = KibanaClient::new(
            "http://localhost:5601".to_string(),
            None,
            "logs-*".to_string(),
            "7.10.2".to_string(),
            false,
        )
        .unwrap();

        let query = LogQuery {
            query: "error".to_string(),
            max_results: 10,
            start_time: Some("2024-01-01T00:00:00Z".to_string()),
            end_time: Some("2024-01-02T00:00:00Z".to_string()),
            level: None,
            service: None,
        };

        let body = client.build_search_body(&query);
        assert!(body.get("params").is_some());
        assert!(body["params"]["body"]["size"].as_u64().unwrap() == 10);
    }
}
