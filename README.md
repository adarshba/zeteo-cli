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

### Quick Start with .env File

The easiest way to get started is to create a `.env` file with your API keys:

```bash
# 1. Copy the example file
cp .env.example .env

# 2. Edit with your credentials
nano .env

# 3. Run zeteo (it will automatically load your .env file)
zeteo chat "Hello, AI!"
```

### MCP Server Configuration

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

Zeteo requires different environment variables depending on the AI provider you want to use.

#### Using a .env File (Recommended)

The easiest way to manage your API keys is to create a `.env` file in the root directory:

```bash
# Copy the example file
cp .env.example .env

# Edit with your credentials
nano .env  # or vim, code, etc.
```

Your `.env` file should look like this:

```bash
# .env file
OPENAI_API_KEY=sk-proj-xxxxxxxxxxxxx
GOOGLE_API_KEY=AIzaSyxxxxxxxxxxxxxxx
GOOGLE_CLOUD_PROJECT=my-project-id
GOOGLE_CLOUD_LOCATION=us-central1
AZURE_OPENAI_API_KEY=xxxxxxxxxxxxxxxx
AZURE_OPENAI_ENDPOINT=https://your-resource.openai.azure.com
AZURE_OPENAI_DEPLOYMENT=gpt-4-deployment
```

**Note:** The `.env` file is automatically loaded when you run `zeteo`. You don't need to export variables manually!

#### Manual Export (Alternative)

You can also export environment variables manually in your shell:

**OpenAI:**
- `OPENAI_API_KEY`: Your OpenAI API key

**Google AI (Gemini):**
- `GOOGLE_API_KEY`: Your Google AI API key from AI Studio

**Vertex AI:**
- `GOOGLE_CLOUD_PROJECT`: Your GCP project ID
- `GOOGLE_CLOUD_LOCATION`: GCP region (optional, defaults to `us-central1`)
- Authentication via `gcloud auth application-default login`

**Azure OpenAI:**
- `AZURE_OPENAI_API_KEY`: Your Azure OpenAI resource key
- `AZURE_OPENAI_ENDPOINT`: Your Azure OpenAI endpoint URL
- `AZURE_OPENAI_DEPLOYMENT`: Your model deployment name

