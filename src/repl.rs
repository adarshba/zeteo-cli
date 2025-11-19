use anyhow::Result;
use colored::*;
use dialoguer::{theme::ColorfulTheme, Input};
use std::sync::Arc;
use std::time::Instant;

use crate::config::Config;
use crate::logs::LogExplorer;
use crate::providers::{AiProvider, ChatRequest, Message};

pub struct ReplSession {
    provider: Arc<dyn AiProvider>,
    provider_name: String,
    conversation_history: Vec<Message>,
    log_explorer: Option<LogExplorer>,
    session_start: Instant,
}

impl ReplSession {
    pub fn new(provider: Arc<dyn AiProvider>, provider_name: String) -> Self {
        Self {
            provider,
            provider_name,
            conversation_history: Vec::new(),
            log_explorer: None,
            session_start: Instant::now(),
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
        // Clear screen for clean start
        print!("\x1B[2J\x1B[1;1H");
        
        // Beautiful ASCII art banner
        println!();
        println!("{}", "  ╔═══════════════════════════════════════════════════════════════╗".bright_cyan().bold());
        println!("{}", "  ║                                                               ║".bright_cyan().bold());
        println!("{}", "  ║   ███████╗███████╗████████╗███████╗ ██████╗                 ║".bright_blue().bold());
        println!("{}", "  ║   ╚══███╔╝██╔════╝╚══██╔══╝██╔════╝██╔═══██╗                ║".bright_blue().bold());
        println!("{}", "  ║     ███╔╝ █████╗     ██║   █████╗  ██║   ██║                ║".bright_blue().bold());
        println!("{}", "  ║    ███╔╝  ██╔══╝     ██║   ██╔══╝  ██║   ██║                ║".bright_blue().bold());
        println!("{}", "  ║   ███████╗███████╗   ██║   ███████╗╚██████╔╝                ║".bright_blue().bold());
        println!("{}", "  ║   ╚══════╝╚══════╝   ╚═╝   ╚══════╝ ╚═════╝                 ║".bright_blue().bold());
        println!("{}", "  ║                                                               ║".bright_cyan().bold());
        println!("{}", "  ║        AI-Powered OTEL Log Explorer & Chat Assistant         ║".bright_cyan().bold());
        println!("{}", "  ║                                                               ║".bright_cyan().bold());
        println!("{}", "  ╚═══════════════════════════════════════════════════════════════╝".bright_cyan().bold());
        println!();
        
        // Provider info with icon
        let provider_icon = match self.provider_name.to_lowercase().as_str() {
            "openai" => "🤖",
            "vertex" => "🔷",
            "google" => "🔵",
            "azure" => "☁️",
            _ => "✨",
        };
        
        println!("{} {} {}", 
            "┌─ Provider:".bright_white().bold(),
            provider_icon,
            self.provider_name.bright_green().bold()
        );
        
        if self.log_explorer.is_some() {
            println!("{} {} {}", 
                "└─ Log Explorer:".bright_white().bold(),
                "✓".bright_green(),
                "Connected".bright_green()
            );
        } else {
            println!("{} {} {}", 
                "└─ Log Explorer:".bright_white().bold(),
                "✗".bright_red(),
                "Not configured".dimmed()
            );
        }
        println!();
        
        // Commands section with better formatting
        println!("{}", "╭──────────── Available Commands ────────────╮".bright_yellow().bold());
        println!("{}", "│                                            │".bright_yellow());
        
        let commands = vec![
            ("/exit, /quit, /q", "Exit the REPL", "🚪"),
            ("/clear", "Clear conversation history", "🗑️"),
            ("/help, /h", "Show detailed help", "❓"),
            ("/logs <query>", "Search OTEL logs", "🔍"),
            ("/stats", "Show session statistics", "📊"),
            ("/export [file]", "Export conversation", "💾"),
            ("/history", "Show conversation history", "📜"),
        ];
        
        for (cmd, desc, icon) in commands {
            println!("│  {} {:<18} {} {}",
                icon,
                cmd.bright_cyan(),
                "→".dimmed(),
                desc.bright_white()
            );
        }
        
        println!("{}", "│                                            │".bright_yellow());
        println!("{}", "╰────────────────────────────────────────────╯".bright_yellow().bold());
        println!();
        
        println!("{}", "💡 Tip: Just type your message to start chatting!".bright_magenta().italic());
        println!("{}", "   Press Ctrl+C or type /exit to quit.".dimmed());
        println!();
        println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_black());
        println!();
    }

