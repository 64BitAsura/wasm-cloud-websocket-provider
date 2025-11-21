#!/bin/bash
# Local test script for WebSocket provider
# This script runs a simple test without requiring full wasmCloud setup

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

echo "================================================"
echo "WebSocket Provider Local Test"
echo "================================================"
echo ""

# Check if the provider binary exists
PROVIDER_BIN="$REPO_ROOT/target/release/websocket-provider"
if [ ! -f "$PROVIDER_BIN" ]; then
    echo "Building WebSocket provider..."
    cd "$REPO_ROOT"
    cargo build --release
    echo "✓ Provider built successfully"
else
    echo "✓ Provider binary found at: $PROVIDER_BIN"
fi

# Check if server_mode example exists
SERVER_EXAMPLE="$REPO_ROOT/examples/server_mode.rs"
if [ ! -f "$SERVER_EXAMPLE" ]; then
    echo "Error: server_mode example not found at $SERVER_EXAMPLE"
    exit 1
fi

echo ""
echo "Starting WebSocket provider in server mode..."
echo "The provider will listen on ws://localhost:8080/ws"
echo ""
echo "To test:"
echo "  1. Install wscat: npm install -g wscat"
echo "  2. Connect: wscat -c ws://localhost:8080/ws"
echo "  3. Send a message: {\"subject\":\"test\",\"body\":\"dGVzdA==\",\"reply_to\":null}"
echo ""
echo "Press Ctrl+C to stop the provider"
echo ""

# Run the server mode example
cd "$REPO_ROOT"
cargo run --example server_mode
