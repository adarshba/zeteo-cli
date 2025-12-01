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

## Pre-commit Hooks

Set up pre-commit hooks to automatically run checks before each commit.

### Using the setup script

```bash
./scripts/setup-hooks.sh
```

### Manual installation

```bash
cp hooks/pre-commit .git/hooks/pre-commit
chmod +x .git/hooks/pre-commit
```

The pre-commit hook runs:

- `cargo fmt --check` - Format checking
- `cargo clippy` - Linting
- `cargo check` - Build verification

## Commit Message Format

We follow the [Conventional Commits](https://www.conventionalcommits.org/) specification. This enables automatic changelog generation and semantic versioning.

### Format

```
<type>(<scope>): <subject>

<body>

<footer>
```

### Types

| Type       | Description                                      | Version Bump |
| ---------- | ------------------------------------------------ | ------------ |
| `feat`     | New feature                                      | MINOR        |
| `fix`      | Bug fix                                          | PATCH        |
| `docs`     | Documentation changes                            | None         |
| `style`    | Code style changes (formatting, whitespace)      | None         |
| `refactor` | Code refactoring (no feature or fix)             | None         |
| `perf`     | Performance improvements                         | PATCH        |
| `test`     | Test additions or changes                        | None         |
| `build`    | Build system or dependencies                     | None         |
| `ci`       | CI/CD configuration changes                      | None         |
| `chore`    | Other changes (tooling, config, etc.)            | None         |
| `revert`   | Revert a previous commit                         | Varies       |

### Breaking Changes

For breaking changes, add `BREAKING CHANGE:` in the commit body or footer, or append `!` after the type:

```
feat!: remove deprecated API endpoints

BREAKING CHANGE: The /api/v1/logs endpoint has been removed.
Use /api/v2/logs instead.
```

Breaking changes trigger a MAJOR version bump.

### Examples

**Feature:**

```
feat(cli): add log filtering capability

- Implemented pattern-based filtering
- Added benchmark tests

Closes #123
```

**Bug Fix:**

```
fix(gemini): handle API timeout errors

Previously, timeouts would crash the application.
Now we gracefully retry up to 3 times.

Fixes #456
```

**Documentation:**

```
docs: update README with new configuration options
```

**CI:**

```
ci: add multi-platform release builds
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
3. Make changes with conventional commit messages
4. Run tests and lints
5. Submit a pull request

### Guidelines

- Use conventional commit messages (enforced by CI)
- Include a clear description of changes
- Reference related issues
- Add tests for new functionality
- Keep commits focused

## Release Process

See [RELEASE.md](RELEASE.md) for details on our automated release process.

## License

Contributions are licensed under the MIT License. See [LICENSE](LICENSE) for details.
