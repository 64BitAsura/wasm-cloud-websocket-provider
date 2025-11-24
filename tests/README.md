# Test Suite Documentation

This directory contains the test suite for the wasmCloud WebSocket Provider.

## Test Structure

### Unit and Integration Tests

Regular tests run with `cargo test` (not ignored):

- **`integration_test.rs`**: Core provider functionality tests
  - Message creation and handling
  - Configuration merging
  - Provider state management
  - Session lifecycle
  - Concurrent access
  - Shutdown procedures

- **`server_mode_test.rs`**: Server mode functionality tests
  - Server initialization
  - Client mode behavior
  - Session routing
  - Broadcast functionality
  - WebSocket client management
  - Reply-to field handling
  - Session tracking configuration

- **`client_broadcast_test.rs`**: Client mode broadcast functionality tests
  - Message parsing and encoding
  - Client mode broadcast to handlers (requires network, ignored)
  - Component reply-to functionality (requires network, ignored)
  - Multiple handlers broadcast (requires network, ignored)
  - Session tracking (requires network, ignored)

### Example Integration Tests

Slow tests marked with `#[ignore]` that run the actual examples, executed with `cargo test -- --ignored`:

- **`example_server_mode_test.rs`**: Tests the `server_mode` example
  - Starts the server_mode example as a subprocess
  - Creates a WebSocket client to connect to the server
  - Sends ping, pong, and echo messages
  - Verifies server accepts connections and handles messages
  - Properly cleans up processes
  
- **`example_client_mode_test.rs`**: Tests client mode with echo server
  - Creates a simple WebSocket echo server
  - Tests client connections and message echoing
  - Verifies ping, pong, and echo messages are handled correctly
  - Tests multiple message formats (JSON, plain text, binary)
  - Tests multiple concurrent clients

## Running Tests

### Run all non-ignored tests (fast):
```bash
cargo test
```

### Run example integration tests (slow):
```bash
cargo test -- --ignored
```

### Run specific test suite:
```bash
cargo test --test integration_test
cargo test --test server_mode_test
cargo test --test example_server_mode_test -- --ignored
```

### Run with output:
```bash
cargo test -- --nocapture
cargo test -- --ignored --nocapture
```

## CI Integration

The CI workflow (`.github/workflows/ci.yml`) runs:

1. Regular tests: `cargo test --verbose`
2. Example integration tests: `cargo test --ignored --verbose --test example_server_mode_test --test example_client_mode_test`

This ensures both the provider code and the examples are tested as part of continuous integration.

## Test Requirements

### Example Integration Tests

The example integration tests require:
- Building the examples (adds ~10-15 seconds per test)
- Starting subprocesses
- Network connectivity (localhost only)
- Proper cleanup of spawned processes

These are marked as `#[ignore]` to avoid slowing down regular development workflows but are run in CI to ensure examples remain functional.

## Writing New Tests

When adding new tests:

1. **Fast unit tests**: Add to existing test files or create new ones
2. **Network-dependent tests**: Mark with `#[ignore = "requires network access"]`
3. **Slow integration tests**: Mark with `#[ignore = "requires building and running example"]`
4. **Example tests**: Follow the pattern in `example_*_test.rs` files:
   - Spawn examples as subprocesses
   - Use `kill_on_drop(true)` for cleanup
   - Add proper timeouts
   - Log important events with `info!`
   - Handle expected errors gracefully

## Debugging Test Failures

### Enable detailed logging:
```bash
RUST_LOG=debug cargo test -- --nocapture
```

### Run a single test:
```bash
cargo test test_name -- --nocapture
```

### Example integration test debugging:
The example tests log subprocess output, so run with `--nocapture` to see what's happening:
```bash
cargo test --test example_server_mode_test test_server_mode_example_with_ping_pong -- --ignored --nocapture
```

## Test Coverage

Current test coverage includes:
- ✅ Provider initialization and configuration
- ✅ Server mode operation
- ✅ Client mode operation
- ✅ Message handling (parsing, encoding, routing)
- ✅ Session management
- ✅ Broadcast functionality
- ✅ Concurrent operations
- ✅ Graceful shutdown
- ✅ Example applications (server and client modes)
- ✅ WebSocket connection handling
- ✅ Ping/pong/echo message flows

## Continuous Integration

The CI pipeline ensures:
1. Code compiles without warnings (`cargo clippy -- -D warnings`)
2. Code is properly formatted (`cargo fmt -- --check`)
3. All unit and integration tests pass (`cargo test --verbose`)
4. Example applications run correctly (`cargo test --ignored --verbose`)
5. Security audits pass (`cargo audit`)
