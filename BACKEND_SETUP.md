# Backend Setup Guide

This guide provides detailed setup instructions for each supported log backend in zeteo-cli.

## Overview

Zeteo supports three log backends:
- **Elasticsearch**: Standard ELK stack queries via MCP or direct connection
- **OpenObserve**: Cloud-native observability platform with SQL queries
- **Kibana**: Visual exploration with KQL (Kibana Query Language)

## Configuration File

All backends are configured in `~/.config/zeteo-cli/config.json`. The configuration file is created automatically on first run with default values.

### Location
- **Linux/macOS**: `~/.config/zeteo-cli/config.json`
- **Windows**: `%APPDATA%\zeteo-cli\config.json`

## Elasticsearch Setup

### Prerequisites
- Elasticsearch cluster running (version 7.x or 8.x)
- Network access to Elasticsearch
- Valid credentials (username/password or API key)

### Configuration

Add to your `config.json`:

```json
{
  "backends": {
    "elasticsearch": {
      "type": "elasticsearch",
      "url": "http://localhost:9200",
      "username": "elastic",
      "password": "changeme",
      "index_pattern": "logs-*",
      "verify_ssl": false
    }
  }
}
```

### Configuration Options

| Field | Description | Required | Default |
|-------|-------------|----------|---------|
| `url` | Elasticsearch URL | Yes | - |
| `username` | Basic auth username | No | - |
| `password` | Basic auth password | No | - |
| `index_pattern` | Elasticsearch index pattern | Yes | `logs-*` |
| `verify_ssl` | Verify SSL certificates | No | `false` |

### Usage Examples

```bash
# Basic query
zeteo logs --backend elasticsearch --query "error" --max 50

# With filters
zeteo logs --backend elasticsearch --query "payment" --level ERROR --service api-gateway

# Time range query
zeteo logs --backend elasticsearch \
  --query "exception" \
  --start-time "2024-01-01T00:00:00Z" \
  --end-time "2024-01-02T00:00:00Z"

# Export to JSON
zeteo logs --backend elasticsearch --query "error" --export errors.json
```

### Query Language

Elasticsearch backend supports Lucene query syntax:
- Simple terms: `error`, `payment failed`
- Field queries: `level:ERROR`, `service.name:payments`
- Boolean operators: `error AND payment`, `error OR exception`
- Wildcards: `pay*`, `err?r`
- Ranges: `timestamp:[2024-01-01 TO 2024-01-31]`

### Troubleshooting

**Connection Error:**
```bash
# Test connectivity
curl -u elastic:password http://localhost:9200/_cluster/health

# Check if Elasticsearch is running
sudo systemctl status elasticsearch  # Linux
```

**Authentication Error:**
```bash
# Verify credentials
curl -u elastic:password http://localhost:9200

# Generate API key (alternative to basic auth)
curl -X POST "localhost:9200/_security/api_key" \
  -u elastic:password \
  -H "Content-Type: application/json" \
  -d '{"name":"zeteo-api-key"}'
```

**No Results:**
- Verify index pattern matches your indices
- Check time range (logs might be outside the default range)
- Try a broader query: `--query "*"`

---

## OpenObserve Setup

### Prerequisites
- OpenObserve instance running (self-hosted or cloud)
- Organization and stream created
- Valid credentials

### Getting Started with OpenObserve

#### Option 1: Docker (Quick Start)

```bash
# Pull and run OpenObserve
docker run -d \
  --name openobserve \
  -p 5080:5080 \
  -e ZO_ROOT_USER_EMAIL="admin@example.com" \
  -e ZO_ROOT_USER_PASSWORD="Complexpass#123" \
  public.ecr.aws/zinclabs/openobserve:latest

# Wait for it to start
sleep 10

# Access UI at http://localhost:5080
```

#### Option 2: Binary Installation

