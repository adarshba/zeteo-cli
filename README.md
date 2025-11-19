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
zeteo-cli logs --interactive
```

### AI Chat

Ask questions about your logs or general queries:
```bash
zeteo-cli chat "What are the most common error patterns?"
```

Specify a provider:
```bash
zeteo-cli chat --provider openai "Analyze the last hour of logs"
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
- [x] OpenAI provider support
- [ ] Full Vertex AI implementation
- [ ] Full Google AI implementation
- [ ] Full Azure OpenAI implementation
- [ ] Real-time log streaming
- [ ] Advanced filtering and aggregation
- [ ] Export functionality
- [ ] Interactive TUI mode

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is inspired by gemini-cli and built for the OpenTelemetry community.

## Acknowledgments

- Similar to [gemini-cli](https://github.com/search?q=gemini-cli)
- Powered by [otel-mcp-server](https://www.npmjs.com/package/otel-mcp-server)
- Built with Rust 🦀