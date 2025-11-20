use anyhow::Result;
use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::{generate, Shell};
use colored::*;
use std::io;

mod cache;
mod config;
mod logs;
mod mcp;
mod providers;
mod repl;
mod retry;
mod tui;

use config::Config;
use logs::LogExplorer;
use providers::{AiProvider, ChatRequest, Message};

#[derive(Parser)]
#[command(name = "zeteo")]
#[command(author, version, about = "A Rust-based CLI AI agent with OTEL log exploration", long_about = None)]
struct Cli {
    /// Enable verbose logging
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Output format (text, json)
    #[arg(short, long, global = true, default_value = "text")]
    output: OutputFormat,

    /// AI provider to use in REPL mode (openai, vertex, google, azure)
    #[arg(short, long, global = true)]
    provider: Option<String>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Clone, Debug, clap::ValueEnum)]
enum OutputFormat {
    Text,
    Json,
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

        /// Stream results in real-time
        #[arg(short, long)]
        stream: bool,

        /// Filter by log level (ERROR, WARN, INFO, DEBUG)
        #[arg(long)]
        level: Option<String>,

        /// Filter by service name
        #[arg(long)]
        service: Option<String>,

        /// Show aggregated statistics
        #[arg(long)]
        aggregate: bool,

        /// Export to file (json or csv)
        #[arg(long)]
        export: Option<String>,
    },
    
    /// Chat with AI about logs or general questions
    Chat {
        /// AI provider to use (openai, vertex, google, azure)
        #[arg(short, long, default_value = "openai")]
        provider: String,
        
        /// Your message to the AI
        message: Option<String>,

        /// Enable streaming responses
        #[arg(short, long)]
        stream: bool,
    },
    
    /// Show or edit configuration
    Config {
        /// Show current configuration
        #[arg(short, long)]
        show: bool,

        /// Initialize configuration file
        #[arg(short, long)]
        init: bool,
    },

    /// Full-screen TUI mode with split panels
    Tui {
        /// AI provider to use (openai, vertex, google, azure)
        #[arg(short, long)]
        provider: Option<String>,
    },

    /// Generate shell completions
    Completions {
        /// Shell type
        #[arg(value_enum)]
        shell: Shell,
    },

    /// Display version information
    Version,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Load .env file if it exists (silently ignore if not found)
    // This allows users to store API keys in a .env file
    let _ = dotenv::dotenv();
    
    // Setup graceful shutdown
    let (shutdown_tx, mut shutdown_rx) = tokio::sync::mpsc::channel::<()>(1);
    
    tokio::spawn(async move {
        if let Err(e) = tokio::signal::ctrl_c().await {
            eprintln!("Failed to listen for ctrl-c: {}", e);
        }
        let _ = shutdown_tx.send(()).await;
    });

    let cli = Cli::parse();
    
    if cli.verbose {
        println!("{}", "Verbose mode enabled".dimmed());
        if dotenv::dotenv().is_ok() {
            println!("{}", "Loaded environment variables from .env file".dimmed());
        }
    }
    
    let result = tokio::select! {
        res = run_command(cli.command, cli.output, cli.provider) => res,
        _ = shutdown_rx.recv() => {
            println!("\n{}", "Received shutdown signal, cleaning up...".yellow());
            Ok(())
        }
    };

    result
}

async fn run_command(command: Option<Commands>, output_format: OutputFormat, provider: Option<String>) -> Result<()> {
    // If no command is provided, enter REPL mode
    if command.is_none() {
        return run_repl_mode(provider).await;
    }

    match command.unwrap() {
        Commands::Logs { query, max, interactive, stream, level, service, aggregate, export } => {
            handle_logs(query, max, interactive, stream, level, service, aggregate, export, output_format).await?;
        }
        Commands::Chat { provider, message, stream } => {
            handle_chat(provider, message, stream, output_format).await?;
        }
        Commands::Config { show, init } => {
            handle_config(show, init)?;
        }
        Commands::Tui { provider } => {
            run_tui_mode(provider).await?;
        }
        Commands::Completions { shell } => {
            generate_completions(shell);
        }
        Commands::Version => {
            print_version();
        }
    }
    
    Ok(())
}