Download from [openobserve.ai](https://openobserve.ai/docs/quickstart/)

### Configuration

Add to your `config.json`:

```json
{
  "backends": {
    "openobserve": {
      "type": "openobserve",
      "url": "http://localhost:5080",
      "username": "admin@example.com",
      "password": "Complexpass#123",
      "organization": "default",
      "stream": "default",
      "verify_ssl": false
    }
  }
}
```

### Configuration Options

| Field | Description | Required | Default |
|-------|-------------|----------|---------|
| `url` | OpenObserve URL | Yes | - |
| `username` | Account email | Yes | - |
| `password` | Account password | Yes | - |
| `organization` | Organization name | Yes | `default` |
| `stream` | Stream name | Yes | `default` |
| `verify_ssl` | Verify SSL certificates | No | `false` |

### Creating Organization and Stream

1. **Via UI:**
   - Navigate to http://localhost:5080
   - Login with credentials
   - Go to Settings → Organizations
   - Create organization (or use "default")
   - Go to Logs → Streams
   - Create stream (or use "default")

2. **Via API:**
   ```bash
   # Create organization
   curl -X POST "http://localhost:5080/api/org" \
     -u admin@example.com:password \
     -H "Content-Type: application/json" \
     -d '{"name":"myorg"}'

   # Logs are automatically organized into streams
   ```

### Usage Examples

```bash
# Basic query
zeteo logs --backend openobserve --query "error" --max 50

# Filter by level
zeteo logs --backend openobserve --query "*" --level ERROR

# Filter by service and level
zeteo logs --backend openobserve \
  --query "payment" \
  --level ERROR \
  --service api-gateway \
  --max 100

# Time range query
zeteo logs --backend openobserve \
  --query "exception" \
  --start-time "2024-01-01T00:00:00Z" \
  --end-time "2024-01-02T00:00:00Z"
```

### Query Language

OpenObserve uses SQL-based queries internally:
- Text search: `error`, `payment failed`
- The query is converted to SQL WHERE clause
- Supports standard SQL comparison operators

### Ingesting Logs

To populate OpenObserve with logs:

```bash
# Via HTTP API
curl -X POST "http://localhost:5080/api/default/default/_json" \
  -u admin@example.com:password \
  -H "Content-Type: application/json" \
  -d '[
    {
      "timestamp": "2024-01-01T00:00:00Z",
      "level": "ERROR",
      "message": "Payment failed",
      "service_name": "payments"
    }
  ]'

# Via OpenTelemetry Collector
# Configure OTLP exporter to send to OpenObserve
```

### Troubleshooting

**Connection Error:**
```bash
# Check if OpenObserve is running
curl http://localhost:5080/healthz

# Check logs
docker logs openobserve
```

**Authentication Error:**
- Verify email and password
- Check organization name
- Ensure user has access to the organization

**No Results:**
- Verify organization and stream names
- Check if stream has data: UI → Logs → Streams
- Try query: `--query "*"`

---

## Kibana Setup

### Prerequisites
- Kibana instance running (version 7.x or 8.x)
- Network access to Kibana
- JWT authentication token (from browser)

### Configuration

Add to your `config.json`:

```json
{
  "backends": {
    "kibana": {
      "type": "kibana",
      "url": "http://localhost:5601",
      "auth_token": "your-jwt-token-here",
      "index_pattern": "logs-*",
      "verify_ssl": false,
      "version": "7.10.2"
    }
  }
}
```

### Configuration Options

| Field | Description | Required | Default |
|-------|-------------|----------|---------|
| `url` | Kibana URL | Yes | - |
| `auth_token` | JWT authentication token | No* | - |
| `index_pattern` | Elasticsearch index pattern | Yes | `logs-*` |
| `verify_ssl` | Verify SSL certificates | No | `false` |
| `version` | Kibana version | Yes | `7.10.2` |

\* Required if authentication is enabled

### Getting Authentication Token

#### Method 1: Browser Developer Tools

1. Open Kibana in your browser
2. Login to Kibana
3. Open Developer Tools (F12)
4. Go to **Application** tab (Chrome) or **Storage** tab (Firefox)
5. Navigate to **Cookies**
6. Find the authentication cookie (common names):
   - `sid` (session ID)
   - `auth_token`
   - JWT token in `Authorization` header
7. Copy the complete token value
8. Add to config.json:
   ```json
   "auth_token": "your-copied-token-here"
   ```

#### Method 2: Network Tab

1. Open Developer Tools (F12)
2. Go to **Network** tab
3. Refresh Kibana page
4. Look for API requests to Kibana
5. Check **Request Headers**
6. Find `Authorization: Bearer <token>`
7. Copy the token (without "Bearer ")

### Usage Examples

```bash
# Basic KQL query
zeteo logs --backend kibana --query "error OR exception" --max 50

# Query specific fields
zeteo logs --backend kibana \
  --query "level:ERROR AND service.name:payments" \
  --max 100

# Time range query
zeteo logs --backend kibana \
  --query "error" \
  --start-time "2024-01-01T00:00:00Z" \
  --end-time "2024-01-02T00:00:00Z"

# Complex KQL query
zeteo logs --backend kibana \
  --query "http.status_code >= 500 AND NOT user.agent: *bot*"
```

### Query Language (KQL)

Kibana uses KQL (Kibana Query Language):

**Basic Syntax:**
- Field queries: `level:ERROR`, `service.name:payments`
- Boolean operators: `error AND payment`, `error OR exception`, `NOT success`
- Wildcards: `pay*`, `err?r`
- Ranges: `http.status_code >= 400`, `bytes < 1000`
- Exists: `error.message:*`
- Grouping: `(error OR warning) AND service.name:api`

**Examples:**
```bash
# Errors in payment service
level:ERROR AND service.name:payments

# High response codes
http.status_code >= 500

# Exclude bot traffic
NOT user.agent:*bot* AND status_code:200

# Wildcard search
message:payment* AND level:ERROR

# Time-based (use --start-time/--end-time flags instead)
```

### Troubleshooting

**Connection Error:**
```bash
# Check if Kibana is running
curl http://localhost:5601/api/status

# Check Kibana logs
tail -f /var/log/kibana/kibana.log
```

**Authentication Error:**
- Token may have expired - get a new one
- Verify token is correctly copied (no extra spaces)
- Check if Kibana authentication is required
- Try without `auth_token` if authentication is disabled

**No Results:**
- Verify index pattern matches your indices
- Check if index pattern exists in Kibana
- Try query in Kibana UI first to verify
- Check Kibana version matches config

**Invalid Version:**
- Update `version` field to match your Kibana version
- Check version: `curl http://localhost:5601/api/status | jq .version.number`

---

## Backend Comparison

| Feature | Elasticsearch | OpenObserve | Kibana |
|---------|--------------|-------------|--------|
| **Query Language** | Lucene | SQL | KQL |
| **Authentication** | Basic Auth / API Key | Basic Auth | JWT Token |
| **Best For** | Standard ELK stack | Cloud-native setups | Visual exploration |
| **Setup Complexity** | Medium | Easy | Medium |
| **Performance** | High | High | Medium (via proxy) |
| **Real-time** | Yes | Yes | Yes |
| **Aggregations** | Yes | Yes | Yes |

## Tips and Best Practices

### General

1. **Start with small queries**: Use `--max 10` initially to test
2. **Use specific queries**: Avoid `--query "*"` for large datasets
3. **Export results**: Use `--export` to save results for analysis
4. **Time ranges**: Always specify time ranges for large datasets

### Elasticsearch

- Use index patterns efficiently: `logs-2024-*` instead of `logs-*`
- Leverage field-specific queries: `level:ERROR` is faster than full-text
- Use aggregations for statistics: `--aggregate`

### OpenObserve

- Create separate streams for different log types
- Use appropriate time ranges (OpenObserve is optimized for recent data)
- SQL syntax is converted automatically

### Kibana

- KQL is more intuitive than Lucene for complex queries
- Tokens expire - refresh regularly
- Use Kibana UI to test queries first

## Environment Variables

You can also set backend credentials via environment variables:

```bash
# Elasticsearch
export ELASTICSEARCH_URL="http://localhost:9200"
export ELASTICSEARCH_USERNAME="elastic"
export ELASTICSEARCH_PASSWORD="changeme"

# OpenObserve
export OPENOBSERVE_URL="http://localhost:5080"
export OPENOBSERVE_USERNAME="admin@example.com"
export OPENOBSERVE_PASSWORD="changeme"

# Kibana
export KIBANA_URL="http://localhost:5601"
export KIBANA_AUTH_TOKEN="your-token"
```

**Note**: Config file takes precedence over environment variables.

## Next Steps

- Read the main [README.md](../README.md) for general usage
- Check [examples/backend_usage.sh](../examples/backend_usage.sh) for more examples
- Join discussions in GitHub Issues for support

## Contributing

Found an issue or have a suggestion? Open an issue or pull request on GitHub!
