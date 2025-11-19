use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::*;

mod config;
mod mcp;
mod providers;
mod logs;

use config::Config;
use logs::LogExplorer;
use providers::{AiProvider, ChatRequest, Message};

#[derive(Parser)]
#[command(name = "zeteo-cli")]
#[command(author, version, about = "A Rust-based CLI AI agent with OTEL log exploration", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Search and explore OTEL logs
    Logs {
        /// Search query for logs
        #[arg(short, long)]
        query: Option<String>,
        
        /// Maximum number of results
        #[arg(short, long, default_value = "50")]
        max: usize,
        
        /// Interactive mode
        #[arg(short, long)]
        interactive: bool,
    },
    
    /// Chat with AI about logs or general questions
    Chat {
        /// AI provider to use (openai, vertex, google, azure)
        #[arg(short, long, default_value = "openai")]
        provider: String,
        
        /// Your message to the AI
        message: Option<String>,
    },
    
    /// Show or edit configuration
    Config {
        /// Show current configuration
        #[arg(short, long)]
        show: bool,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Logs { query, max, interactive } => {
            handle_logs(query, max, interactive).await?;
        }
        Commands::Chat { provider, message } => {
            handle_chat(provider, message).await?;
        }
        Commands::Config { show } => {
            handle_config(show)?;
        }
    }
    
    Ok(())
}

async fn handle_logs(query: Option<String>, max: usize, interactive: bool) -> Result<()> {
    let _config = Config::load()?;
    
    let server_name = "otel-mcp-server".to_string();
    let explorer = LogExplorer::new(server_name.clone());
    
    if interactive {
        explorer.interactive_mode().await?;
    } else if let Some(q) = query {
        let logs = explorer.search_logs(&q, max).await?;
        explorer.display_logs(&logs);
    } else {
        println!("{}", "Please provide a query with --query or use --interactive mode".yellow());
    }
    
    Ok(())
}

async fn handle_chat(provider_name: String, message: Option<String>) -> Result<()> {
    println!("{}", format!("Using AI provider: {}", provider_name).cyan());
    
    if let Some(msg) = message {
        let request = ChatRequest {
            messages: vec![Message {
                role: "user".to_string(),
                content: msg,
            }],
            temperature: Some(0.7),
            max_tokens: Some(1000),
        };
        
        // For demonstration, we'll use OpenAI if API key is available
        if let Ok(api_key) = std::env::var("OPENAI_API_KEY") {
            let provider = providers::OpenAiProvider::new(api_key, None);
            let response = provider.chat(request).await?;
            println!("\n{}", response.content.green());
        } else {
            println!("{}", "Please set OPENAI_API_KEY environment variable".yellow());
            println!("{}", "Other providers (Vertex, Google AI, Azure) are placeholders for now".dimmed());
        }
    } else {
        println!("{}", "Interactive chat mode coming soon!".yellow());
        println!("{}", "For now, use: zeteo-cli chat <your message>".dimmed());
    }
    
    Ok(())
}

fn handle_config(show: bool) -> Result<()> {
    let config = Config::load()?;
    
    if show {
        println!("{}", "=== Zeteo CLI Configuration ===".green().bold());
        println!("\nMCP Servers:");
        for (name, server) in &config.servers {
            println!("\n  {}", name.cyan().bold());
            println!("    Command: {}", server.command);
            println!("    Args: {:?}", server.args);
            if !server.env.is_empty() {
                println!("    Environment:");
                for (key, value) in &server.env {
                    println!("      {}: {}", key, value);
                }
            }
        }
    } else {
        println!("{}", "Use --show to display configuration".yellow());
    }
    
    Ok(())
}
