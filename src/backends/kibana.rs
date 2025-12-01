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

    fn build_search_body(&self, query: &LogQuery) -> serde_json::Value {
        let mut filters: Vec<serde_json::Value> = vec![];

        if !query.query.is_empty() && query.query != "*" {
            filters.push(json!({
                "multi_match": {
                    "type": "best_fields",
                    "query": query.query,
                    "lenient": true
                }
            }));
        }

        let now = chrono::Utc::now();
        let start_time = query
            .start_time
            .as_ref()
            .cloned()
            .unwrap_or_else(|| (now - chrono::Duration::hours(1)).to_rfc3339());
        let end_time = query
            .end_time
            .as_ref()
            .cloned()
            .unwrap_or_else(|| now.to_rfc3339());

        filters.push(json!({
            "range": {
                "timestamp": {
                    "gte": start_time,
                    "lte": end_time,
                    "format": "strict_date_optional_time"
                }
            }
        }));

        if let Some(level) = &query.level {
            let level_upper = level.to_uppercase();
            filters.push(json!({
                "multi_match": {
                    "type": "best_fields",
                    "query": level_upper,
                    "fields": ["level", "severity", "log_level"],
                    "lenient": true
                }
            }));
        }

        if let Some(service) = &query.service {
            filters.push(json!({
                "multi_match": {
                    "type": "best_fields",
                    "query": service,
                    "fields": ["pod_name", "pod_name.keyword", "service", "service_name"],
                    "lenient": true
                }
            }));
        }

        let index_pattern = query
            .index_pattern
            .as_deref()
            .unwrap_or(&self.index_pattern);

        json!({
            "params": {
                "index": index_pattern,
                "body": {
                    "version": true,
                    "size": query.max_results,
                    "sort": [
                        {
                            "timestamp": {
                                "order": "desc",
                                "unmapped_type": "boolean"
                            }
                        }
                    ],
                    "stored_fields": ["*"],
                    "docvalue_fields": [
                        {
                            "field": "timestamp",
                            "format": "date_time"
                        }
                    ],
                    "_source": {
                        "excludes": []
                    },
                    "query": {
                        "bool": {
                            "must": [],
                            "filter": filters,
                            "should": [],
                            "must_not": []
                        }
                    },
                    "highlight": {
                        "pre_tags": ["@kibana-highlighted-field@"],
                        "post_tags": ["@/kibana-highlighted-field@"],
                        "fields": {"*": {}},
                        "fragment_size": 2147483647
                    }
                },
                "preference": chrono::Utc::now().timestamp_millis()
            }
        })
    }

    fn parse_log_entry(&self, hit: &serde_json::Value) -> Option<LogEntry> {
        let source = hit.get("_source")?;
        let fields = hit.get("fields");

        let timestamp = fields
            .and_then(|f| f.get("timestamp"))
            .and_then(|t| t.as_array())
            .and_then(|arr| arr.first())
            .and_then(|v| v.as_str())
            .or_else(|| source.get("timestamp").and_then(|v| v.as_str()))
            .or_else(|| source.get("@timestamp").and_then(|v| v.as_str()))
            .unwrap_or("")
            .to_string();

        let level = source
            .get("level")
            .or_else(|| source.get("severity"))
            .or_else(|| source.get("log_level"))
            .and_then(|v| v.as_str())
            .unwrap_or("INFO")
            .to_uppercase();

        let message = source
            .get("message")
            .or_else(|| source.get("log"))
            .or_else(|| source.get("body"))
            .and_then(|v| v.as_str())
            .map(|s| {
                if s.len() > 500 {
                    format!("{}...", &s[..500])
                } else {
                    s.to_string()
                }
            })
            .unwrap_or_default();

        let service = source
            .get("pod_name")
            .or_else(|| source.get("service"))
            .or_else(|| source.get("service_name"))
            .and_then(|v| v.as_str())
            .map(|s| {
                if s.contains("breeze-api-custom-pre") {
                    "Vayu(Jockey) Preflight".to_string()
                } else if s.contains("breeze-api-custom") {
                    "Vayu(Jockey)".to_string()
                } else if s.contains("breeze-api-pre") {
                    "Vayu Preflight".to_string()
                } else if s.contains("breeze-api-producer") {
                    "Vayu Producer".to_string()
                } else if s.contains("breeze-api") {
                    "Vayu".to_string()
                } else if s.contains("breeze-app-jockey") {
                    "Nimble(Jockey)".to_string()
                } else if s.contains("breeze-app-analytics") {
                    "Nimble Analytics".to_string()
                } else if s.contains("breeze-app-cron") {
                    "Nimble Cron".to_string()
                } else if s.contains("breeze-app-") {
                    "Nimble".to_string()
                } else if s.contains("breeze-lighthouse-cron") {
                    "Lighthouse Cron".to_string()
                } else if s.contains("breeze-lighthouse-pre") {
                    "Lighthouse Preflight".to_string()
                } else if s.contains("breeze-lighthouse") {
                    "Lighthouse".to_string()
                } else {
                    s.to_string()
                }
            });

        let trace_id = source
            .get("trace_id")
            .or_else(|| source.get("traceId"))
            .or_else(|| source.get("request_id"))
            .and_then(|v| v.as_str())
            .map(String::from);

        Some(LogEntry {
            timestamp,
            level,
            message,
            service,
            trace_id,
            labels: HashMap::new(),
        })
    }
}

