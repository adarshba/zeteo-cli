# Zeteo CLI - Interactive REPL Mode Examples

This guide demonstrates the enhanced interactive REPL mode with beautiful UI and advanced features.

## 🎨 New Enhanced REPL Experience

The REPL mode has been completely redesigned with a focus on visual appeal and user experience!

### Starting the REPL

Simply run `zeteo` without any command to enter the beautiful interactive mode:

```bash
$ export OPENAI_API_KEY="your-key"
$ zeteo

  ╔═══════════════════════════════════════════════════════════════╗
  ║                                                               ║
  ║   ███████╗███████╗████████╗███████╗ ██████╗                 ║
  ║   ╚══███╔╝██╔════╝╚══██╔══╝██╔════╝██╔═══██╗                ║
  ║     ███╔╝ █████╗     ██║   █████╗  ██║   ██║                ║
  ║    ███╔╝  ██╔══╝     ██║   ██╔══╝  ██║   ██║                ║
  ║   ███████╗███████╗   ██║   ███████╗╚██████╔╝                ║
  ║   ╚══════╝╚══════╝   ╚═╝   ╚══════╝ ╚═════╝                 ║
  ║                                                               ║
  ║        AI-Powered OTEL Log Explorer & Chat Assistant         ║
  ║                                                               ║
  ╚═══════════════════════════════════════════════════════════════╝

┌─ Provider: 🤖 openai
└─ Log Explorer: ✓ Connected

╭──────────── Available Commands ────────────╮
│                                            │
│  🚪 /exit, /quit, /q   → Exit the REPL
│  🗑️ /clear            → Clear conversation history
│  ❓ /help, /h         → Show detailed help
│  🔍 /logs <query>     → Search OTEL logs
│  📊 /stats            → Show session statistics
│  💾 /export [file]    → Export conversation
│  📜 /history          → Show conversation history
│                                            │
╰────────────────────────────────────────────╯

💡 Tip: Just type your message to start chatting!
   Press Ctrl+C or type /exit to quit.

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

zeteo [0]>
```

### 🌟 Visual Enhancements

- **Beautiful ASCII Art Banner**: Eye-catching ZETEO logo on startup
- **Provider Icons**: Emoji indicators for each AI provider (🤖 🔷 🔵 ☁️)
- **Color-Coded Output**: Different colors for different types of information
- **Clean Layout**: Professional borders and section dividers
- **Message Counter**: Track conversation depth in the prompt `[N]`

### Using Different Providers

Each provider gets its unique icon and color scheme:

```bash
# OpenAI - 🤖
$ export OPENAI_API_KEY="your-key"
$ zeteo --provider openai

# Vertex AI - 🔷
$ export GOOGLE_CLOUD_PROJECT="your-project"
$ zeteo --provider vertex

# Google AI - 🔵
$ export GOOGLE_API_KEY="your-key"
$ zeteo --provider google

# Azure OpenAI - ☁️
$ export AZURE_OPENAI_API_KEY="your-key"
$ export AZURE_OPENAI_ENDPOINT="https://your-resource.openai.azure.com"
$ export AZURE_OPENAI_DEPLOYMENT="your-deployment"
$ zeteo --provider azure
```

### 💬 Example Conversation with Enhanced UI

```
zeteo [0]> What is OpenTelemetry?

💭 Thinking...

┌─ AI Response ─────────────────────────────────────
OpenTelemetry is an observability framework for cloud-native software...

Key features:
- Unified standard for traces, metrics, and logs
- Vendor-neutral and open-source
- Wide language support
└───────────────────────────────────────────────────

⏱  Response time: 1.23s

zeteo [1]> How do I instrument a Node.js application?

💭 Thinking...

┌─ AI Response ─────────────────────────────────────
To instrument a Node.js application with OpenTelemetry:

1. Install the required packages
2. Initialize the SDK
3. Configure exporters
└───────────────────────────────────────────────────

⏱  Response time: 1.45s

zeteo [2]> /stats

╔══════════════════════════════════════════════════╗
║          Session Statistics                      ║
╚══════════════════════════════════════════════════╝

  💬 Total messages exchanged:     2
  📝 Messages in history:          4
  ⏱  Session duration:             0h 2m 15s
  🤖 AI Provider:                  openai
  🔍 Log Explorer:                 Connected ✓

  📊 Average time per exchange: 1.3s

zeteo [2]> /history

╭────────── Conversation History ──────────╮
│                                          │
│ 👤 [1] You:
│   What is OpenTelemetry?
│                                          │
│ 🤖 [1] AI:
│   OpenTelemetry is an observability framework for c...
│                                          │
│ 👤 [2] You:
│   How do I instrument a Node.js application?
│                                          │
│ 🤖 [2] AI:
│   To instrument a Node.js application with OpenTele...
│                                          │
╰──────────────────────────────────────────╯

💡 Tip: Use /export to save full conversation

zeteo [2]> /export otel-conversation.json

✓ Conversation exported to: otel-conversation.json

zeteo [2]> /exit

╔═══════════════════════════════════════════════════════════╗
║                 Thank You for Using Zeteo!               ║
╚═══════════════════════════════════════════════════════════╝

📊 Session Summary:
   2 messages exchanged in 2 minutes

💡 Tip: Don't forget to export your conversation with /export

👋 Goodbye!
```

