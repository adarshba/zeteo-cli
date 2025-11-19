# Zeteo CLI - Interactive REPL Mode Examples

This guide demonstrates the new interactive REPL mode and advanced features added to Zeteo CLI.

## Interactive REPL Mode

### Starting the REPL

Simply run `zeteo` without any command to enter interactive mode:

```bash
$ export OPENAI_API_KEY="your-key"
$ zeteo

╔═══════════════════════════════════════════════════════════╗
║           Welcome to Zeteo Interactive Shell             ║
╚═══════════════════════════════════════════════════════════╝

Provider: openai

Available commands:
  /exit - Exit the REPL
  /clear - Clear conversation history
  /help - Show help
  /logs - Search logs (e.g., /logs error)
  /provider - Switch provider (e.g., /provider openai)
  /export - Export conversation to file (json or csv)
  /history - Show conversation history

Type your message and press Enter to chat.
Press Ctrl+C or type /exit to quit.

zeteo>
```

### Using Different Providers

```bash
# Use Google AI
$ export GOOGLE_API_KEY="your-key"
$ zeteo --provider google

# Use Vertex AI
$ export GOOGLE_CLOUD_PROJECT="your-project"
$ zeteo --provider vertex

# Use Azure OpenAI
$ export AZURE_OPENAI_API_KEY="your-key"
$ export AZURE_OPENAI_ENDPOINT="https://your-resource.openai.azure.com"
$ export AZURE_OPENAI_DEPLOYMENT="your-deployment"
$ zeteo --provider azure
```

### Example Conversation

```
zeteo> What is OpenTelemetry?

OpenTelemetry is an observability framework for cloud-native software...

zeteo> How do I instrument a Node.js application?

To instrument a Node.js application with OpenTelemetry:
1. Install the required packages...
2. Initialize the SDK...

zeteo> Can you show me a code example?

Here's a complete example:
[code example provided with context from previous questions]

zeteo> /history
=== Conversation History ===

[1] You: What is OpenTelemetry?
[2] AI: OpenTelemetry is an observability framework...
[3] You: How do I instrument a Node.js application?
[4] AI: To instrument a Node.js application...
[5] You: Can you show me a code example?
[6] AI: Here's a complete example...

zeteo> /export otel-conversation.json
Conversation exported to: otel-conversation.json

zeteo> /exit
Exiting REPL...
```

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

## Tips and Best Practices

1. **Start with REPL for exploration**: Use interactive mode to explore logs and get AI assistance
2. **Use filters to reduce noise**: Combine `--level`, `--service`, and query for precise results
3. **Export important conversations**: Save debugging sessions for team review
4. **Stream during incidents**: Use `--stream` to monitor logs in real-time
5. **Aggregate for overview**: Use `--aggregate` to understand log distribution
6. **Export in appropriate format**:
   - JSON for programmatic processing
   - CSV for spreadsheet analysis

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

## Future Enhancements

While these features are now complete, future versions may include:
- Full ncurses-style TUI mode
- Persistent conversation history across sessions
- Response caching for repeated queries
- Automatic retry with exponential backoff for failed requests
