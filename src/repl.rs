use anyhow::Result;
use colored::*;
use dialoguer::{theme::ColorfulTheme, Input};
use std::sync::Arc;

use crate::config::Config;
use crate::logs::LogExplorer;
use crate::providers::{AiProvider, ChatRequest, Message};

pub struct ReplSession {
    provider: Arc<dyn AiProvider>,
    provider_name: String,
    conversation_history: Vec<Message>,
    log_explorer: Option<LogExplorer>,
}

impl ReplSession {
    pub fn new(provider: Arc<dyn AiProvider>, provider_name: String) -> Self {
        Self {
            provider,
            provider_name,
            conversation_history: Vec::new(),
            log_explorer: None,
        }
    }

    pub fn with_log_explorer(mut self, explorer: LogExplorer) -> Self {
        self.log_explorer = Some(explorer);
        self
    }

    pub async fn run(&mut self) -> Result<()> {
        self.print_welcome();

        loop {
            let input = match self.get_input() {
                Ok(input) => input,
                Err(_) => {
                    println!("\n{}", "Goodbye!".yellow());
                    break;
                }
            };

            let input = input.trim();

            if input.is_empty() {
                continue;
            }

            // Handle special commands
            if input.starts_with('/') {
                if self.handle_command(input).await? {
                    break; // Exit requested
                }
                continue;
            }

            // Regular chat message
            if let Err(e) = self.handle_chat_message(input).await {
                eprintln!("{} {}", "Error:".red().bold(), e);
            }
        }

        Ok(())
    }

    fn print_welcome(&self) {
        println!("\n{}", "╔═══════════════════════════════════════════════════════════╗".cyan());
        println!("{}", "║           Welcome to Zeteo Interactive Shell             ║".cyan());
        println!("{}", "╚═══════════════════════════════════════════════════════════╝".cyan());
        println!();
        println!("{} {}", "Provider:".bold(), self.provider_name.green());
        println!();
        println!("{}", "Available commands:".bold());
        println!("  {} - Exit the REPL", "/exit".cyan());
        println!("  {} - Clear conversation history", "/clear".cyan());
        println!("  {} - Show help", "/help".cyan());
        println!("  {} - Search logs (e.g., /logs error)", "/logs".cyan());
        println!("  {} - Switch provider (e.g., /provider openai)", "/provider".cyan());
        println!("  {} - Export conversation to file (json or csv)", "/export".cyan());
        println!("  {} - Show conversation history", "/history".cyan());
        println!();
        println!("{}", "Type your message and press Enter to chat.".dimmed());
        println!("{}", "Press Ctrl+C or type /exit to quit.".dimmed());
        println!();
    }

    fn get_input(&self) -> Result<String> {
        let prompt = format!("{}> ", "zeteo".cyan().bold());
        
        Input::with_theme(&ColorfulTheme::default())
            .with_prompt(&prompt)
            .allow_empty(true)
            .interact_text()
            .map_err(|e| anyhow::anyhow!("Input error: {}", e))
    }

    async fn handle_command(&mut self, command: &str) -> Result<bool> {
        let parts: Vec<&str> = command.split_whitespace().collect();
        let cmd = parts[0].to_lowercase();

        match cmd.as_str() {
            "/exit" | "/quit" | "/q" => {
                println!("{}", "Exiting REPL...".yellow());
                return Ok(true);
            }
            "/clear" => {
                self.conversation_history.clear();
                println!("{}", "Conversation history cleared.".green());
            }
            "/help" | "/h" => {
                self.print_help();
            }
            "/logs" => {
                if parts.len() > 1 {
                    let query = parts[1..].join(" ");
                    self.handle_logs_command(&query).await?;
                } else {
                    println!("{}", "Usage: /logs <query>".yellow());
                    println!("{}", "Example: /logs error".dimmed());
                }
            }
            "/provider" => {
                if parts.len() > 1 {
                    println!("{}", "Provider switching not yet implemented in current session.".yellow());
                    println!("{}", "Please restart zeteo with --provider flag.".dimmed());
                } else {
                    println!("{}", "Current provider: ".bold().to_string() + &self.provider_name.green().to_string());
                }
            }
            "/export" => {
                let filename = parts.get(1).copied();
                self.export_conversation(filename)?;
            }
            "/history" => {
                self.show_history();
            }
            _ => {
                println!("{} {}", "Unknown command:".red(), command);
                println!("{}", "Type /help for available commands".dimmed());
            }
        }

        Ok(false)
    }

    async fn handle_chat_message(&mut self, input: &str) -> Result<()> {
        // Add user message to history
        let user_message = Message {
            role: "user".to_string(),
            content: input.to_string(),
        };
        self.conversation_history.push(user_message.clone());

        // Create request with full conversation history
        let request = ChatRequest {
            messages: self.conversation_history.clone(),
            temperature: Some(0.7),
            max_tokens: Some(2000),
        };

        // Show thinking indicator
        print!("{}", "Thinking...".dimmed());
        std::io::Write::flush(&mut std::io::stdout())?;
        print!("\r");

        // Get response from AI
        let response = self.provider.chat(request).await?;

        // Add assistant response to history
        let assistant_message = Message {
            role: "assistant".to_string(),
            content: response.content.clone(),
        };
        self.conversation_history.push(assistant_message);

        // Display response
        println!("\n{}", response.content.green());
        println!();

        Ok(())
    }

