# Contributing to Zeteo CLI

Thank you for your interest in contributing to Zeteo CLI! This document provides guidelines and information for contributors.

## Development Setup

### Prerequisites

- Rust 1.70 or later
- Node.js (for testing with otel-mcp-server)
- Git

### Clone and Build

```bash
git clone https://github.com/adarshba/zeteo-cli
cd zeteo-cli
cargo build
```

### Running Tests

```bash
cargo test
```

### Running Lints

```bash
cargo clippy
cargo fmt --check
```

## Project Structure

```
zeteo-cli/
├── src/
│   ├── main.rs              # CLI entry point and command handling
│   ├── config/
│   │   └── mod.rs           # Configuration management
│   ├── mcp/
│   │   └── mod.rs           # MCP client integration
│   ├── providers/
│   │   ├── mod.rs           # AI provider trait and common types
│   │   ├── openai.rs        # OpenAI provider implementation
│   │   ├── vertex.rs        # Vertex AI provider (placeholder)
│   │   ├── google.rs        # Google AI provider (placeholder)
│   │   └── azure.rs         # Azure OpenAI provider (placeholder)
│   └── logs/
│       └── mod.rs           # Log exploration and display logic
├── Cargo.toml
├── README.md
└── CONTRIBUTING.md
```

## Adding a New AI Provider

To add a new AI provider:

1. Create a new file in `src/providers/` (e.g., `anthropic.rs`)
2. Implement the `AiProvider` trait:

```rust
use super::{AiProvider, ChatRequest, ChatResponse};
use anyhow::Result;

#[derive(Clone)]
pub struct AnthropicProvider {
    api_key: String,
    model: String,
}

impl AnthropicProvider {
    pub fn new(api_key: String, model: Option<String>) -> Self {
        AnthropicProvider {
            api_key,
            model: model.unwrap_or_else(|| "claude-3-opus".to_string()),
        }
    }
}

#[async_trait::async_trait]
impl AiProvider for AnthropicProvider {
    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse> {
        // Implementation here
        todo!()
    }
    
    fn provider_name(&self) -> &str {
        "Anthropic"
    }
}
```

3. Export the provider in `src/providers/mod.rs`:

```rust
pub mod anthropic;
pub use anthropic::AnthropicProvider;
```

4. Add command-line support in `src/main.rs`

## Code Style

- Follow Rust standard formatting (`cargo fmt`)
- Run clippy and address all warnings (`cargo clippy`)
- Write tests for new functionality
- Document public APIs with doc comments

## Testing

### Unit Tests

Place unit tests in the same file as the code:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_something() {
        // Test code
    }
}
```

### Integration Tests

For testing with actual services, create integration tests that can be skipped in CI:

```rust
#[tokio::test]
#[ignore] // Skip in CI
async fn test_with_real_service() {
    // Test code
}
```

## Pull Request Process

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Run tests and lints
5. Commit your changes (`git commit -m 'Add amazing feature'`)
6. Push to your fork (`git push origin feature/amazing-feature`)
7. Open a Pull Request

### PR Guidelines

- Include a clear description of the changes
- Reference any related issues
- Add tests for new functionality
- Update documentation as needed
- Ensure all tests pass
- Keep commits focused and atomic

## Areas for Contribution

### High Priority

- [ ] Full Vertex AI provider implementation
- [ ] Full Google AI provider implementation
- [ ] Full Azure OpenAI provider implementation
- [ ] Real MCP server communication (currently placeholder)
- [ ] Real-time log streaming
- [ ] Advanced log filtering

### Medium Priority

- [ ] Interactive TUI mode
- [ ] Log export functionality
- [ ] Custom query templates
- [ ] Log aggregation and analytics
- [ ] Performance optimizations

### Documentation

- [ ] More usage examples
- [ ] Video tutorials
- [ ] API documentation
- [ ] Architecture diagrams

## Questions?

Feel free to open an issue for any questions or discussions about contributing.

## License

By contributing, you agree that your contributions will be licensed under the same license as the project.
