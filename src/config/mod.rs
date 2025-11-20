use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub servers: HashMap<String, McpServer>,
    #[serde(default)]
    pub backends: HashMap<String, LogBackend>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServer {
    pub command: String,
    pub args: Vec<String>,
    #[serde(default)]
    pub env: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum LogBackend {
    Elasticsearch {
        url: String,
        username: Option<String>,
        password: Option<String>,
        #[serde(default = "default_index_pattern")]
        index_pattern: String,
        #[serde(default)]
        verify_ssl: bool,
    },
    OpenObserve {
        url: String,
        username: String,
        password: String,
        #[serde(default = "default_organization")]
        organization: String,
        #[serde(default = "default_stream")]
        stream: String,
        #[serde(default)]
        verify_ssl: bool,
    },
    Kibana {
        url: String,
        auth_token: Option<String>,
        #[serde(default = "default_index_pattern")]
        index_pattern: String,
        #[serde(default)]
        verify_ssl: bool,
        #[serde(default = "default_kibana_version")]
        version: String,
    },
}

fn default_index_pattern() -> String {
    "logs-*".to_string()
}

fn default_organization() -> String {
    "default".to_string()
}

fn default_stream() -> String {
    "default".to_string()
}

fn default_kibana_version() -> String {
    "7.10.2".to_string()
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;
        
        if !config_path.exists() {
            // Create default config
            let default_config = Self::default_config();
            default_config.save()?;
            return Ok(default_config);
        }
        
        let config_str = fs::read_to_string(&config_path)
            .context("Failed to read config file")?;
        
        let config: Config = serde_json::from_str(&config_str)
            .context("Failed to parse config file")?;
        
        Ok(config)
    }
    
    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path()?;
        
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)
                .context("Failed to create config directory")?;
        }
        
        let config_str = serde_json::to_string_pretty(self)
            .context("Failed to serialize config")?;
        
        fs::write(&config_path, config_str)
            .context("Failed to write config file")?;
        
        Ok(())
    }
    
    pub fn config_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .context("Could not determine config directory")?;
        Ok(config_dir.join("zeteo-cli").join("config.json"))
    }
    
    fn default_config() -> Self {
        let mut servers = HashMap::new();
        let mut backends = HashMap::new();
        
        let mut env = HashMap::new();
        env.insert("ELASTICSEARCH_URL".to_string(), "http://localhost:9200".to_string());
        env.insert("ELASTICSEARCH_USERNAME".to_string(), "elastic".to_string());
        env.insert("ELASTICSEARCH_PASSWORD".to_string(), "changeme".to_string());
        env.insert("SERVER_NAME".to_string(), "otel-mcp-server".to_string());
        env.insert("LOGLEVEL".to_string(), "OFF".to_string());
        
        servers.insert("otel-mcp-server".to_string(), McpServer {
            command: "npx".to_string(),
            args: vec!["-y".to_string(), "otel-mcp-server".to_string()],
            env,
        });
        
        // Add default backends
        backends.insert("elasticsearch".to_string(), LogBackend::Elasticsearch {
            url: "http://localhost:9200".to_string(),
            username: Some("elastic".to_string()),
            password: Some("changeme".to_string()),
            index_pattern: "logs-*".to_string(),
            verify_ssl: false,
        });
        
        backends.insert("openobserve".to_string(), LogBackend::OpenObserve {
            url: "http://localhost:5080".to_string(),
            username: "admin@example.com".to_string(),
            password: "changeme".to_string(),
            organization: "default".to_string(),
            stream: "default".to_string(),
            verify_ssl: false,
        });
        
        backends.insert("kibana".to_string(), LogBackend::Kibana {
            url: "http://localhost:5601".to_string(),
            auth_token: None,
            index_pattern: "logs-*".to_string(),
            verify_ssl: false,
            version: "7.10.2".to_string(),
        });
        
        Config { servers, backends }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_has_otel_server() {
        let config = Config::default_config();
        assert!(config.servers.contains_key("otel-mcp-server"));
    }

    #[test]
    fn test_otel_server_config() {
        let config = Config::default_config();
        let server = config.servers.get("otel-mcp-server").unwrap();
        
        assert_eq!(server.command, "npx");
        assert_eq!(server.args, vec!["-y", "otel-mcp-server"]);
        assert!(server.env.contains_key("ELASTICSEARCH_URL"));
        assert_eq!(server.env.get("ELASTICSEARCH_URL").unwrap(), "http://localhost:9200");
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::default_config();
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: Config = serde_json::from_str(&json).unwrap();
        
        assert_eq!(config.servers.len(), deserialized.servers.len());
        assert!(deserialized.servers.contains_key("otel-mcp-server"));
        assert_eq!(config.backends.len(), deserialized.backends.len());
        assert!(deserialized.backends.contains_key("elasticsearch"));
        assert!(deserialized.backends.contains_key("openobserve"));
        assert!(deserialized.backends.contains_key("kibana"));
    }
    
    #[test]
    fn test_backend_types() {
        let config = Config::default_config();
        
        // Check Elasticsearch backend
        if let Some(LogBackend::Elasticsearch { url, .. }) = config.backends.get("elasticsearch") {
            assert_eq!(url, "http://localhost:9200");
        } else {
            panic!("Elasticsearch backend not found or wrong type");
        }
        
        // Check OpenObserve backend
        if let Some(LogBackend::OpenObserve { url, organization, stream, .. }) = config.backends.get("openobserve") {
            assert_eq!(url, "http://localhost:5080");
            assert_eq!(organization, "default");
            assert_eq!(stream, "default");
        } else {
            panic!("OpenObserve backend not found or wrong type");
        }
        
        // Check Kibana backend
        if let Some(LogBackend::Kibana { url, version, .. }) = config.backends.get("kibana") {
            assert_eq!(url, "http://localhost:5601");
            assert_eq!(version, "7.10.2");
        } else {
            panic!("Kibana backend not found or wrong type");
        }
    }
}

