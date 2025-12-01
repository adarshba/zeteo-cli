# Zeteo

[![Crates.io](https://img.shields.io/crates/v/zeteo.svg)](https://crates.io/crates/zeteo)
[![Downloads](https://img.shields.io/crates/d/zeteo.svg)](https://crates.io/crates/zeteo)
[![License](https://img.shields.io/crates/l/zeteo.svg)](LICENSE)
[![CI](https://github.com/adarshba/zeteo-cli/actions/workflows/ci.yml/badge.svg)](https://github.com/adarshba/zeteo-cli/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/adarshba/zeteo-cli)](https://github.com/adarshba/zeteo-cli/releases/latest)

A terminal-based AI assistant with observability log analysis capabilities, built in Rust.

## Overview

Zeteo is a command-line AI assistant that combines conversational AI with observability log analysis. It provides a responsive terminal user interface with support for multiple AI providers and log backends, enabling natural language queries against your logs.

## Features

- **Multiple AI Providers** - OpenAI, Google AI, Azure OpenAI, and Vertex AI
- **Log Backends** - Query logs from Kibana, OpenObserve, and Elasticsearch
- **Tool Calling** - AI automatically queries logs to answer your questions
- **Rich TUI** - Markdown rendering, slash commands, and keyboard navigation
- **Configurable** - Flexible configuration for backends and providers

## Installation

### From crates.io

```bash
cargo install zeteo
```

### From GitHub Releases

Download pre-built binaries from the [releases page](https://github.com/adarshba/zeteo-cli/releases/latest).

Available for:

- Linux (x86_64, aarch64)
- macOS (x86_64, Apple Silicon)
- Windows (x86_64)

### From Source

```bash
git clone https://github.com/adarshba/zeteo-cli.git
cd zeteo-cli
cargo install --path .
```

## Quick Start

1. Set up an AI provider (at least one required):

```bash
# OpenAI
export OPENAI_API_KEY=sk-your-api-key

# Google AI
export GOOGLE_API_KEY=AIza-your-api-key

# Azure OpenAI
export AZURE_OPENAI_API_KEY=your-api-key
export AZURE_OPENAI_ENDPOINT=https://your-resource.openai.azure.com
export AZURE_OPENAI_DEPLOYMENT=your-deployment-name
```

2. Run zeteo:

```bash
zeteo
```

3. (Optional) Configure a log backend for log analysis:

```bash
cp config.example.json config.json
# Edit config.json with your backend settings
```

## Configuration

### AI Providers

Set credentials via environment variables or a `.env` file:

| Provider     | Required Variables                                                         |
| ------------ | -------------------------------------------------------------------------- |
| OpenAI       | `OPENAI_API_KEY`                                                           |
| Google AI    | `GOOGLE_API_KEY`                                                           |
| Azure OpenAI | `AZURE_OPENAI_API_KEY`, `AZURE_OPENAI_ENDPOINT`, `AZURE_OPENAI_DEPLOYMENT` |
| Vertex AI    | `GOOGLE_CLOUD_PROJECT`, `GOOGLE_CLOUD_LOCATION`                            |

### Log Backends

Configure backends in `config.json`. The config file is searched in the following order:

1. `./config.json` (current directory)
2. Next to the zeteo executable
3. `~/.config/zeteo/config.json` (Linux/macOS) or `%APPDATA%\zeteo\config.json` (Windows)

```json
{
  "backends": {
    "kibana": {
      "type": "kibana",
      "url": "http://localhost:5601",
      "index_pattern": "logs-*"
    },
    "openobserve": {
      "type": "openobserve",
      "url": "http://localhost:5080",
      "username": "admin@example.com",
      "password": "changeme",
      "organization": "default",
      "stream": "default"
    },
    "elasticsearch": {
      "type": "elasticsearch",
      "url": "http://localhost:9200",
      "index_pattern": "logs-*"
    }
  }
}
```

See [config.example.json](config.example.json) for a complete example.

### Backend Configuration Options

**Kibana**

| Field           | Description                                     | Required |
| --------------- | ----------------------------------------------- | -------- |
| `type`          | Must be `"kibana"`                              | Yes      |
| `url`           | Kibana server URL                               | Yes      |
| `auth_token`    | JWT or API token for authentication             | No       |
| `index_pattern` | Elasticsearch index pattern (default: `logs-*`) | No       |
| `verify_ssl`    | Verify SSL certificates (default: `false`)      | No       |
| `version`       | Kibana version (default: `7.10.2`)              | No       |

**OpenObserve**

| Field          | Description                                | Required |
| -------------- | ------------------------------------------ | -------- |
| `type`         | Must be `"openobserve"`                    | Yes      |
| `url`          | OpenObserve server URL                     | Yes      |
| `username`     | OpenObserve username                       | Yes      |
| `password`     | OpenObserve password                       | Yes      |
| `organization` | Organization name (default: `default`)     | No       |
| `stream`       | Stream name (default: `default`)           | No       |
| `verify_ssl`   | Verify SSL certificates (default: `false`) | No       |

**Elasticsearch**

| Field           | Description                                | Required |
| --------------- | ------------------------------------------ | -------- |
| `type`          | Must be `"elasticsearch"`                  | Yes      |
| `url`           | Elasticsearch cluster URL                  | Yes      |
| `username`      | Elasticsearch username                     | No       |
| `password`      | Elasticsearch password                     | No       |
| `index_pattern` | Index pattern to query (default: `logs-*`) | No       |
| `verify_ssl`    | Verify SSL certificates (default: `false`) | No       |

## Usage

### Command Line Options

```bash
zeteo                              # Auto-detect provider and backend
zeteo --provider openai            # Use specific AI provider
zeteo --backend kibana             # Use specific log backend
zeteo --provider openai --backend kibana
```

### Keyboard Controls

| Key          | Action                             |
| ------------ | ---------------------------------- |
| Enter        | Send message                       |
| Esc          | Exit application                   |
| Up/Down      | Scroll through messages            |
| Page Up/Down | Scroll by page                     |
| Ctrl+Y       | Copy last AI response to clipboard |
| Ctrl+C       | Force exit                         |

### Slash Commands

| Command    | Shortcut | Description                           |
| ---------- | -------- | ------------------------------------- |
| `/help`    | `/h`     | Show available commands               |
| `/quit`    | `/q`     | Exit the application                  |
| `/clear`   | `/c`     | Clear current session history         |
| `/copy`    | `/y`     | Copy last AI response to clipboard    |
| `/backend` | `/b`     | Switch log backend                    |
| `/index`   | `/i`     | Change index pattern for this session |
| `/resume`  | `/r`     | Resume a previous conversation        |

### Example Queries

With a log backend configured:

- "Show me the last 10 errors"
- "Find all payment failures in the last hour"
- "What services had errors today?"
- "Search for timeout errors in the auth service"

## Development

```bash
# Build
cargo build

# Test
cargo test

# Lint
cargo clippy
cargo fmt --check
```

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

MIT License. See [LICENSE](LICENSE) for details.
