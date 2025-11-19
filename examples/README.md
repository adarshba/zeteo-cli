# Zeteo CLI Examples

This directory contains examples of using Zeteo CLI for various tasks.

## Basic Usage

### 1. View Configuration

```bash
zeteo-cli config --show
```

Output:
```
=== Zeteo CLI Configuration ===

MCP Servers:

  otel-mcp-server
    Command: npx
    Args: ["-y", "otel-mcp-server"]
    Environment:
      ELASTICSEARCH_URL: http://localhost:9200
      ELASTICSEARCH_USERNAME: elastic
      ELASTICSEARCH_PASSWORD: changeme
      SERVER_NAME: otel-mcp-server
      LOGLEVEL: OFF
```

### 2. Search Logs

Search for error logs:
```bash
zeteo-cli logs --query "error" --max 10
```

Search for logs from a specific service:
```bash
zeteo-cli logs --query "service:api-gateway AND level:error" --max 20
```

Search within a time range:
```bash
zeteo-cli logs --query "timestamp:[2024-01-01 TO 2024-01-02]" --max 50
```

### 3. Interactive Log Explorer

Launch interactive mode to explore logs:
```bash
zeteo-cli logs --interactive
```

In interactive mode, you can type queries and see results immediately.

### 4. AI-Powered Analysis

Ask the AI about your logs:
```bash
export OPENAI_API_KEY="your-api-key"
zeteo-cli chat "What are the most common errors in the last hour?"
```

Get help debugging:
```bash
zeteo-cli chat "Explain the error pattern in trace ID abc123"
```

Generate insights:
```bash
zeteo-cli chat "Summarize the performance issues from the last deployment"
```

## Advanced Examples

### Query Syntax

The query syntax depends on your backend (Elasticsearch, OpenObserve, etc.).

#### Elasticsearch Queries

```bash
# Full-text search
zeteo-cli logs --query "connection timeout"

# Field-specific search
zeteo-cli logs --query "level:ERROR AND service:payment"

# Wildcard search
zeteo-cli logs --query "user_id:user_*"

# Range queries
zeteo-cli logs --query "status:[400 TO 599]"
```

#### Boolean Operators

```bash
# AND
zeteo-cli logs --query "error AND database"

# OR
zeteo-cli logs --query "error OR warning"

# NOT
zeteo-cli logs --query "error NOT timeout"
```

### Using with Different Backends

#### OpenObserve

Update your config to point to OpenObserve:

```json
{
  "servers": {
    "otel-mcp-server": {
      "command": "npx",
      "args": ["-y", "otel-mcp-server"],
      "env": {
        "OPENOBSERVE_URL": "http://localhost:5080",
        "OPENOBSERVE_USERNAME": "admin",
        "OPENOBSERVE_PASSWORD": "password",
        "SERVER_NAME": "otel-mcp-server",
        "LOGLEVEL": "OFF"
      }
    }
  }
}
```

#### Kibana

For Kibana with Elasticsearch:

```json
{
  "servers": {
    "otel-mcp-server": {
      "command": "npx",
      "args": ["-y", "otel-mcp-server"],
      "env": {
        "ELASTICSEARCH_URL": "http://localhost:9200",
        "ELASTICSEARCH_USERNAME": "elastic",
        "ELASTICSEARCH_PASSWORD": "your-password",
        "KIBANA_URL": "http://localhost:5601",
        "SERVER_NAME": "otel-mcp-server",
        "LOGLEVEL": "OFF"
      }
    }
  }
}
```

## Scripting with Zeteo

### Bash Script Example

```bash
#!/bin/bash

# Monitor errors in production
while true; do
    echo "Checking for errors at $(date)"
    zeteo-cli logs --query "level:ERROR AND env:production" --max 5
    sleep 60
done
```

### Error Alert Script

```bash
#!/bin/bash

# Get error count and alert if too high
ERROR_COUNT=$(zeteo-cli logs --query "level:ERROR" --max 1000 | grep -c "ERROR")

if [ "$ERROR_COUNT" -gt 100 ]; then
    echo "ALERT: Too many errors detected: $ERROR_COUNT"
    # Send alert notification
fi
```

## AI Integration Examples

### Analyzing Log Patterns

```bash
# Get logs and ask AI to analyze
LOGS=$(zeteo-cli logs --query "level:ERROR" --max 50)
zeteo-cli chat "Analyze these errors and suggest solutions: $LOGS"
```

### Automated Troubleshooting

```bash
# Get specific error and ask for help
zeteo-cli chat "What causes this error: 'Connection refused to database'"
```

## Tips and Best Practices

1. **Use specific queries**: More specific queries return faster and more relevant results
2. **Limit results**: Use `--max` to limit the number of results for faster queries
3. **Save common queries**: Create shell aliases for frequently used queries
4. **Combine with other tools**: Pipe output to `jq`, `grep`, or other CLI tools
5. **Environment variables**: Keep API keys in environment variables, not in scripts

## Example Aliases

Add these to your `.bashrc` or `.zshrc`:

```bash
alias zlogs='zeteo-cli logs'
alias zconfig='zeteo-cli config --show'
alias zchat='zeteo-cli chat'
alias zerrors='zeteo-cli logs --query "level:ERROR"'
alias zwarnings='zeteo-cli logs --query "level:WARN"'
```
