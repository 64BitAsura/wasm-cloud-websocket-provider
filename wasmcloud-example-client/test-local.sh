#!/bin/bash
# Local test script for WebSocket provider in CLIENT mode
# This script runs a simple test without requiring full wasmCloud setup

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

echo "================================================"
echo "WebSocket Provider Client Mode Test"
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

# Check if client_broadcast example exists
CLIENT_EXAMPLE="$REPO_ROOT/examples/client_broadcast.rs"
if [ ! -f "$CLIENT_EXAMPLE" ]; then
    echo "Error: client_broadcast example not found at $CLIENT_EXAMPLE"
    exit 1
fi

echo ""
echo "Starting WebSocket provider in client mode..."
echo "The provider will connect to ws://echo.websocket.org"
echo ""
echo "This example demonstrates:"
echo "  ✓ Connecting to remote WebSocket server"
echo "  ✓ Sending messages to remote server"
echo "  ✓ Receiving messages from remote server"
echo "  ✓ Bidirectional communication"
echo ""
echo "Press Ctrl+C to stop"
echo ""

# Run the client mode example
cd "$REPO_ROOT"
cargo run --example client_broadcast
