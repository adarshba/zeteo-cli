use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub servers: HashMap<String, McpServer>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServer {
    pub command: String,
    pub args: Vec<String>,
    #[serde(default)]
    pub env: HashMap<String, String>,
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
    
    fn config_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .context("Could not determine config directory")?;
        Ok(config_dir.join("zeteo-cli").join("config.json"))
    }
    
    fn default_config() -> Self {
        let mut servers = HashMap::new();
        
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
        
        Config { servers }
    }
}
