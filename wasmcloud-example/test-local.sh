#!/bin/bash
# Local test script for WebSocket provider
# This script runs a simple test without requiring full wasmCloud setup

set -e

echo "================================================"
echo "WebSocket Provider Local Test"
echo "================================================"
echo ""

# Check if the provider is built
if [ ! -f "../target/release/websocket-provider" ]; then
    echo "Building WebSocket provider..."
    cd ..
    cargo build --release
    cd wasmcloud-example
    echo "✓ Provider built successfully"
else
    echo "✓ Provider binary found"
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
cd ..
cargo run --example server_mode
