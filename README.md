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
- [ ] Real-time log streaming
- [ ] Advanced filtering and aggregation
- [ ] Export functionality (CSV, JSON files)
- [ ] Interactive TUI mode
- [ ] Response caching for better performance
- [ ] Retry logic with exponential backoff

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is inspired by gemini-cli and built for the OpenTelemetry community.

## Acknowledgments

- Similar to [gemini-cli](https://github.com/search?q=gemini-cli)
- Powered by [otel-mcp-server](https://www.npmjs.com/package/otel-mcp-server)
- Built with Rust 🦀