async fn run_repl_mode(provider: Option<String>) -> Result<()> {
    let mut session = repl::create_repl_session(provider).await?;
    session.run().await
}

async fn run_tui_mode(provider: Option<String>) -> Result<()> {
    let mut app = tui::create_tui_session(provider).await?;
    app.run().await
}

async fn handle_logs(
    query: Option<String>,
    max: usize,
    interactive: bool,
    stream: bool,
    level: Option<String>,
    service: Option<String>,
    aggregate: bool,
    export: Option<String>,
    output_format: OutputFormat,
) -> Result<()> {
    let _config = Config::load()?;
    
    let server_name = "otel-mcp-server".to_string();
    let explorer = LogExplorer::new(server_name.clone());
    
    if interactive {
        explorer.interactive_mode().await?;
    } else if stream {
        println!("{}", "Streaming mode enabled...".cyan());
        let query_str = query.as_deref().unwrap_or("");
        
        explorer.stream_logs(query_str, |_log| {
            // Return true to continue streaming
            true
        }).await?;
    } else if let Some(q) = query {
        let logs = if level.is_some() || service.is_some() {
            let filter = logs::LogFilter {
                level,
                service,
                start_time: None,
                end_time: None,
                contains: None,
            };
            explorer.search_logs_with_filter(&q, max, &filter).await?
        } else {
            explorer.search_logs(&q, max).await?
        };

        if aggregate {
            let agg = explorer.aggregate_logs(&logs);
            explorer.display_aggregation(&agg);
        }

        if let Some(export_path) = export {
            if export_path.ends_with(".csv") {
                explorer.export_logs_csv(&logs, &export_path)?;
            } else {
                // Default to JSON
                let json_path = if export_path.ends_with(".json") {
                    export_path
                } else {
                    format!("{}.json", export_path)
                };
                explorer.export_logs_json(&logs, &json_path)?;
            }
        } else {
            match output_format {
                OutputFormat::Text => explorer.display_logs(&logs),
                OutputFormat::Json => {
                    let json = serde_json::to_string_pretty(&logs)?;
                    println!("{}", json);
                }
            }
        }
    } else {
        println!("{}", "Please provide a query with --query or use --interactive mode".yellow());
        println!("{}", "Example: zeteo logs --query \"error\" --max 10".dimmed());
        println!();
        println!("{}", "Advanced examples:".bold());
        println!("  {} - Filter by log level", "zeteo logs --query \"error\" --level ERROR".dimmed());
        println!("  {} - Filter by service", "zeteo logs --query \"*\" --service \"api\"".dimmed());
        println!("  {} - Show statistics", "zeteo logs --query \"error\" --aggregate".dimmed());
        println!("  {} - Export to JSON", "zeteo logs --query \"error\" --export logs.json".dimmed());
        println!("  {} - Export to CSV", "zeteo logs --query \"error\" --export logs.csv".dimmed());
        println!("  {} - Stream logs in real-time", "zeteo logs --query \"*\" --stream".dimmed());
    }
    
    Ok(())
}

