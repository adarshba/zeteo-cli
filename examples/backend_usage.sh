#!/bin/bash
# Example script demonstrating different backend usage with zeteo-cli

set -e

echo "=========================================="
echo "Zeteo CLI Backend Examples"
echo "=========================================="
echo ""

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Example 1: Using Elasticsearch backend
echo -e "${BLUE}Example 1: Querying Elasticsearch${NC}"
echo "Command: zeteo logs --backend elasticsearch --query \"error\" --max 10"
echo "Description: Search for 'error' in Elasticsearch logs, limited to 10 results"
echo ""

# Example 2: Using OpenObserve backend with filters
echo -e "${BLUE}Example 2: Querying OpenObserve with filters${NC}"
echo "Command: zeteo logs --backend openobserve --query \"payment\" --level ERROR --service api-gateway --max 20"
echo "Description: Search OpenObserve for payment-related ERROR logs from api-gateway service"
echo ""

# Example 3: Using Kibana backend with time range
echo -e "${BLUE}Example 3: Querying Kibana with time range${NC}"
echo "Command: zeteo logs --backend kibana --query \"exception\" --start-time \"2024-01-01T00:00:00Z\" --end-time \"2024-01-02T00:00:00Z\" --max 50"
echo "Description: Search Kibana for exceptions within a specific time range"
echo ""

# Example 4: Using default MCP server
echo -e "${BLUE}Example 4: Using default MCP server (Elasticsearch via otel-mcp-server)${NC}"
echo "Command: zeteo logs --query \"error\" --max 10"
echo "Description: Query logs using the default MCP server integration"
echo ""

# Example 5: Export results to JSON
echo -e "${BLUE}Example 5: Export logs to JSON file${NC}"
echo "Command: zeteo logs --backend openobserve --query \"*\" --level ERROR --max 100 --export errors.json"
echo "Description: Query ERROR logs and export to JSON file"
echo ""

# Example 6: Show aggregated statistics
echo -e "${BLUE}Example 6: Show log statistics${NC}"
echo "Command: zeteo logs --backend elasticsearch --query \"*\" --aggregate --max 1000"
echo "Description: Show aggregated statistics of logs (by level, service, etc.)"
echo ""

echo "=========================================="
echo -e "${GREEN}Configuration${NC}"
echo "=========================================="
echo ""
echo "Configure backends in: ~/.config/zeteo-cli/config.json"
echo ""
echo "Example configuration:"
echo '{'
echo '  "backends": {'
echo '    "elasticsearch": {'
echo '      "type": "elasticsearch",'
echo '      "url": "http://localhost:9200",'
echo '      "username": "elastic",'
echo '      "password": "changeme",'
echo '      "index_pattern": "logs-*",'
echo '      "verify_ssl": false'
echo '    },'
echo '    "openobserve": {'
echo '      "type": "openobserve",'
echo '      "url": "http://localhost:5080",'
echo '      "username": "admin@example.com",'
echo '      "password": "changeme",'
echo '      "organization": "default",'
echo '      "stream": "default",'
echo '      "verify_ssl": false'
echo '    },'
echo '    "kibana": {'
echo '      "type": "kibana",'
echo '      "url": "http://localhost:5601",'
echo '      "auth_token": null,'
echo '      "index_pattern": "logs-*",'
echo '      "verify_ssl": false,'
echo '      "version": "7.10.2"'
echo '    }'
echo '  }'
echo '}'
echo ""

echo "=========================================="
echo -e "${GREEN}Quick Start${NC}"
echo "=========================================="
echo ""
echo "1. Install zeteo-cli:"
echo "   cargo install --path ."
echo ""
echo "2. Initialize configuration:"
echo "   zeteo config --init"
echo ""
echo "3. Edit configuration with your backend credentials:"
echo "   nano ~/.config/zeteo-cli/config.json"
echo ""
echo "4. Test your configuration:"
echo "   zeteo logs --backend elasticsearch --query \"*\" --max 5"
echo ""
echo "5. For help with any command:"
echo "   zeteo logs --help"
echo ""
