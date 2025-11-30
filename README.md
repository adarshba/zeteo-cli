# Zeteo

A terminal-based AI assistant built with Rust.

## Overview

Zeteo is a command-line interface for interacting with AI language models. It provides a responsive terminal user interface with support for multiple AI providers.

## Requirements

- Rust 1.70 or later
- An API key from a supported AI provider

## Installation

```bash
cargo install --path .
```

Or build from source:

```bash
cargo build --release
```

The binary will be available at `target/release/zeteo`.

## Configuration

Configure an AI provider by setting the appropriate environment variables:

### OpenAI

```bash
export OPENAI_API_KEY="sk-..."
```

### Google AI

```bash
export GOOGLE_API_KEY="AIza..."
```

### Azure OpenAI

```bash
export AZURE_OPENAI_API_KEY="..."
export AZURE_OPENAI_ENDPOINT="https://..."
export AZURE_OPENAI_DEPLOYMENT="..."
```

### Vertex AI

```bash
export GOOGLE_CLOUD_PROJECT="..."
export GOOGLE_CLOUD_LOCATION="us-central1"
```

You can also use a `.env` file in the current directory. Copy `.env.example` as a starting point:

```bash
cp .env.example .env
```

## Usage

Start the application:

```bash
zeteo
```

Use a specific provider:

```bash
zeteo --provider openai
zeteo --provider google
zeteo --provider azure
zeteo --provider vertex
```

Generate shell completions:

```bash
zeteo completions bash > ~/.bash_completion.d/zeteo
zeteo completions zsh > ~/.zfunc/_zeteo
```

## Keyboard Controls

| Key | Action |
|-----|--------|
| Enter | Send message |
| Esc | Exit application |
| Up/Down | Scroll through messages |
| Page Up/Down | Scroll by page |
| Ctrl+C | Force exit |

### Commands

| Command | Description |
|---------|-------------|
| /quit, /q | Exit application |
| /clear | Clear conversation |

## Development

Build the project:

```bash
cargo build
```

Run tests:

```bash
cargo test
```

Run linting:

```bash
cargo clippy
cargo fmt --check
```

## License

MIT License. See [LICENSE](LICENSE) for details.