    async fn handle_logs_command(&self, query: &str) -> Result<()> {
        if let Some(explorer) = &self.log_explorer {
            println!("{}", format!("Searching logs for: '{}'", query).cyan());
            let logs = explorer.search_logs(query, 20).await?;
            
            if logs.is_empty() {
                println!("{}", "No logs found.".yellow());
            } else {
                explorer.display_logs(&logs);
            }
        } else {
            println!("{}", "Log explorer not available. Please configure MCP server.".yellow());
        }
        Ok(())
    }

    fn export_conversation(&self, filename: Option<&str>) -> Result<()> {
        use std::fs::File;
        use std::io::Write;

        let filename = filename.unwrap_or("conversation.json");
        
        if filename.ends_with(".csv") {
            // Export as CSV
            let mut file = File::create(filename)?;
            writeln!(file, "role,content")?;
            
            for msg in &self.conversation_history {
                let content = msg.content.replace(",", ";").replace("\n", " ");
                writeln!(file, "{},{}", msg.role, content)?;
            }
            
            println!("{}", format!("Conversation exported to: {}", filename).green());
        } else {
            // Export as JSON (default)
            let json_filename = if filename.ends_with(".json") {
                filename.to_string()
            } else {
                format!("{}.json", filename)
            };
            
            let export_data = serde_json::json!({
                "provider": self.provider_name,
                "messages": self.conversation_history,
                "timestamp": chrono::Utc::now().to_rfc3339(),
            });

            let mut file = File::create(&json_filename)?;
            file.write_all(serde_json::to_string_pretty(&export_data)?.as_bytes())?;
            
            println!("{}", format!("Conversation exported to: {}", json_filename).green());
        }
        
        Ok(())
    }

    fn show_history(&self) {
        if self.conversation_history.is_empty() {
            println!("{}", "No conversation history yet.".yellow());
            return;
        }

        println!("\n{}", "=== Conversation History ===".cyan().bold());
        for (i, msg) in self.conversation_history.iter().enumerate() {
            let role_display = match msg.role.as_str() {
                "user" => "You".blue().bold(),
                "assistant" => "AI".green().bold(),
                _ => msg.role.as_str().normal(),
            };
            
            println!("\n[{}] {}: {}", i + 1, role_display, msg.content);
        }
        println!();
    }

    fn print_help(&self) {
        println!("\n{}", "=== Zeteo REPL Commands ===".cyan().bold());
        println!();
        println!("{:<20} {}", "/exit, /quit, /q".cyan(), "Exit the REPL");
        println!("{:<20} {}", "/clear".cyan(), "Clear conversation history");
        println!("{:<20} {}", "/help, /h".cyan(), "Show this help message");
        println!("{:<20} {}", "/logs <query>".cyan(), "Search OTEL logs");
        println!("{:<20} {}", "/provider [name]".cyan(), "Show or switch AI provider");
        println!("{:<20} {}", "/export [filename]".cyan(), "Export conversation (json or csv, e.g., /export chat.csv)");
        println!("{:<20} {}", "/history".cyan(), "Show conversation history");
        println!();
        println!("{}", "Just type your message to chat with AI.".dimmed());
        println!();
    }
}

pub async fn create_repl_session(provider_name: Option<String>) -> Result<ReplSession> {
    let provider_name = provider_name.unwrap_or_else(|| "openai".to_string());
    
    let provider: Arc<dyn AiProvider> = match provider_name.to_lowercase().as_str() {
        "openai" => {
            let api_key = std::env::var("OPENAI_API_KEY")
                .map_err(|_| anyhow::anyhow!(
                    "OPENAI_API_KEY not set. Please set it with: export OPENAI_API_KEY=your-key"
                ))?;
            Arc::new(crate::providers::OpenAiProvider::new(api_key, None))
        }
        "vertex" => {
            let project_id = std::env::var("GOOGLE_CLOUD_PROJECT")
                .map_err(|_| anyhow::anyhow!(
                    "GOOGLE_CLOUD_PROJECT not set. Please set it with: export GOOGLE_CLOUD_PROJECT=your-project"
                ))?;
            let location = std::env::var("GOOGLE_CLOUD_LOCATION")
                .unwrap_or_else(|_| "us-central1".to_string());
            Arc::new(crate::providers::VertexProvider::new(project_id, location, None))
        }
        "google" => {
            let api_key = std::env::var("GOOGLE_API_KEY")
                .map_err(|_| anyhow::anyhow!(
                    "GOOGLE_API_KEY not set. Please set it with: export GOOGLE_API_KEY=your-key"
                ))?;
            Arc::new(crate::providers::GoogleProvider::new(api_key, None))
        }
        "azure" => {
            let api_key = std::env::var("AZURE_OPENAI_API_KEY")
                .map_err(|_| anyhow::anyhow!("AZURE_OPENAI_API_KEY not set"))?;
            let endpoint = std::env::var("AZURE_OPENAI_ENDPOINT")
                .map_err(|_| anyhow::anyhow!("AZURE_OPENAI_ENDPOINT not set"))?;
            let deployment = std::env::var("AZURE_OPENAI_DEPLOYMENT")
                .map_err(|_| anyhow::anyhow!("AZURE_OPENAI_DEPLOYMENT not set"))?;
            Arc::new(crate::providers::AzureProvider::new(api_key, endpoint, deployment))
        }
        _ => {
            return Err(anyhow::anyhow!(
                "Unknown provider: {}. Supported: openai, vertex, google, azure",
                provider_name
            ));
        }
    };

    let mut session = ReplSession::new(provider, provider_name.clone());

    // Try to initialize log explorer
    if let Ok(config) = Config::load() {
        if config.servers.contains_key("otel-mcp-server") {
            let explorer = LogExplorer::new("otel-mcp-server".to_string());
            session = session.with_log_explorer(explorer);
        }
    }

    Ok(session)
}
