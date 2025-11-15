#!/bin/bash
# Test script for Icarus learning system

ICARUS_BIN="./target/release/icarus-mcp"

# Start Icarus in background
echo "Starting Icarus MCP server..."
$ICARUS_BIN > icarus_output.log 2>&1 &
ICARUS_PID=$!
echo "Icarus started with PID: $ICARUS_PID"

# Give it time to start
sleep 2

# Function to send JSON-RPC request
send_request() {
    local request="$1"
    echo "$request" | nc localhost 3000 2>&1
}

# Initialize server
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test-client","version":"1.0"}}}' | $ICARUS_BIN

# Wait for response
sleep 1

# Send initialized notification
echo '{"jsonrpc":"2.0","method":"initialized"}' | $ICARUS_BIN

# Clean up
trap "kill $ICARUS_PID 2>/dev/null" EXIT

echo "Icarus MCP server ready for testing"
echo "PID: $ICARUS_PID"
echo "Send SIGTERM to stop: kill $ICARUS_PID"