See the [AI Provider Setup](#ai-provider-setup) section below for detailed configuration instructions.

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

Zeteo supports multiple AI providers with comprehensive setup instructions below.

## AI Provider Setup

### OpenAI Setup

**Prerequisites:**
- OpenAI account
- API key from [OpenAI Platform](https://platform.openai.com/api-keys)

**Setup Steps:**

1. Get your API key:
   ```bash
   # Visit https://platform.openai.com/api-keys
   # Create a new API key
   ```

2. Set environment variable:
   ```bash
   export OPENAI_API_KEY="sk-your-key-here"
   ```

3. Test the connection:
   ```bash
   zeteo chat "Hello, OpenAI!"
   # or explicitly specify provider
   zeteo chat --provider openai "What are the most common error patterns?"
   ```

**Supported Models:** GPT-4o (default), GPT-4, GPT-3.5-turbo

**Persist Configuration (Optional):**
```bash
# Add to your shell profile (~/.bashrc, ~/.zshrc, etc.)
echo 'export OPENAI_API_KEY="sk-your-key-here"' >> ~/.bashrc
source ~/.bashrc
```

---

### Google AI (Gemini) Setup

**Prerequisites:**
- Google account
- API key from [Google AI Studio](https://aistudio.google.com/app/apikey)

**Setup Steps:**

1. Get your API key:
   ```bash
   # Visit https://aistudio.google.com/app/apikey
   # Click "Create API Key"
   # Choose existing or create new Google Cloud project
   ```

2. Set environment variable:
   ```bash
   export GOOGLE_API_KEY="AIzaSy-your-key-here"
   ```

3. Test the connection:
   ```bash
   zeteo chat --provider google "Hello, Gemini!"
   zeteo chat --provider google "Explain OTEL log structure"
   ```

**Supported Models:** Gemini Pro (default), Gemini 1.5 Pro

**Persist Configuration (Optional):**
```bash
# Add to your shell profile
echo 'export GOOGLE_API_KEY="AIzaSy-your-key-here"' >> ~/.bashrc
source ~/.bashrc
```

---

### Vertex AI Setup

**Prerequisites:**
- Google Cloud Platform account
- GCP project with Vertex AI API enabled
- gcloud CLI installed

**Setup Steps:**

1. Install gcloud CLI (if not already installed):
   ```bash
   # macOS
   brew install --cask google-cloud-sdk
   
   # Linux
   curl https://sdk.cloud.google.com | bash
   exec -l $SHELL
   
   # Windows
   # Download from https://cloud.google.com/sdk/docs/install
   ```

2. Initialize gcloud and authenticate:
   ```bash
   # Login to your Google Cloud account
   gcloud auth login
   
   # Set up application default credentials
   gcloud auth application-default login
   
   # Set your project
   gcloud config set project YOUR_PROJECT_ID
   ```

3. Enable Vertex AI API:
   ```bash
   gcloud services enable aiplatform.googleapis.com
   ```

4. Set environment variables:
   ```bash
   export GOOGLE_CLOUD_PROJECT="your-project-id"
   export GOOGLE_CLOUD_LOCATION="us-central1"  # optional, defaults to us-central1
   ```

5. Test the connection:
   ```bash
   zeteo chat --provider vertex "Hello, Vertex AI!"
   zeteo chat --provider vertex "Help me debug this issue"
   ```

**Supported Models:** Gemini Pro (default)

**Available Regions:** us-central1, us-east1, us-west1, europe-west1, asia-northeast1

**Persist Configuration (Optional):**
```bash
# Add to your shell profile
echo 'export GOOGLE_CLOUD_PROJECT="your-project-id"' >> ~/.bashrc
echo 'export GOOGLE_CLOUD_LOCATION="us-central1"' >> ~/.bashrc
source ~/.bashrc
```

**Troubleshooting:**
```bash
# Verify authentication
gcloud auth application-default print-access-token

# Check current project
gcloud config get-value project

# Re-authenticate if needed
gcloud auth application-default login
```

---

### Azure OpenAI Setup

**Prerequisites:**
- Azure account
- Azure OpenAI resource created
- Model deployment configured

**Setup Steps:**

1. Create Azure OpenAI Resource:
   ```bash
   # Via Azure Portal:
   # 1. Go to https://portal.azure.com
   # 2. Search for "Azure OpenAI"
   # 3. Click "Create"
   # 4. Fill in resource details
   # 5. Wait for deployment to complete
   ```

2. Deploy a model:
   ```bash
   # In Azure Portal:
   # 1. Navigate to your Azure OpenAI resource
   # 2. Go to "Model deployments" or "Azure OpenAI Studio"
   # 3. Click "Create new deployment"
   # 4. Select model (e.g., gpt-4, gpt-35-turbo)
   # 5. Give it a deployment name (e.g., "gpt-4-deployment")
   # 6. Click "Create"
   ```

3. Get your credentials:
   ```bash
   # In Azure Portal:
   # 1. Go to your Azure OpenAI resource
   # 2. Click "Keys and Endpoint" in the left menu
   # 3. Copy "KEY 1" (or KEY 2)
   # 4. Copy "Endpoint" URL
   ```

4. Set environment variables:
   ```bash
   export AZURE_OPENAI_API_KEY="your-key-from-azure"
   export AZURE_OPENAI_ENDPOINT="https://your-resource.openai.azure.com"
   export AZURE_OPENAI_DEPLOYMENT="your-deployment-name"
   ```

5. Test the connection:
   ```bash
   zeteo chat --provider azure "Hello, Azure OpenAI!"
   zeteo chat --provider azure "Summarize these errors"
   ```

**Supported Models:** All Azure OpenAI models (GPT-4, GPT-3.5-turbo, etc.)

**API Version:** 2024-02-15-preview (automatically configured)

**Persist Configuration (Optional):**
```bash
# Add to your shell profile
echo 'export AZURE_OPENAI_API_KEY="your-key"' >> ~/.bashrc
echo 'export AZURE_OPENAI_ENDPOINT="https://your-resource.openai.azure.com"' >> ~/.bashrc
echo 'export AZURE_OPENAI_DEPLOYMENT="your-deployment-name"' >> ~/.bashrc
source ~/.bashrc
```

**Troubleshooting:**
```bash
# Test endpoint connectivity
curl -i $AZURE_OPENAI_ENDPOINT/openai/deployments?api-version=2024-02-15-preview \
  -H "api-key: $AZURE_OPENAI_API_KEY"

# Verify environment variables are set
echo $AZURE_OPENAI_API_KEY
echo $AZURE_OPENAI_ENDPOINT
echo $AZURE_OPENAI_DEPLOYMENT
```

---

### Quick Usage Examples

After setting up your preferred provider:

```bash
# OpenAI (default)
zeteo chat "What are the most common error patterns?"

# Google AI
zeteo chat --provider google "Explain OTEL log structure"

# Vertex AI
zeteo chat --provider vertex "Help me debug this issue"

# Azure OpenAI
zeteo chat --provider azure "Summarize these errors"

# Get JSON output for scripting
zeteo chat --output json "What is OpenTelemetry?" | jq '.content'
```

---

### Using Providers in REPL Mode

The REPL (interactive) mode is Zeteo's flagship feature. You can start REPL mode with any provider:

```bash
# Start REPL with default provider (OpenAI)
zeteo

# Start REPL with specific provider
zeteo --provider google
zeteo --provider vertex
zeteo --provider azure
```

**REPL Features:**
- 💬 Continuous conversation with context
- 🎨 Beautiful ASCII art interface
- 📊 Session statistics with `/stats`
- 📜 Conversation history with `/history`
- 💾 Export conversations with `/export`
- 🔍 Search logs directly with `/logs <query>`
- ⏱️ Response time tracking

**Example REPL Session:**
```bash
$ export GOOGLE_API_KEY="your-key"
$ zeteo --provider google

  ╔═══════════════════════════════════════════════════════════════╗
  ║        AI-Powered OTEL Log Explorer & Chat Assistant         ║
  ╚═══════════════════════════════════════════════════════════════╝

┌─ Provider: 🔵 google
└─ Log Explorer: ✓ Connected

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
  ⏱  Session duration:             0h 1m 5s
  🤖 AI Provider:                  google

zeteo [1]> /exit

👋 Goodbye!
```

See [REPL_GUIDE.md](examples/REPL_GUIDE.md) for more detailed examples.

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

**Note:** Before using REPL mode, make sure you've set up at least one AI provider. See [AI Provider Setup](#ai-provider-setup) for detailed instructions.

```bash
# Start interactive mode (default when no command specified)
zeteo

# Or specify a provider (requires provider setup first)
zeteo --provider google      # Requires GOOGLE_API_KEY
zeteo --provider vertex      # Requires GOOGLE_CLOUD_PROJECT and gcloud auth
zeteo --provider azure       # Requires AZURE_OPENAI_* variables
zeteo --provider openai      # Requires OPENAI_API_KEY (default)
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

## Troubleshooting

### Common Issues

#### "API key not set" Error

**Problem:** Environment variable not configured.

**Solution:**
```bash
# Check which provider you're using
zeteo chat --provider <provider> "test"

# Set the appropriate variable:
export OPENAI_API_KEY="sk-..."        # for OpenAI
export GOOGLE_API_KEY="AIza..."       # for Google AI
export AZURE_OPENAI_API_KEY="..."    # for Azure
# See provider setup sections above for complete configuration
```

#### Vertex AI: "Failed to get access token"

**Problem:** Not authenticated with gcloud.

**Solution:**
```bash
# Authenticate
gcloud auth application-default login

# Verify authentication
gcloud auth application-default print-access-token

# Set project
export GOOGLE_CLOUD_PROJECT="your-project-id"
```

#### Azure OpenAI: "API error"

**Problem:** Incorrect endpoint or deployment name.

**Solution:**
```bash
# Verify your endpoint format (should include https://)
echo $AZURE_OPENAI_ENDPOINT
# Should be: https://your-resource.openai.azure.com

# Verify deployment name matches Azure portal
echo $AZURE_OPENAI_DEPLOYMENT

# Test with curl
curl -i $AZURE_OPENAI_ENDPOINT/openai/deployments?api-version=2024-02-15-preview \
  -H "api-key: $AZURE_OPENAI_API_KEY"
```

#### "Unknown provider" Error

**Problem:** Invalid provider name specified.

**Solution:**
```bash
# Use one of these valid provider names:
zeteo chat --provider openai "test"
zeteo chat --provider google "test"
zeteo chat --provider vertex "test"
zeteo chat --provider azure "test"
```

### Getting Help

If you encounter issues:

1. Check the [AI Provider Setup](#ai-provider-setup) section for your provider
2. Verify environment variables are set: `env | grep -E "(OPENAI|GOOGLE|AZURE)"`
3. Try running with `--verbose` flag for more details: `zeteo --verbose chat "test"`
4. Check the [GitHub Issues](https://github.com/adarshba/zeteo-cli/issues) page

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is inspired by gemini-cli and built for the OpenTelemetry community.

## Acknowledgments

- Similar to [gemini-cli](https://github.com/search?q=gemini-cli)
- Powered by [otel-mcp-server](https://www.npmjs.com/package/otel-mcp-server)
- Built with Rust 🦀