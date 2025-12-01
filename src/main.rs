use anyhow::Result;
use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::{generate, Shell};
use std::io;

mod backends;
mod cache;
mod config;
mod logs;
mod mcp;
mod providers;
mod retry;
mod session;
mod tools;
mod tui;

#[derive(Parser)]
#[command(name = "zeteo")]
#[command(author, version, about = "AI assistant with log analysis", long_about = None)]
struct Cli {
    #[arg(short, long, global = true)]
    provider: Option<String>,
    
    /// Backend to use for log queries (kibana, openobserve)
    #[arg(short, long, global = true)]
    backend: Option<String>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Completions {
        #[arg(value_enum)]
        shell: Shell,
    },
    Version,
}

#[tokio::main]
async fn main() -> Result<()> {
    let _ = dotenv::dotenv();
    
    let cli = Cli::parse();
    
    match cli.command {
        Some(Commands::Completions { shell }) => {
            let mut cmd = Cli::command();
            generate(shell, &mut cmd, "zeteo", &mut io::stdout());
        }
        Some(Commands::Version) => {
            println!("zeteo {}", env!("CARGO_PKG_VERSION"));
        }
        None => {
            let mut app = tui::create_tui_session(cli.provider, cli.backend).await?;
            app.run().await?;
        }
    }
    
    Ok(())
}