    fn get_input(&self) -> Result<String> {
        let msg_count = self.conversation_history.len() / 2;
        let prompt = format!("{} [{}]> ", 
            "zeteo".bright_cyan().bold(),
            msg_count.to_string().bright_black()
        );
        
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
                self.print_goodbye();
                return Ok(true);
            }
            "/clear" => {
                self.conversation_history.clear();
                println!();
                println!("{} {}", "✓".bright_green().bold(), "Conversation history cleared.".bright_green());
                println!();
            }
            "/help" | "/h" => {
                self.print_help();
            }
            "/logs" => {
                if parts.len() > 1 {
                    let query = parts[1..].join(" ");
                    self.handle_logs_command(&query).await?;
                } else {
                    println!();
                    println!("{} {}", "⚠".bright_yellow(), "Usage: /logs <query>".yellow());
                    println!("{} {}", "  Example:".dimmed(), "/logs error".bright_cyan());
                    println!();
                }
            }
            "/provider" => {
                if parts.len() > 1 {
                    println!();
                    println!("{} {}", "ℹ".bright_blue(), "Provider switching not yet implemented in current session.".yellow());
                    println!("{} {}", "  Tip:".dimmed(), "Restart zeteo with --provider flag.".dimmed());
                    println!();
                } else {
                    self.show_provider_info();
                }
            }
            "/export" => {
                let filename = parts.get(1).copied();
                self.export_conversation(filename)?;
            }
            "/history" => {
                self.show_history();
            }
            "/stats" => {
                self.show_statistics();
            }
            _ => {
                println!();
                println!("{} {} {}", "❌".red(), "Unknown command:".red().bold(), command.bright_red());
                println!("{} {}", "  Tip:".dimmed(), "Type /help for available commands".dimmed());
                println!();
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

        // Show animated thinking indicator
        let start_time = Instant::now();
        self.show_thinking_indicator();

        // Get response from AI
        let response = self.provider.chat(request).await?;
        let elapsed = start_time.elapsed();

        // Add assistant response to history
        let assistant_message = Message {
            role: "assistant".to_string(),
            content: response.content.clone(),
        };
        self.conversation_history.push(assistant_message);

        // Display response with formatting
        println!();
        println!("{}", "┌─ AI Response ─────────────────────────────────────".bright_green());
        self.display_formatted_response(&response.content);
        println!("{}", "└───────────────────────────────────────────────────".bright_green());
        println!();
        println!("{} {:.2}s", "⏱  Response time:".dimmed(), elapsed.as_secs_f64());
        println!();

        Ok(())
    }

    async fn handle_logs_command(&self, query: &str) -> Result<()> {
        if let Some(explorer) = &self.log_explorer {
            println!();
            println!("{} {}", "🔍 Searching logs for:".bright_cyan(), query.bright_yellow());
            let logs = explorer.search_logs(query, 20).await?;
            
            if logs.is_empty() {
                println!();
                println!("{} {}", "⚠".bright_yellow(), "No logs found.".yellow());
                println!();
            } else {
                explorer.display_logs(&logs);
            }
        } else {
            println!();
            println!("{} {}", "⚠".bright_yellow(), "Log explorer not available.".yellow());
            println!("{} {}", "  Tip:".dimmed(), "Configure MCP server in config.json".dimmed());
            println!();
        }
        Ok(())
    }

    fn export_conversation(&self, filename: Option<&str>) -> Result<()> {
        use std::fs::File;
        use std::io::Write;

        let filename = filename.unwrap_or("conversation.json");
        
        println!();
        if filename.ends_with(".csv") {
            // Export as CSV
            let mut file = File::create(filename)?;
            writeln!(file, "role,content")?;
            
            for msg in &self.conversation_history {
                let content = msg.content.replace(",", ";").replace("\n", " ");
                writeln!(file, "{},{}", msg.role, content)?;
            }
            
            println!("{} {} {}", "✓".bright_green().bold(), "Conversation exported to:".bright_green(), filename.bright_cyan());
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
                "message_count": self.conversation_history.len(),
                "timestamp": chrono::Utc::now().to_rfc3339(),
                "session_duration_seconds": self.session_start.elapsed().as_secs(),
            });

