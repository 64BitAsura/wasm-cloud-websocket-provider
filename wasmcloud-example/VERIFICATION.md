# Verification Checklist

Use this checklist to verify that the WebSocket provider is working correctly with wasmCloud.

## ✅ Provider Verification

### 1. Build Verification

- [ ] Provider builds successfully
  ```bash
  cd /path/to/wasm-cloud-websocket-provider
  cargo build --release
  ```
  Expected: Build completes without errors

- [ ] Provider binary exists
  ```bash
  ls -lh target/release/websocket-provider
  ```
  Expected: Binary file is present

### 2. Server Mode Test

- [ ] Provider starts in server mode
  ```bash
  cargo run --example server_mode
  ```
  Expected output:
  ```
  INFO WebSocket Messaging Provider - Server Mode Example
  INFO Starting WebSocket server...
  INFO WebSocket server listening on 127.0.0.1:8080
  INFO WebSocket server started on 127.0.0.1:8080
  ```

- [ ] Port is listening
  ```bash
  # In another terminal
  netstat -an | grep 8080
  # or
  lsof -i :8080
  ```
  Expected: Port 8080 is in LISTEN state

- [ ] WebSocket connection works
  ```bash
  wscat -c ws://localhost:8080/ws
  ```
  Expected: Connection established, provider logs show new client

### 3. Message Handling Test

- [ ] Send JSON message
  ```json
  {"subject":"test.message","body":"dGVzdA==","reply_to":null}
  ```
  Expected: Provider receives and logs the message

- [ ] Check session tracking
  Expected: Provider logs show session ID assignment

## ✅ Component Verification

### 1. Build Verification

- [ ] Component builds successfully
  ```bash
  cd wasmcloud-example/echo-component
  cargo build --target wasm32-wasip1 --release
  ```
  Expected: Build completes without errors

- [ ] WASM file exists
  ```bash
  ls -lh target/wasm32-wasip1/release/echo_component.wasm
  ```
  Expected: WASM file is present (~13KB)

- [ ] Component is valid WebAssembly
  ```bash
  file target/wasm32-wasip1/release/echo_component.wasm
  ```
  Expected: Output indicates WebAssembly binary

### 2. WIT Interface Verification

- [ ] WIT files are present
  ```bash
  ls -la wit/
  ```
  Expected: `world.wit` and `deps/messaging/messaging.wit` exist

- [ ] WIT syntax is valid
  ```bash
  cat wit/world.wit
  ```
  Expected: Valid WIT syntax with proper imports/exports

## ✅ Integration Verification

### 1. Local Test Setup

- [ ] Quick test script runs
  ```bash
  cd wasmcloud-example
  ./test-local.sh
  ```
  Expected: Provider starts, shows help message

- [ ] Makefile targets work
  ```bash
  make help
  make build-provider
  make build-component
  ```
  Expected: All commands execute successfully

### 2. Documentation Verification

- [ ] README is comprehensive
  - Architecture diagram present
  - Prerequisites listed
  - Build instructions clear
  - Testing steps provided
  - Troubleshooting included

- [ ] QUICKSTART is clear
  - Step-by-step instructions
  - Browser test included
  - Multiple client options shown

## ✅ wasmCloud Integration (Optional)

If you have wasmCloud installed:

### 1. Host Setup

- [ ] wasmCloud host starts
  ```bash
  wash up
  ```
  Expected: Host starts and shows ready status

- [ ] Host is accessible
  ```bash
  wash get hosts
  ```
  Expected: At least one host listed

### 2. Component Deployment

- [ ] Component can be started
  ```bash
  wash start component file://./echo-component/target/wasm32-wasip1/release/echo_component.wasm echo-test
  ```
  Expected: Component starts successfully

- [ ] Component is listed
  ```bash
  wash get inventory
  ```
  Expected: echo-test component is listed

## 🎯 Success Criteria

All checks passed means:
- ✅ Provider builds and runs correctly
- ✅ Provider accepts WebSocket connections
- ✅ Provider manages sessions properly
- ✅ Component builds as valid WebAssembly
- ✅ Component implements messaging interface correctly
- ✅ Documentation is complete and accurate
- ✅ Example is ready for users to try

## 🔍 Common Issues

### Build Issues

**Problem**: Rust target not installed
```
error: can't find crate for `core`
```
**Solution**:
```bash
rustup target add wasm32-wasip1
```

**Problem**: wit-bindgen version mismatch
```
error: expected one of: `path`, `inline`, ...
```
**Solution**: Check Cargo.toml uses wit-bindgen 0.34

### Runtime Issues

**Problem**: Port already in use
```
Address already in use
```
**Solution**:
```bash
lsof -i :8080
kill <PID>
```

**Problem**: Connection refused
```
Connection refused
```
**Solution**: Ensure provider is running and listening on correct address

### Component Issues

**Problem**: WASM file not found
**Solution**: Run `cargo build --target wasm32-wasip1 --release` first

**Problem**: Invalid WIT syntax
**Solution**: Verify WIT files match the examples in the repository

## 📝 Notes

- All tests should be run from a clean state
- Stop any running providers before starting new tests
- Check logs for detailed error messages
- Use `RUST_LOG=debug` for more verbose output

## 🚀 Next Steps After Verification

1. Customize the echo component for your use case
2. Deploy to a wasmCloud cluster
3. Integrate with your WebSocket clients
4. Monitor performance and sessions
5. Contribute improvements back to the project!
