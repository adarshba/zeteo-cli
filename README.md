# zeteo-cli

A Rust-based CLI AI agent with OTEL log exploration capabilities. Zeteo (from Greek, meaning "to seek") is a powerful command-line tool for exploring and analyzing OpenTelemetry logs with AI-powered assistance.

## Features

- 🔍 **Log Explorer**: Search and explore OTEL-based logs from OpenObserve, Kibana, and Elasticsearch
- 🤖 **AI Integration**: Chat with multiple AI providers (OpenAI, Vertex AI, Google AI, Azure OpenAI)
- 📊 **MCP Server Support**: Integrated with otel-mcp-server for seamless log queries
- 🎨 **Beautiful CLI**: Colored output with interactive modes

## Installation

### Prerequisites

- Rust 1.70 or later
- Node.js (for otel-mcp-server)

### Build from Source

```bash
git clone https://github.com/adarshba/zeteo-cli
cd zeteo-cli
cargo build --release
```

The binary will be available at `target/release/zeteo-cli`.

## Configuration

On first run, Zeteo creates a configuration file at `~/.config/zeteo-cli/config.json` with default settings for the otel-mcp-server:

```json
{
  "servers": {
    "otel-mcp-server": {
      "command": "npx",
      "args": ["-y", "otel-mcp-server"],
      "env": {
        "ELASTICSEARCH_URL": "http://localhost:9200",
        "ELASTICSEARCH_USERNAME": "elastic",
        "ELASTICSEARCH_PASSWORD": "changeme",
        "SERVER_NAME": "otel-mcp-server",
        "LOGLEVEL": "OFF"
      }
    }
  }
}
```

### Environment Variables

For AI features, set the appropriate API key:
- `OPENAI_API_KEY`: For OpenAI integration
- Additional providers (Vertex, Google AI, Azure) coming soon

## Usage

### View Configuration

```bash
zeteo-cli config --show
```

### Log Exploration

Search logs with a query:
```bash
zeteo-cli logs --query "error" --max 50
```

Interactive mode:
```bash
zeteo logs --interactive
```

JSON output for scripting:
```bash
zeteo logs --query "error" --output json | jq '.[] | select(.level=="ERROR")'
```

### AI Chat

Zeteo supports multiple AI providers. Configure them using environment variables:

#### OpenAI
```bash
export OPENAI_API_KEY="your-key-here"
zeteo chat "What are the most common error patterns?"
zeteo chat --provider openai "Analyze these logs"
```

#### Google AI (Gemini)
```bash
export GOOGLE_API_KEY="your-key-here"
zeteo chat --provider google "Explain OTEL log structure"
```

#### Vertex AI
```bash
export GOOGLE_CLOUD_PROJECT="your-project-id"
export GOOGLE_CLOUD_LOCATION="us-central1"  # optional, defaults to us-central1
gcloud auth application-default login
zeteo chat --provider vertex "Help me debug this issue"
```

#### Azure OpenAI
```bash
export AZURE_OPENAI_API_KEY="your-key"
export AZURE_OPENAI_ENDPOINT="https://your-resource.openai.azure.com"
export AZURE_OPENAI_DEPLOYMENT="your-deployment-name"
zeteo chat --provider azure "Summarize these errors"
```

Get JSON output:
```bash
zeteo chat --output json "What is OpenTelemetry?" | jq '.content'
```

### Shell Completions

Generate completions for your shell:

```bash
# Bash
zeteo completions bash > /etc/bash_completion.d/zeteo

# Zsh
zeteo completions zsh > ~/.zsh/completion/_zeteo

# Fish
zeteo completions fish > ~/.config/fish/completions/zeteo.fish

# PowerShell
zeteo completions powershell > ~/.config/powershell/zeteo.ps1
```

## MCP Server Integration

