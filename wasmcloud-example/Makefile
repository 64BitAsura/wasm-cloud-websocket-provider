.PHONY: help build-provider build-component test-provider clean

help:
	@echo "WebSocket Provider Example - Make targets"
	@echo ""
	@echo "  build-provider    - Build the WebSocket provider"
	@echo "  build-component   - Build the echo component (using cargo)"
	@echo "  test-provider     - Run the provider in server mode for testing"
	@echo "  clean            - Clean build artifacts"
	@echo ""
	@echo "Note: For wasmCloud development, you can also use 'wash build'"
	@echo "in the echo-component directory."
	@echo ""

build-provider:
	@echo "Building WebSocket provider..."
	cd .. && cargo build --release
	@echo "✓ Provider built: ../target/release/websocket-provider"

build-component:
	@echo "Building echo component..."
	@echo "Using: cargo build --target wasm32-wasip1 --release"
	@echo "(Alternative: use 'wash build' if wash CLI is installed)"
	cd echo-component && cargo build --target wasm32-wasip1 --release
	@echo "✓ Component built: echo-component/target/wasm32-wasip1/release/echo_component.wasm"

test-provider:
	@echo "Starting WebSocket provider in server mode..."
	@echo "Connect with: wscat -c ws://localhost:8080/ws"
	@echo "Press Ctrl+C to stop"
	cd .. && cargo run --example server_mode

clean:
	@echo "Cleaning build artifacts..."
	cd .. && cargo clean
	cd echo-component && cargo clean
	@echo "✓ Clean complete"
