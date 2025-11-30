# zeteo

AI assistant for your terminal.

## Install

```bash
cargo install --path .
```

## Usage

```bash
zeteo
```

## Configuration

Set one of these environment variables:

```bash
export OPENAI_API_KEY="sk-..."
export GOOGLE_API_KEY="AIza..."
export AZURE_OPENAI_API_KEY="..." 
export AZURE_OPENAI_ENDPOINT="https://..."
export AZURE_OPENAI_DEPLOYMENT="..."
export GOOGLE_CLOUD_PROJECT="..."
```

## Providers

Specify a provider:

```bash
zeteo --provider openai
zeteo --provider google
zeteo --provider azure
zeteo --provider vertex
```

## Commands

| Key | Action |
|-----|--------|
| Enter | Send message |
| Esc | Quit |
| ↑↓ | Scroll |
| /clear | Clear chat |
| /quit | Exit |

## License

MIT
