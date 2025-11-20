# OpenObserve Integration Tests

This directory contains integration tests for verifying that zeteo-cli can correctly fetch and process logs from OpenObserve dashboard using the otel-mcp-server.

## Prerequisites

1. **Node.js** - Required for otel-mcp-server
2. **OpenAI API Key** - Required by otel-mcp-server for ML embedding features

## Running the Tests

### Set the OpenAI API Key

```bash
export OPENAI_API_KEY="your-api-key-here"
```

### Run All Integration Tests

```bash
cargo test --test openobserve_integration_tests -- --nocapture
```

### Run a Specific Test

```bash
cargo test --test openobserve_integration_tests test_mcp_client_connection -- --exact --nocapture
```

## Test Coverage

The test suite includes:

1. **Connection Tests**
   - `test_mcp_client_connection` - Verifies MCP client can connect to otel-mcp-server
   
2. **Query Tests**
   - `test_query_logs_basic` - Tests basic wildcard queries
   - `test_query_logs_with_error_filter` - Tests filtering by log level
   - `test_query_logs_with_specific_search` - Tests search terms (exception, timeout, failed, success)
   - `test_query_logs_different_limits` - Tests result pagination (1, 5, 10, 20, 50 results)
   
3. **Integration Tests**
   - `test_log_explorer_integration` - Tests LogExplorer with OpenObserve config
   
4. **Error Handling Tests**
   - `test_error_handling_invalid_credentials` - Verifies proper error handling
   - `test_concurrent_queries` - Tests multiple sequential queries

## OpenObserve Configuration

The tests are configured to connect to:
- **URL**: `https://periscope.breeze.in/openobserve`
- **Username**: `support@breeze.in`
- **Password**: `Breeze@123`

## Test Output

Tests will output detailed information about:
- Connection status
- Number of logs fetched
- Sample log entries
- Errors encountered

Use `-- --nocapture` flag to see all output.

## Known Issues

- The otel-mcp-server may not respond if the OpenAI API key is invalid
- Connection tests may timeout if Node.js or npx is not available
- Tests skip gracefully if the MCP server cannot be initialized

## Troubleshooting

### "Could not create MCP client" Error

Ensure Node.js and npx are installed:
```bash
node --version
npx --version
```

### "MCP client initialization failed" Error

1. Check that the OpenAI API key is set and valid
2. Verify network connectivity to OpenObserve
3. Check that otel-mcp-server can be installed: `npx -y otel-mcp-server --help`

### Timeout Errors

The MCP server may take time to start. If tests consistently timeout, try:
1. Running tests with longer timeout (modify MAX_ATTEMPTS in the test code)
2. Testing the otel-mcp-server manually
3. Checking firewall/network restrictions