async fn handle_chat(
    provider_name: String,
    message: Option<String>,
    stream: bool,
    output_format: OutputFormat,
) -> Result<()> {
    match output_format {
        OutputFormat::Text => {
            println!("{}", format!("Using AI provider: {}", provider_name).cyan());
        }
        OutputFormat::Json => {}
    }
    
    if let Some(msg) = message {
        let request = ChatRequest {
            messages: vec![Message {
                role: "user".to_string(),
                content: msg,
            }],
            temperature: Some(0.7),
            max_tokens: Some(1000),
        };
        
        if stream {
            println!("{}", "Streaming mode enabled...".cyan());
        }

        let response = match provider_name.to_lowercase().as_str() {
            "openai" => {
                let api_key = std::env::var("OPENAI_API_KEY")
                    .map_err(|_| anyhow::anyhow!("OPENAI_API_KEY not set"))?;
                let provider = providers::OpenAiProvider::new(api_key, None);
                provider.chat(request).await?
            }
            "vertex" => {
                let project_id = std::env::var("GOOGLE_CLOUD_PROJECT")
                    .map_err(|_| anyhow::anyhow!("GOOGLE_CLOUD_PROJECT not set"))?;
                let location = std::env::var("GOOGLE_CLOUD_LOCATION")
                    .unwrap_or_else(|_| "us-central1".to_string());
                let provider = providers::VertexProvider::new(project_id, location, None);
                provider.chat(request).await?
            }
            "google" => {
                let api_key = std::env::var("GOOGLE_API_KEY")
                    .map_err(|_| anyhow::anyhow!("GOOGLE_API_KEY not set"))?;
                let provider = providers::GoogleProvider::new(api_key, None);
                provider.chat(request).await?
            }
            "azure" => {
                let api_key = std::env::var("AZURE_OPENAI_API_KEY")
                    .map_err(|_| anyhow::anyhow!("AZURE_OPENAI_API_KEY not set"))?;
                let endpoint = std::env::var("AZURE_OPENAI_ENDPOINT")
                    .map_err(|_| anyhow::anyhow!("AZURE_OPENAI_ENDPOINT not set"))?;
                let deployment = std::env::var("AZURE_OPENAI_DEPLOYMENT")
                    .map_err(|_| anyhow::anyhow!("AZURE_OPENAI_DEPLOYMENT not set"))?;
                let provider = providers::AzureProvider::new(api_key, endpoint, deployment);
                provider.chat(request).await?
            }
            _ => {
                anyhow::bail!("Unknown provider: {}. Supported: openai, vertex, google, azure", provider_name);
            }
        };

        match output_format {
            OutputFormat::Text => {
                println!("\n{}", response.content.green());
            }
            OutputFormat::Json => {
                let json = serde_json::json!({
                    "content": response.content,
                    "model": response.model,
                    "provider": provider_name
                });
                println!("{}", serde_json::to_string_pretty(&json)?);
            }
        }
    } else {
        println!("{}", "Please provide a message to chat with AI".yellow());
        println!("{}", "Example: zeteo chat \"Explain OTEL logs\"".dimmed());
        println!();
        println!("{}", "Supported providers:".bold());
        println!("  {} - Set OPENAI_API_KEY", "openai".cyan());
        println!("  {} - Set GOOGLE_CLOUD_PROJECT, authenticate with gcloud", "vertex".cyan());
        println!("  {} - Set GOOGLE_API_KEY", "google".cyan());
        println!("  {} - Set AZURE_OPENAI_API_KEY, AZURE_OPENAI_ENDPOINT, AZURE_OPENAI_DEPLOYMENT", "azure".cyan());
    }
    
    Ok(())
}

fn handle_config(show: bool, init: bool) -> Result<()> {
    if init {
        println!("{}", "Initializing configuration...".cyan());
        let config = Config::load()?;
        config.save()?;
        println!("{}", "Configuration initialized successfully!".green());
        if let Ok(path) = Config::config_path() {
            println!("Location: {}", path.display());
        }
        return Ok(());
    }

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
                    // Mask sensitive values
                    let display_value = if key.to_lowercase().contains("password")
                        || key.to_lowercase().contains("key")
                        || key.to_lowercase().contains("token")
                    {
                        "********".to_string()
                    } else {
                        value.clone()
                    };
                    println!("      {}: {}", key, display_value);
                }
            }
        }
        println!();
    } else {
        println!("{}", "Use --show to display configuration".yellow());
        println!("{}", "Use --init to initialize configuration file".yellow());
    }
    
    Ok(())
}

fn generate_completions(shell: Shell) {
    let mut cmd = Cli::command();
    let name = cmd.get_name().to_string();
    generate(shell, &mut cmd, name, &mut io::stdout());
}

fn print_version() {
    println!("Zeteo CLI v{}", env!("CARGO_PKG_VERSION"));
    println!("A Rust-based CLI AI agent with OTEL log exploration");
    println!("\nBuild information:");
    println!("  Profile: {}", if cfg!(debug_assertions) { "debug" } else { "release" });
    println!("\nFor more information, visit: https://github.com/adarshba/zeteo-cli");
}

