# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2024-11-18

### Added
- Initial implementation of WebSocket messaging provider based on wasmCloud NATS provider
- WebSocket connection management with support for ws:// and wss:// protocols
- Session tracking and management for individual WebSocket connections
- Session-specific message routing capabilities
- Flexible configuration system with support for:
  - Custom URIs
  - Authentication tokens
  - Connection timeouts
  - Custom HTTP headers
  - Session tracking toggle
- Bidirectional WebSocket communication
- Component link lifecycle management (connect, disconnect, shutdown)
- Thread-safe concurrent access using Arc<RwLock>
- Comprehensive error handling and logging
- Message encoding/decoding (JSON format with base64 payload)
- Configuration merging for combining default and link-specific settings

### Testing
- 6 unit tests covering configuration and provider state
- 6 integration tests covering session lifecycle, concurrent access, and shutdown
- Example application demonstrating basic usage
- All tests passing with 100% success rate

### Documentation
- README.md with features, configuration, and usage examples
- CONTRIBUTING.md with development setup and contribution guidelines
- TECHNICAL.md with architecture details and design decisions
- CONFIG.md with configuration examples and troubleshooting
- Inline code documentation throughout the codebase
- CI/CD workflow configuration

### Infrastructure
- GitHub Actions CI workflow for automated testing
- Cargo.toml with all necessary dependencies
- WIT interface definitions for wasmCloud messaging contract
- Example code demonstrating provider usage

### Known Limitations
- Request-reply pattern not fully implemented (placeholder only)
- Automatic reconnection not yet implemented
- No message acknowledgment system
- No compression support yet
- No connection pooling for multiple components to same server

### Future Enhancements
See TECHNICAL.md for planned features and optimizations.

[0.1.0]: https://github.com/64BitAsura/wasm-cloud-websocket-provider/releases/tag/v0.1.0