## 📊 New /stats Command

The `/stats` command provides detailed session analytics:

- **Message Count**: Total exchanges and history size
- **Session Duration**: Time spent in the REPL
- **Provider Info**: Current AI provider and connection status
- **Performance Metrics**: Average response time per exchange

```bash
zeteo> /stats
```

## 📋 REPL Special Commands Reference

### Core Commands

| Command | Icon | Description | Example |
|---------|------|-------------|---------|
| `/exit`, `/quit`, `/q` | 🚪 | Exit the REPL with session summary | `/exit` |
| `/clear` | 🗑️ | Clear conversation history | `/clear` |
| `/help`, `/h` | ❓ | Show detailed help and tips | `/help` |
| `/stats` | 📊 | Show session statistics | `/stats` |

### Log Commands

| Command | Icon | Description | Example |
|---------|------|-------------|---------|
| `/logs <query>` | 🔍 | Search OTEL logs | `/logs error` |

### Export & History

| Command | Icon | Description | Example |
|---------|------|-------------|---------|
| `/export [file]` | 💾 | Export conversation to JSON/CSV | `/export chat.json` |
| `/history` | 📜 | Show conversation history | `/history` |

### Provider Info

| Command | Icon | Description | Example |
|---------|------|-------------|---------|
| `/provider` | 🔄 | Show current provider info | `/provider` |

## 🎨 Enhanced Features

### Color-Coded Output

The REPL uses intelligent color coding:
- **Green**: Success messages and AI responses
- **Yellow**: Warnings and tips
- **Red**: Errors and issues
- **Cyan**: Commands and prompts
- **Magenta**: Statistics and highlights
- **Dimmed**: Less important info and timestamps

### Response Formatting

