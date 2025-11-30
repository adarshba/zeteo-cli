# Contributing

Guidelines for contributing to Zeteo.

## Development Setup

### Prerequisites

- Rust 1.70 or later
- Git

### Build

```bash
git clone https://github.com/adarshba/zeteo-cli
cd zeteo-cli
cargo build
```

### Test

```bash
cargo test
```

### Lint

```bash
cargo clippy
cargo fmt --check
```

## Project Structure

```
zeteo-cli/
├── src/
│   ├── main.rs           # Application entry point
│   ├── tui.rs            # Terminal user interface
│   ├── config/           # Configuration management
│   ├── providers/        # AI provider implementations
│   ├── backends/         # Log backend clients
│   ├── logs/             # Log exploration
│   └── mcp/              # MCP client integration
├── tests/                # Integration tests
├── Cargo.toml
└── README.md
```

## Code Style

- Format code with `cargo fmt`
- Address all clippy warnings with `cargo clippy`
- Write tests for new functionality
- Document public APIs

## Pull Request Process

1. Fork the repository
2. Create a feature branch
3. Make changes
4. Run tests and lints
5. Submit a pull request

### Guidelines

- Include a clear description of changes
- Reference related issues
- Add tests for new functionality
- Keep commits focused

## License

Contributions are licensed under the MIT License. See [LICENSE](LICENSE) for details.