            let mut file = File::create(&json_filename)?;
            file.write_all(serde_json::to_string_pretty(&export_data)?.as_bytes())?;
            
            println!("{} {} {}", "✓".bright_green().bold(), "Conversation exported to:".bright_green(), json_filename.bright_cyan());
        }
        println!();
        
        Ok(())
    }

    fn show_history(&self) {
        if self.conversation_history.is_empty() {
            println!();
            println!("{} {}", "ℹ".bright_blue(), "No conversation history yet.".yellow());
            println!();
            return;
        }

        println!();
        println!("{}", "╭────────── Conversation History ──────────╮".bright_cyan().bold());
        println!("{}", "│                                          │".bright_cyan());
        
        for (i, msg) in self.conversation_history.iter().enumerate() {
            let (role_display, icon) = match msg.role.as_str() {
                "user" => ("You".bright_blue().bold(), "👤"),
                "assistant" => ("AI".bright_green().bold(), "🤖"),
                _ => (msg.role.as_str().normal(), "•"),
            };
            
            println!("│ {} [{}] {}:", icon, (i / 2) + 1, role_display);
            
            // Truncate long messages for history display
            let content = if msg.content.len() > 60 {
                format!("{}...", &msg.content[..60])
            } else {
                msg.content.clone()
            };
            
            for line in content.lines().take(2) {
                println!("│   {}", line.dimmed());
            }
            
            if i < self.conversation_history.len() - 1 {
                println!("{}", "│                                          │".bright_cyan());
            }
        }
        
        println!("{}", "│                                          │".bright_cyan());
        println!("{}", "╰──────────────────────────────────────────╯".bright_cyan().bold());
        println!();
        println!("{} {}", "💡 Tip:".bright_magenta(), "Use /export to save full conversation".dimmed());
        println!();
    }

    fn print_help(&self) {
        println!();
        println!("{}", "╔══════════════════════════════════════════════════════════════╗".bright_cyan().bold());
        println!("{}", "║                  Zeteo REPL Commands                         ║".bright_cyan().bold());
        println!("{}", "╚══════════════════════════════════════════════════════════════╝".bright_cyan().bold());
        println!();
        
        let commands = vec![
            ("🚪", "/exit, /quit, /q", "Exit the REPL and end session"),
            ("🗑️", "/clear", "Clear conversation history to start fresh"),
            ("❓", "/help, /h", "Show this detailed help message"),
            ("🔍", "/logs <query>", "Search OTEL logs (e.g., /logs error)"),
            ("🔄", "/provider", "Show current AI provider info"),
            ("📊", "/stats", "Display session statistics"),
            ("💾", "/export [file]", "Export conversation (json/csv)"),
            ("📜", "/history", "Show conversation history summary"),
        ];
        
        for (icon, cmd, desc) in commands {
            println!("  {} {:<20} {}", 
                icon,
                cmd.bright_cyan().bold(),
                desc.bright_white()
            );
        }
        
        println!();
        println!("{}", "╭──────────────────────────────────────────────────────────────╮".bright_yellow());
        println!("{}", "│  💡 Tips & Tricks                                            │".bright_yellow());
        println!("{}", "├──────────────────────────────────────────────────────────────┤".bright_yellow());
        println!("{}", "│  • Just type your message to chat with AI                   │".bright_white());
        println!("{}", "│  • Use multi-line input with Shift+Enter (if supported)     │".bright_white());
        println!("{}", "│  • Export conversations for sharing with your team          │".bright_white());
        println!("{}", "│  • Check /stats to see your session activity                │".bright_white());
        println!("{}", "╰──────────────────────────────────────────────────────────────╯".bright_yellow());
        println!();
    }
    
    fn show_provider_info(&self) {
        println!();
        println!("{}", "╭──────── Provider Information ────────╮".bright_cyan().bold());
        println!("{}", "│                                      │".bright_cyan());
        
        let provider_icon = match self.provider_name.to_lowercase().as_str() {
            "openai" => "🤖",
            "vertex" => "🔷",
            "google" => "🔵",
            "azure" => "☁️",
            _ => "✨",
        };
        
        println!("│  {} Name: {:<24} │", 
            provider_icon,
            self.provider_name.bright_green().bold()
        );
        
        let model_info = match self.provider_name.to_lowercase().as_str() {
            "openai" => "GPT-4o / GPT-4",
            "vertex" => "Gemini Pro (GCP)",
            "google" => "Gemini Pro",
            "azure" => "Azure OpenAI",
            _ => "Unknown",
        };
        
        println!("│  📋 Model: {:<24} │", model_info.bright_white());
        println!("{}", "│                                      │".bright_cyan());
        println!("{}", "╰──────────────────────────────────────╯".bright_cyan().bold());
        println!();
    }
    
    fn show_statistics(&self) {
        let duration = self.session_start.elapsed();
        let hours = duration.as_secs() / 3600;
        let minutes = (duration.as_secs() % 3600) / 60;
        let seconds = duration.as_secs() % 60;
        
        let message_pairs = self.conversation_history.len() / 2;
        
        println!();
        println!("{}", "╔══════════════════════════════════════════════════╗".bright_magenta().bold());
        println!("{}", "║          Session Statistics                      ║".bright_magenta().bold());
        println!("{}", "╚══════════════════════════════════════════════════╝".bright_magenta().bold());
        println!();
        
        println!("  {} {:<30} {}", 
            "💬".bright_cyan(),
            "Total messages exchanged:",
            format!("{}", message_pairs).bright_yellow().bold()
        );
        
        println!("  {} {:<30} {}", 
            "📝".bright_cyan(),
            "Messages in history:",
            format!("{}", self.conversation_history.len()).bright_yellow().bold()
        );
        
        println!("  {} {:<30} {}", 
            "⏱".bright_cyan(),
            "Session duration:",
            format!("{}h {}m {}s", hours, minutes, seconds).bright_yellow().bold()
        );
        
        println!("  {} {:<30} {}", 
            "🤖".bright_cyan(),
            "AI Provider:",
            self.provider_name.bright_green().bold()
        );
        
        if self.log_explorer.is_some() {
            println!("  {} {:<30} {}", 
                "🔍".bright_cyan(),
                "Log Explorer:",
                "Connected ✓".bright_green().bold()
            );
        }
        
        println!();
        
        if message_pairs > 0 {
            let avg_time_per_msg = duration.as_secs() as f64 / message_pairs as f64;
            println!("  {} {:.1}s", 
                "📊 Average time per exchange:".dimmed(),
                avg_time_per_msg
            );
            println!();
        }
    }
    
    fn show_thinking_indicator(&self) {
        print!("{} ", "💭 Thinking...".bright_magenta().bold());
        std::io::Write::flush(&mut std::io::stdout()).ok();
        print!("\r");
    }
    
    fn display_formatted_response(&self, content: &str) {
        // Simple formatting with color coding
        for line in content.lines() {
            if line.trim().starts_with("```") {
                // Code block delimiter
                println!("{}", line.bright_black());
            } else if line.trim().starts_with('#') {
                // Heading
                println!("{}", line.bright_yellow().bold());
            } else if line.trim().starts_with("- ") || line.trim().starts_with("* ") {
                // List item
                println!("  {}", line.bright_cyan());
            } else if line.trim().starts_with(&['1', '2', '3', '4', '5', '6', '7', '8', '9'][..]) 
                && line.contains(". ") {
                // Numbered list
                println!("  {}", line.bright_cyan());
            } else {
                // Regular text
                println!("{}", line.bright_white());
            }
        }
    }
    
    fn print_goodbye(&self) {
        let duration = self.session_start.elapsed();
        let minutes = duration.as_secs() / 60;
        let message_pairs = self.conversation_history.len() / 2;
        
        println!();
        println!("{}", "╔═══════════════════════════════════════════════════════════╗".bright_cyan().bold());
        println!("{}", "║                 Thank You for Using Zeteo!               ║".bright_cyan().bold());
        println!("{}", "╚═══════════════════════════════════════════════════════════╝".bright_cyan().bold());
        println!();
        
        println!("{}", "📊 Session Summary:".bright_magenta().bold());
        println!("   {} messages exchanged in {} minutes", 
            message_pairs.to_string().bright_yellow(),
            minutes.to_string().bright_yellow()
        );
        
        if message_pairs > 0 && !self.conversation_history.is_empty() {
            println!();
            println!("{} {}", "💡 Tip:".bright_blue(), "Don't forget to export your conversation with /export".dimmed());
        }
        
        println!();
        println!("{}", "👋 Goodbye!".bright_green().bold());
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