Zeteo integrates with the [otel-mcp-server](https://www.npmjs.com/package/otel-mcp-server) to query OpenTelemetry logs from various backends:

- Elasticsearch
- OpenObserve
- Kibana

The MCP server is automatically configured and managed by Zeteo.

## Development

### Project Structure

```
zeteo-cli/
├── src/
│   ├── main.rs           # CLI entry point
│   ├── config/           # Configuration management
│   ├── mcp/              # MCP client integration
│   ├── providers/        # AI provider implementations
│   └── logs/             # Log exploration logic
├── Cargo.toml
└── README.md
```

### Building

```bash
cargo build
```

### Running Tests

```bash
cargo test
```

### Linting

```bash
cargo clippy
```

## 🎨 Enhanced Interactive REPL Mode

Zeteo features a beautifully redesigned interactive REPL (Read-Eval-Print Loop) shell with a focus on visual appeal and user experience. The REPL is the main product of Zeteo, offering continuous conversational interaction with AI while exploring OTEL logs.

```bash
# Start interactive mode (default when no command specified)
zeteo

# Or specify a provider
zeteo --provider google
zeteo --provider vertex
zeteo --provider azure
```

### ✨ Visual Enhancements

- **🎨 Beautiful ASCII Art Banner**: Eye-catching ZETEO logo on startup
- **🤖 Provider Icons**: Emoji indicators (🤖 OpenAI, 🔷 Vertex, 🔵 Google, ☁️ Azure)
- **🌈 Rich Color Scheme**: Intelligent color coding for different output types
- **📊 Professional Layout**: Clean borders, dividers, and formatting
- **🔢 Message Counter**: Track conversation depth in the prompt

### 🚀 Key Features

- **💬 Continuous conversation**: Maintains context across multiple messages
- **📊 Session Statistics**: Track messages, duration, and performance with `/stats`
- **⏱️ Response Timing**: See how long each AI response takes
- **🎯 Smart Indicators**: Visual feedback with ✓, ⚠, ❌, and ℹ icons
- **💾 Export Conversations**: Save to JSON or CSV with `/export`
- **📜 History Management**: View conversation history with `/history`
- **🔍 Log Integration**: Search OTEL logs directly with `/logs`

### 📋 REPL Commands

- `/exit`, `/quit`, `/q` 🚪 - Exit with session summary
- `/clear` 🗑️ - Clear conversation history
- `/help` ❓ - Show detailed help
- `/stats` 📊 - Show session statistics (NEW!)
- `/logs <query>` 🔍 - Search OTEL logs
- `/provider` 🔄 - Show provider info (ENHANCED!)
- `/export [file]` 💾 - Export conversation (ENHANCED!)
- `/history` 📜 - Show conversation history (ENHANCED!)

### 🎬 Example Session

```bash
$ export OPENAI_API_KEY="your-key"
$ zeteo

  ╔═══════════════════════════════════════════════════════════════╗
  ║   ███████╗███████╗████████╗███████╗ ██████╗                 ║
  ║   ╚══███╔╝██╔════╝╚══██╔══╝██╔════╝██╔═══██╗                ║
  ║     ███╔╝ █████╗     ║   █████╗  ██║   ██║                ║
  ║    ███╔╝  ██╔══╝     ██║   ██╔══╝  ██║   ██║                ║
  ║   ███████╗███████╗   ██║   ███████╗╚██████╔╝                ║
  ║   ╚══════╝╚══════╝   ╚═╝   ╚══════╝ ╚═════╝                 ║
  ║        AI-Powered OTEL Log Explorer & Chat Assistant         ║
  ╚═══════════════════════════════════════════════════════════════╝

┌─ Provider: 🤖 openai
└─ Log Explorer: ✓ Connected

💡 Tip: Just type your message to start chatting!

zeteo [0]> What is OpenTelemetry?

💭 Thinking...

┌─ AI Response ─────────────────────────────────────
OpenTelemetry is an observability framework...
└───────────────────────────────────────────────────

⏱  Response time: 1.23s

zeteo [1]> /stats

╔══════════════════════════════════════════════════╗
║          Session Statistics                      ║
╚══════════════════════════════════════════════════╝

  💬 Total messages exchanged:     1
  📝 Messages in history:          2
  ⏱  Session duration:             0h 1m 5s
  🤖 AI Provider:                  openai
  🔍 Log Explorer:                 Connected ✓

zeteo [1]> /export my-conversation.json

✓ Conversation exported to: my-conversation.json

zeteo [1]> /exit

╔═══════════════════════════════════════════════════════════╗
║                 Thank You for Using Zeteo!               ║
╚═══════════════════════════════════════════════════════════╝

📊 Session Summary: 1 messages exchanged in 1 minutes
👋 Goodbye!
```

See [examples/REPL_GUIDE.md](examples/REPL_GUIDE.md) for more detailed examples and tips.


## Roadmap

- [x] Basic CLI structure
- [x] MCP server integration
- [x] OpenAI provider support (fully implemented)
- [x] Full Vertex AI implementation (with gcloud authentication)
- [x] Full Google AI implementation (Gemini API)
- [x] Full Azure OpenAI implementation
- [x] Shell completions (bash, zsh, fish, powershell)
- [x] JSON output format for scripting
- [x] Graceful shutdown handling
- [x] **Interactive REPL mode** ✨
- [x] **Enhanced REPL UI with beautiful colors and formatting** 🎨 (NEW!)
- [x] **Session statistics and tracking** 📊 (NEW!)
- [x] **Response time monitoring** ⏱️ (NEW!)
- [x] **Improved command help and documentation** 📚 (NEW!)
- [x] **Real-time log streaming**
- [x] **Advanced filtering and aggregation**
- [x] **Export functionality (CSV, JSON files)**
- [x] **Response caching for better performance**
- [x] **Retry logic with exponential backoff**
- [ ] Interactive TUI mode with full terminal UI
- [ ] Full MCP client implementation

**Note**: The REPL mode is now the flagship feature with a complete visual overhaul and stabilization!

## Advanced Features

### Log Filtering and Aggregation

```bash
# Filter by log level
zeteo logs --query "error" --level ERROR

# Filter by service
zeteo logs --query "*" --service "api-gateway"

# Show aggregated statistics
zeteo logs --query "error" --aggregate

# Combine filters
zeteo logs --query "database" --level WARN --service "backend" --aggregate
```

### Export Logs

```bash
# Export to JSON
zeteo logs --query "error" --export logs.json

# Export to CSV
zeteo logs --query "error" --export logs.csv
```

### Real-time Log Streaming

```bash
# Stream logs in real-time
zeteo logs --query "*" --stream

# Stream with filters
zeteo logs --query "error" --level ERROR --stream
```

### Conversation Export

Within REPL mode:
```bash
# Export as JSON (default)
zeteo> /export my-chat.json

# Export as CSV
zeteo> /export my-chat.csv
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is inspired by gemini-cli and built for the OpenTelemetry community.

## Acknowledgments

- Similar to [gemini-cli](https://github.com/search?q=gemini-cli)
- Powered by [otel-mcp-server](https://www.npmjs.com/package/otel-mcp-server)
- Built with Rust 🦀