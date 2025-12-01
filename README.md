# Zeteo

A terminal-based AI assistant with log analysis capabilities, built with Rust.

## Overview

Zeteo is a command-line AI assistant that combines conversational AI with observability log analysis. It provides a responsive terminal user interface with support for multiple AI providers and log backends, allowing you to query and analyze logs using natural language.

## Demo

<p align="center">
  <img src="assets/demo.gif" alt="Zeteo Demo" width="800">
</p>

*Zeteo in action: querying logs and analyzing errors using natural language*

> **Note**: To generate the demo GIF, run `vhs demo.tape` (requires [vhs](https://github.com/charmbracelet/vhs))

## Features

- **Multiple AI Providers**: OpenAI, Google AI, Azure OpenAI, and Vertex AI
- **Log Analysis**: Query and analyze logs from Kibana, OpenObserve, and Elasticsearch
- **Tool Calling**: AI can automatically query logs to answer your questions
- **Rich TUI**: Markdown rendering, slash commands, and keyboard navigation
- **Configurable**: Flexible configuration for backends and providers

## Prerequisites

- **Rust 1.70 or later** - Install from [rustup.rs](https://rustup.rs)
- **An AI Provider API Key** - At least one of:
  - OpenAI API key
  - Google AI API key
  - Azure OpenAI credentials
  - Google Cloud project (for Vertex AI)
- **Log Backend (optional)** - For log analysis features:
  - Kibana instance
  - OpenObserve instance
  - Elasticsearch cluster

## Installation

### From crates.io (Recommended)

```bash
cargo install zeteo
```

To upgrade to the latest version:

```bash
cargo install zeteo --force
```

### From Source

1. Clone the repository:
   ```bash
   git clone https://github.com/adarshba/zeteo-cli.git
   cd zeteo-cli
   ```

2. Build and install:
   ```bash
   cargo install --path .
   ```

   Or build without installing:
   ```bash
   cargo build --release
   ```

   The binary will be available at `target/release/zeteo`.

## Configuration

Zeteo uses two configuration methods:
1. **Environment variables** - For AI provider credentials
2. **config.json** - For log backend settings

### Step 1: Set Up AI Provider Credentials

Create a `.env` file in the project directory (or export environment variables):

```bash
cp .env.example .env
```

Then edit `.env` with your credentials:

#### OpenAI

```bash
OPENAI_API_KEY=sk-your-api-key-here
```

#### Google AI

```bash
GOOGLE_API_KEY=AIza-your-api-key-here
```

#### Azure OpenAI

```bash
AZURE_OPENAI_API_KEY=your-api-key
AZURE_OPENAI_ENDPOINT=https://your-resource.openai.azure.com
AZURE_OPENAI_DEPLOYMENT=your-deployment-name
```

#### Vertex AI

```bash
GOOGLE_CLOUD_PROJECT=your-project-id
GOOGLE_CLOUD_LOCATION=us-central1
```

### Step 2: Configure Log Backends (Optional)

Create a `config.json` file to enable log analysis features. The file can be placed in:
- Current directory (`./config.json`) - **recommended**
- Next to the executable
- Global config directory (`~/.config/zeteo-cli/config.json`)

Start with the example configuration:

```bash
cp config.example.json config.json
```

Then edit `config.json` with your backend settings:

```json
{
  "servers": {},
  "backends": {
    "kibana": {
      "type": "kibana",
      "url": "http://localhost:5601",
      "auth_token": null,
      "index_pattern": "logs-*",
      "verify_ssl": false,
      "version": "7.10.2"
    },
    "openobserve": {
      "type": "openobserve",
      "url": "http://localhost:5080",
      "username": "admin@example.com",
      "password": "changeme",
      "organization": "default",
      "stream": "default",
      "verify_ssl": false
    },
    "elasticsearch": {
      "type": "elasticsearch",
      "url": "http://localhost:9200",
      "username": "elastic",
      "password": "changeme",
      "index_pattern": "logs-*",
      "verify_ssl": false
    }
  }
}
```

#### Backend Configuration Options

**Kibana Backend**
| Field | Description | Required |
|-------|-------------|----------|
| `type` | Must be `"kibana"` | Yes |
| `url` | Kibana server URL | Yes |
| `auth_token` | JWT or API token for authentication | No |
| `index_pattern` | Elasticsearch index pattern | No (default: `logs-*`) |
| `verify_ssl` | Verify SSL certificates | No (default: `false`) |
| `version` | Kibana version | No (default: `7.10.2`) |

**OpenObserve Backend**
| Field | Description | Required |
|-------|-------------|----------|
| `type` | Must be `"openobserve"` | Yes |
| `url` | OpenObserve server URL | Yes |
| `username` | OpenObserve username | Yes |
| `password` | OpenObserve password | Yes |
| `organization` | Organization name | No (default: `default`) |
| `stream` | Stream name | No (default: `default`) |
| `verify_ssl` | Verify SSL certificates | No (default: `false`) |

**Elasticsearch Backend**
| Field | Description | Required |
|-------|-------------|----------|
| `type` | Must be `"elasticsearch"` | Yes |
| `url` | Elasticsearch cluster URL | Yes |
| `username` | Elasticsearch username | No |
| `password` | Elasticsearch password | No |
| `index_pattern` | Index pattern to query | No (default: `logs-*`) |
| `verify_ssl` | Verify SSL certificates | No (default: `false`) |

## Usage

### Starting the Application

```bash
zeteo
```

Zeteo will automatically detect configured providers and backends.

### Specifying Provider and Backend

```bash
# Use a specific AI provider
zeteo --provider openai
zeteo --provider google
zeteo --provider azure
zeteo --provider vertex

# Use a specific log backend
zeteo --backend kibana
zeteo --backend openobserve

# Combine both
zeteo --provider openai --backend kibana
```

### Shell Completions

```bash
# Bash
zeteo completions bash > ~/.bash_completion.d/zeteo

# Zsh
zeteo completions zsh > ~/.zfunc/_zeteo
```

## Keyboard Controls

| Key | Action |
|-----|--------|
| Enter | Send message |
| Esc | Exit application |
| Up/Down | Scroll through messages |
| Page Up/Down | Scroll by page |
| Left/Right | Move cursor in input |
| Home/End | Jump to start/end of input |
| Ctrl+C | Force exit |

## Slash Commands

Type `/` to see available commands:

| Command | Shortcut | Description |
|---------|----------|-------------|
| `/help` | `/h` | Show available commands |
| `/quit` | `/q` | Exit application |
| `/clear` | `/c` | Clear chat history |
| `/backend` | `/b` | Switch log backend |

### Switching Backends

```
/backend           # List available backends
/backend kibana    # Switch to kibana backend
/backend openobserve # Switch to openobserve backend
```

## Log Analysis Examples

Once configured with a log backend, you can ask questions like:

- "Show me the last 10 errors"
- "Find all payment failures in the last hour"
- "What services had errors today?"
- "Search for timeout errors in the auth service"
- "Get log statistics for the past 30 minutes"

Zeteo will automatically query your log backend and analyze the results.

## Development

### Build the Project

```bash
cargo build
```

### Run Tests

```bash
cargo test
```

### Run Linting

```bash
cargo clippy
cargo fmt --check
```

### Project Structure

```
zeteo-cli/
├── src/
│   ├── main.rs           # CLI entry point
│   ├── tui.rs            # Terminal UI
│   ├── backends/         # Log backend clients
│   │   ├── elasticsearch.rs
│   │   ├── kibana.rs
│   │   └── openobserve.rs
│   ├── providers/        # AI provider clients
│   │   ├── openai.rs
│   │   ├── google.rs
│   │   ├── azure.rs
│   │   └── vertex.rs
│   ├── config/           # Configuration management
│   └── tools/            # Tool execution for AI
├── config.json           # Backend configuration
├── config.example.json   # Example configuration
└── .env.example          # Example environment variables
```

## Troubleshooting

### "No AI provider configured"

Ensure you have set at least one provider's environment variables. Check with:

```bash
echo $OPENAI_API_KEY
# or
cat .env
```

### "No log backend configured"

Create a `config.json` file with your backend settings. The AI will still work for general questions, but log analysis features will be disabled.

### Connection errors to backends

- Verify the backend URL is accessible
- Check authentication credentials
- If using HTTPS with self-signed certificates, set `verify_ssl` to `false`

## License

MIT License. See [LICENSE](LICENSE) for details.