#[async_trait]
impl LogBackendClient for KibanaClient {
    async fn query_logs(&self, query: &LogQuery) -> Result<Vec<LogEntry>> {
        let search_url = format!("{}/_plugin/kibana/internal/search/es", self.url);
        let body = self.build_search_body(query);

        let mut request = self
            .client
            .post(&search_url)
            .header("Content-Type", "application/json")
            .header("kbn-version", &self.version)
            .header(
                "Referer",
                format!("{}/_plugin/kibana/app/discover", self.url),
            )
            .header("Origin", &self.url)
            .json(&body);

        if let Some(token) = &self.auth_token {
            request = request.header("Cookie", format!("_pomerium={}", token));
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

        let hits = result
            .get("rawResponse")
            .or_else(|| result.get("response"))
            .and_then(|r| r.get("hits"))
            .and_then(|h| h.get("hits"))
            .and_then(|h| h.as_array());

        match hits {
            Some(hits_array) => Ok(hits_array
                .iter()
                .filter_map(|hit| self.parse_log_entry(hit))
                .collect()),
            None => {
                let direct_hits = result
                    .get("hits")
                    .and_then(|h| h.get("hits"))
                    .and_then(|h| h.as_array());

                match direct_hits {
                    Some(hits_array) => Ok(hits_array
                        .iter()
                        .filter_map(|hit| self.parse_log_entry(hit))
                        .collect()),
                    None => Ok(vec![]),
                }
            }
        }
    }

    async fn health_check(&self) -> Result<bool> {
        let health_url = format!("{}/_plugin/kibana/api/status", self.url);
        let mut request = self.client.get(&health_url);

        if let Some(token) = &self.auth_token {
            request = request.header("Cookie", format!("_pomerium={}", token));
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
    fn test_build_search_body_simple() {
        let client = KibanaClient::new(
            "http://localhost:5601".to_string(),
            None,
            "breeze-v2*".to_string(),
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
            index_pattern: None,
        };

        let body = client.build_search_body(&query);
        assert!(body.get("params").is_some());
        assert_eq!(body["params"]["index"], "breeze-v2*");
        assert_eq!(body["params"]["body"]["size"], 10);
    }

    #[test]
    fn test_build_search_body_with_filters() {
        let client = KibanaClient::new(
            "http://localhost:5601".to_string(),
            None,
            "breeze-v2*".to_string(),
            "7.10.2".to_string(),
            false,
        )
        .unwrap();

        let query = LogQuery {
            query: "payment".to_string(),
            max_results: 50,
            start_time: Some("2024-01-01T00:00:00Z".to_string()),
            end_time: Some("2024-01-02T00:00:00Z".to_string()),
            level: Some("ERROR".to_string()),
            service: Some("vayu".to_string()),
            index_pattern: None,
        };

        let body = client.build_search_body(&query);
        let filters = body["params"]["body"]["query"]["bool"]["filter"]
            .as_array()
            .unwrap();
        assert!(filters.len() >= 2);
    }
}
