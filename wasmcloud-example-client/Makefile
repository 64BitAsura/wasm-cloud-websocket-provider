.PHONY: help build-provider build-component test-provider clean

help:
	@echo "WebSocket Provider Client Mode Example - Make targets"
	@echo ""
	@echo "  build-provider    - Build the WebSocket provider"
	@echo "  build-component   - Build the client component (using cargo)"
	@echo "  test-provider     - Run the provider in client mode for testing"
	@echo "  clean            - Clean build artifacts"
	@echo ""
	@echo "Note: For wasmCloud development, you can also use 'wash build'"
	@echo "in the client-component directory."
	@echo ""

build-provider:
	@echo "Building WebSocket provider..."
	cd .. && cargo build --release
	@echo "✓ Provider built: ../target/release/websocket-provider"

build-component:
	@echo "Building client component..."
	@echo "Using: cargo build --target wasm32-wasip1 --release"
	@echo "(Alternative: use 'wash build' if wash CLI is installed)"
	cd client-component && cargo build --target wasm32-wasip1 --release
	@echo "✓ Component built: client-component/target/wasm32-wasip1/release/client_component.wasm"

test-provider:
	@echo "Starting WebSocket provider in client mode..."
	@echo "This will connect to ws://echo.websocket.org and demonstrate bidirectional communication"
	@echo "Press Ctrl+C to stop"
	cd .. && cargo run --example client_broadcast

clean:
	@echo "Cleaning build artifacts..."
	cd .. && cargo clean
	cd client-component && cargo clean
	@echo "✓ Clean complete"