AI responses are automatically formatted with:
- Syntax highlighting hints for code blocks
- Bold headers (lines starting with #)
- Highlighted list items (- and numbered lists)
- Clean borders and separators
- Response time tracking

### Smart Indicators

- **💭 Thinking...**: Shows while waiting for AI response
- **✓ Success**: Confirms successful operations
- **⚠ Warning**: Alerts about potential issues
- **❌ Error**: Indicates problems with helpful suggestions
- **ℹ Info**: Provides additional context

### Session Tracking

Every REPL session tracks:
- Number of message exchanges
- Total session duration
- Individual response times
- Provider and configuration status
- Log explorer connectivity

## Advanced Log Features

### Filtering Logs

```bash
# Filter by log level
$ zeteo logs --query "error" --level ERROR

# Filter by service
$ zeteo logs --query "database" --service "api-gateway"

# Combine multiple filters
$ zeteo logs --query "timeout" --level WARN --service "backend"
```

### Log Aggregation

```bash
# Show statistics
$ zeteo logs --query "error" --aggregate

=== Log Aggregation ===
Total logs: 150

By Level:
  ERROR: 120
  WARN: 30

By Service:
  api-gateway: 80
  backend: 70

Time Range:
  From: 2024-01-01T10:00:00Z
  To:   2024-01-01T12:00:00Z
```

### Export Logs

```bash
# Export to JSON
$ zeteo logs --query "error" --max 100 --export errors.json
Exported 100 logs to errors.json

# Export to CSV
$ zeteo logs --query "error" --max 100 --export errors.csv
Exported 100 logs to errors.csv

# Export with filters
$ zeteo logs --query "*" --level ERROR --service "api" --export api-errors.json
```

### Real-time Log Streaming

```bash
# Stream all logs
$ zeteo logs --query "*" --stream

Starting log stream...
Query: *
Press Ctrl+C to stop streaming

[2024-01-01T12:00:01Z] INFO New request received
[2024-01-01T12:00:02Z] ERROR Database connection failed
[2024-01-01T12:00:03Z] WARN High memory usage detected
...

# Stream with filters
$ zeteo logs --query "error" --level ERROR --stream
```

### Combined Advanced Usage

```bash
# Search, filter, aggregate, and export in one command
$ zeteo logs \
  --query "database" \
  --level ERROR \
  --service "backend" \
  --max 200 \
  --aggregate \
  --export db-errors.json

Searching logs with query: database
MCP Server: otel-mcp-server
Max results: 200

=== Log Aggregation ===
Total logs: 45

By Level:
  ERROR: 45

By Service:
  backend: 45

Exported 45 logs to db-errors.json
```

## REPL Special Commands

### Search Logs from REPL

```
zeteo> /logs error
Searching logs for: 'error'

[2024-01-01T12:00:00Z] ERROR Database connection failed
  Service: api-gateway
  Trace ID: abc123def456

[2024-01-01T12:00:05Z] ERROR Authentication failed
  Service: auth-service
  Trace ID: def456ghi789
```

### Export Conversations

```
# Export as JSON (default)
zeteo> /export my-chat
Conversation exported to: my-chat.json

# Export as CSV
zeteo> /export analysis.csv
Conversation exported to: analysis.csv

# Export with full filename
zeteo> /export debugging-session.json
Conversation exported to: debugging-session.json
```

### Clear and View History

```
# View conversation history
zeteo> /history
=== Conversation History ===

[1] You: What is OTEL?
[2] AI: OpenTelemetry (OTEL) is...

# Clear conversation to start fresh
zeteo> /clear
Conversation history cleared.

zeteo> /history
No conversation history yet.
```

## Integration with Other Tools

### Pipeline with jq

```bash
# Search logs and process with jq
$ zeteo logs --query "error" --output json | jq '.[] | select(.level=="ERROR")'

# Export and analyze
$ zeteo logs --query "*" --export logs.json
$ cat logs.json | jq '.[] | .service' | sort | uniq -c
```

### Scripting

```bash
#!/bin/bash

# Automated log analysis script
export OPENAI_API_KEY="your-key"

# Collect errors
zeteo logs --query "error" --level ERROR --export errors.json

# Get AI analysis
echo "Analyze these errors and suggest solutions" | \
  zeteo chat --provider openai --output json | \
  jq -r '.content' > analysis.txt

# Generate report
echo "Error Analysis Report" > report.txt
echo "=====================" >> report.txt
cat analysis.txt >> report.txt
```

## 💡 Tips and Best Practices

### REPL Usage Tips

1. **Start with REPL for exploration**: The interactive mode is perfect for ad-hoc queries and exploration
2. **Use /stats regularly**: Monitor your session performance and activity
3. **Export important conversations**: Save debugging sessions for team review with `/export`
4. **Leverage conversation history**: The AI remembers context, so build on previous questions
5. **Check response times**: Use the displayed metrics to understand AI performance

### Visual Experience Tips

1. **Full terminal width**: The REPL looks best in a wide terminal (80+ columns)
2. **Color support**: Ensure your terminal supports ANSI colors for the full experience
3. **Clear screen on start**: The REPL automatically clears the screen for a fresh start
4. **Watch the message counter**: The `[N]` in the prompt shows conversation depth

### Log Exploration Tips

1. **Use filters to reduce noise**: Combine `--level`, `--service`, and query for precise results
2. **Stream during incidents**: Use `--stream` to monitor logs in real-time
3. **Aggregate for overview**: Use `--aggregate` to understand log distribution
4. **Export in appropriate format**:
   - JSON for programmatic processing
   - CSV for spreadsheet analysis

### Performance Tips

1. **Monitor response times**: Each AI response shows its execution time
2. **Check session stats**: Use `/stats` to see average response times
3. **Clear history when needed**: Use `/clear` to start fresh if the context gets too large

## Configuration

All features work with the existing configuration file at `~/.config/zeteo-cli/config.json`. 

Initialize if not already done:
```bash
$ zeteo config --init
```

Verify configuration:
```bash
$ zeteo config --show
```

## 🚀 What's New in the Enhanced REPL

### Visual Improvements
- ✨ Beautiful ASCII art banner with ZETEO branding
- 🎨 Provider-specific emoji icons for quick identification
- 🌈 Rich color scheme with intelligent color coding
- 📊 Professional borders and section dividers
- 🔢 Message counter in prompt showing conversation depth

### New Commands
- 📊 `/stats` - Comprehensive session statistics
- 📋 Enhanced `/history` - Beautiful conversation summary
- 🔄 `/provider` - Detailed provider information

### UX Enhancements
- ⚡ Response time tracking for every AI interaction
- 💭 Animated thinking indicator
- 🎯 Color-coded status messages (✓ ⚠ ❌ ℹ)
- 📈 Session duration and performance metrics
- 👋 Friendly goodbye message with session summary

### Better Formatting
- 🎨 Syntax highlighting hints in responses
- 📝 Formatted lists and headers
- 📦 Clean response boxes with borders
- 💡 Helpful tips throughout the interface
- 🗂️ Truncated history view for quick browsing

## Future Enhancements

While the REPL is now beautifully enhanced and stable, future versions may include:
- 🖥️ Full ncurses-style TUI mode with split panels
- 💾 Persistent conversation history across sessions
- ⚡ Response caching for repeated queries
- 🔄 Automatic retry with exponential backoff for failed requests
- 🎭 Theme customization options
- 📱 Mobile-friendly terminal output
