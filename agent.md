# Agent Development Guidelines

This document outlines the development practices and standards for contributing to the WebSocket Messaging Provider project as a senior Rust developer.

## Core Principles

### 1. Atomic Commits
- Each commit should represent a single, complete, and logical change
- Commits should be self-contained and independently reversible
- A commit should not break the build or tests
- Commit messages follow conventional commit standards

### 2. Conventional Commit Messages

All commit messages must follow the [Conventional Commits](https://www.conventionalcommits.org/) specification:

```
<type>[optional scope]: <description>

[optional body]

[optional footer(s)]
```

**Types:**
- `feat`: New feature for the user
- `fix`: Bug fix for the user
- `docs`: Documentation changes
- `style`: Code style changes (formatting, missing semi-colons, etc.)
- `refactor`: Code change that neither fixes a bug nor adds a feature
- `perf`: Performance improvements
- `test`: Adding or updating tests
- `chore`: Maintenance tasks, dependency updates
- `ci`: CI/CD configuration changes
- `build`: Build system or external dependencies changes
- `revert`: Reverting a previous commit

**Examples:**
```
feat(websocket): add session tracking for client connections
fix(server): resolve race condition in client disconnection
docs(readme): update configuration examples
refactor(connection): simplify connection pool management
test(integration): add end-to-end test for broadcast feature
```

### 3. Single Responsibility Principle

Each function must do **only one thing**:

```rust
// ❌ Bad: Function does multiple things
async fn connect_and_send_message(uri: &str, message: String) -> Result<()> {
    let ws = connect_websocket(uri).await?;
    send_message(&ws, message).await?;
    close_connection(ws).await?;
    Ok(())
}

// ✅ Good: Each function has a single responsibility
async fn connect_websocket(uri: &str) -> Result<WebSocket> {
    // Only handles connection establishment
}

async fn send_message(ws: &WebSocket, message: String) -> Result<()> {
    // Only handles message sending
}

async fn close_connection(ws: WebSocket) -> Result<()> {
    // Only handles connection closure
}
```

### 4. No Unnecessary Code Reduction

**Avoid premature optimization or excessive abstraction:**
- Prefer clarity over cleverness
- Don't force code reuse where it reduces readability
- Duplicate simple code rather than create complex abstractions
- Only extract common functionality when it genuinely improves maintainability

```rust
// ❌ Bad: Over-engineered abstraction
fn apply<F, T>(value: T, f: F) -> T 
where F: Fn(T) -> T {
    f(value)
}

// ✅ Good: Clear and direct
fn validate_session_id(session_id: &str) -> Result<()> {
    if session_id.is_empty() {
        return Err(anyhow::anyhow!("Session ID cannot be empty"));
    }
    Ok(())
}

fn validate_uri(uri: &str) -> Result<()> {
    if uri.is_empty() {
        return Err(anyhow::anyhow!("URI cannot be empty"));
    }
    Ok(())
}
```

## Testing Strategy

### End-to-End Testing Priority

**Prefer end-to-end tests over unit tests** to validate complete workflows:

```rust
// ✅ Preferred: End-to-end test covering complete workflow
#[tokio::test]
async fn test_complete_websocket_message_flow() {
    // Setup: Create provider, establish connection
    let provider = setup_test_provider().await;
    
    // Action: Send message through complete pipeline
    let message = create_test_message();
    provider.publish("test-component", message.clone()).await.unwrap();
    
    // Verification: Validate message received and processed
    let received = wait_for_message_receipt().await;
    assert_eq!(received.body, message.body);
    
    // Cleanup
    teardown_test_provider(provider).await;
}
```

**Unit tests are acceptable for:**
- Complex algorithms requiring validation
- Edge cases in parsing or validation logic
- Error handling paths

### Test Coverage Guidelines

1. **Happy Path**: Test successful execution flows
2. **Error Cases**: Test failure scenarios and error handling
3. **Edge Cases**: Test boundary conditions and unusual inputs
4. **Integration Points**: Test interactions between components
5. **Concurrency**: Test thread-safety and race conditions

## Code Quality Standards

### Security First

Security is a top priority:

1. **Input Validation**: Always validate external inputs
2. **Error Handling**: Never expose sensitive information in errors
3. **Resource Management**: Ensure proper cleanup of resources
4. **Dependency Audit**: Regularly audit dependencies for vulnerabilities
5. **Secure Defaults**: Use secure configuration defaults

```rust
// ✅ Good: Proper input validation and error handling
async fn process_incoming_message(raw_message: &[u8]) -> Result<BrokerMessage> {
    // Validate input size to prevent DoS
    if raw_message.len() > MAX_MESSAGE_SIZE {
        return Err(anyhow::anyhow!("Message exceeds maximum size"));
    }
    
    // Safely parse JSON with error handling
    let message: BrokerMessage = serde_json::from_slice(raw_message)
        .map_err(|e| anyhow::anyhow!("Failed to parse message: {}", e))?;
    
    // Validate message fields
    validate_message_fields(&message)?;
    
    Ok(message)
}
```

### Readability

Code should be self-documenting and easy to understand:

1. **Descriptive Names**: Use clear, descriptive variable and function names
2. **Type Clarity**: Leverage Rust's type system for clarity
3. **Avoid Magic Values**: Use constants with meaningful names
4. **Logical Structure**: Organize code in a logical, top-down manner
5. **Minimal Nesting**: Keep nesting levels shallow

```rust
// ❌ Bad: Unclear and hard to follow
async fn h(s: &str, t: i32) -> Result<bool> {
    if t > 30 { return Err(anyhow::anyhow!("err")); }
    let x = c(s).await?;
    Ok(x.is_some())
}

// ✅ Good: Clear and descriptive
async fn handle_connection_with_timeout(
    uri: &str,
    timeout_seconds: i32
) -> Result<bool> {
    // Validate timeout is within acceptable range
    if timeout_seconds > MAX_TIMEOUT_SECONDS {
        return Err(anyhow::anyhow!("Timeout exceeds maximum allowed value"));
    }
    
    // Attempt to establish connection
    let connection = connect_with_timeout(uri, timeout_seconds).await?;
    
    // Check if connection was successful
    Ok(connection.is_some())
}
```

## Code Documentation

### Comment Standards

**When to Comment:**

1. **Why, Not What**: Explain the reasoning behind non-obvious decisions
2. **Complex Logic**: Clarify intricate algorithms or business logic
3. **Workarounds**: Document temporary solutions and their reasoning
4. **Public APIs**: Document all public functions and types
5. **Safety**: Explain any unsafe code usage

```rust
// ✅ Good: Comments explain the "why"
/// Processes incoming WebSocket messages and routes them to appropriate handlers.
///
/// This function implements a broadcast mechanism where messages from a remote
/// WebSocket server are distributed to all registered handler components. This
/// design allows multiple components to react to the same message independently.
///
/// # Arguments
/// * `raw_message` - Raw bytes received from the WebSocket connection
/// * `session_id` - Unique identifier for the WebSocket session
///
/// # Returns
/// * `Ok(())` if message was successfully processed and routed
/// * `Err(...)` if parsing or routing failed
///
/// # Example
/// ```no_run
/// let result = process_and_broadcast_message(&message_bytes, &session_id).await;
/// ```
async fn process_and_broadcast_message(
    raw_message: &[u8],
    session_id: &str
) -> Result<()> {
    // Parse the raw message into a structured format
    // Note: We use serde_json instead of binary formats for better
    // compatibility with JavaScript clients in browser environments
    let message = parse_websocket_message(raw_message)?;
    
    // Retrieve all handler components registered for this session
    // We intentionally iterate over all handlers rather than using
    // a single handler to support multiple independent consumers
    let handlers = get_registered_handlers(session_id).await?;
    
    // Broadcast to all handlers concurrently for better performance
    // Using join_all ensures we wait for all broadcasts to complete
    // before considering the operation successful
    let broadcast_tasks: Vec<_> = handlers
        .iter()
        .map(|handler| broadcast_to_handler(handler, &message))
        .collect();
    
    futures::future::join_all(broadcast_tasks).await;
    
    Ok(())
}
```

**When Not to Comment:**

```rust
// ❌ Bad: Commenting the obvious
// Increment counter by 1
counter += 1;

// ✅ Good: Self-documenting code
message_count += 1;
```

## Development Workflow

### Step 1: Design Phase - Show Steps and Functions

**Before writing any code**, document:

1. **High-level Overview**: What is the feature or fix?
2. **Function List**: What functions will be needed?
3. **Control Flow**: How do functions interact?
4. **Data Structures**: What types are involved?

**Example Design Document:**

```markdown
## Feature: Add Connection Pooling

### Overview
Implement connection pooling to reuse WebSocket connections and improve performance.

### Functions Required

1. `create_connection_pool(max_size: usize) -> ConnectionPool`
   - Purpose: Initialize a new connection pool
   - Input: Maximum number of connections
   - Output: ConnectionPool instance

2. `acquire_connection(pool: &ConnectionPool) -> Result<Connection>`
   - Purpose: Get an available connection from the pool
   - Input: Reference to connection pool
   - Output: Connection or error if pool exhausted

3. `release_connection(pool: &ConnectionPool, conn: Connection) -> Result<()>`
   - Purpose: Return connection to the pool for reuse
   - Input: Pool reference and connection to return
   - Output: Success or error

4. `cleanup_idle_connections(pool: &ConnectionPool) -> usize`
   - Purpose: Remove connections that have been idle too long
   - Input: Pool reference
   - Output: Number of connections removed

### Control Flow

```
[Component Request]
       ↓
[acquire_connection] ─→ Pool has connection? ─Yes→ [Return existing connection]
       ↓                                   ↓
       No                                  |
       ↓                                   |
[create_new_connection]                    |
       ↓                                   |
[add_to_pool] ←─────────────────────────┘
       ↓
[Use connection for communication]
       ↓
[release_connection] ─→ [Return to pool]
```

### Data Structures

```rust
struct ConnectionPool {
    available: Arc<Mutex<Vec<Connection>>>,
    in_use: Arc<Mutex<HashSet<ConnectionId>>>,
    max_size: usize,
    idle_timeout: Duration,
}

struct Connection {
    id: ConnectionId,
    websocket: WebSocket,
    last_used: Instant,
}
```
```

### Step 2: Request Approval

**Before implementation:**
1. Present the design document
2. Show the control flow diagram
3. List all functions with their signatures
4. Wait for approval or feedback
5. Iterate on design if needed

### Step 3: Implementation

Only after approval:
1. Implement functions one at a time
2. Follow atomic commit guidelines
3. Write end-to-end tests
4. Ensure code passes linting and security checks

### Step 4: Verification

Before committing:
```bash
# Format code
cargo fmt

# Check for common mistakes
cargo clippy -- -D warnings

# Run tests
cargo test

# Check for security vulnerabilities
cargo audit
```

## Code Review Checklist

Before submitting a PR, verify:

- [ ] Each commit is atomic and has a conventional commit message
- [ ] Each function does only one thing
- [ ] No unnecessary code abstractions or "clever" solutions
- [ ] End-to-end tests cover the main workflows
- [ ] Code is secure (input validation, error handling)
- [ ] Code is readable (clear names, logical structure)
- [ ] Important decisions are commented
- [ ] Public APIs have documentation comments
- [ ] Design was documented and approved before implementation
- [ ] All tests pass
- [ ] Code passes `cargo fmt` and `cargo clippy`
- [ ] No new security vulnerabilities introduced

## Best Practices Summary

### DO:
✅ Write atomic commits with conventional messages  
✅ Keep functions focused on single responsibility  
✅ Prioritize end-to-end tests  
✅ Validate all external inputs  
✅ Use descriptive names for clarity  
✅ Comment the "why", not the "what"  
✅ Show design and control flow before coding  
✅ Wait for approval before implementation  
✅ Run linters and tests before committing  

### DON'T:
❌ Make commits that do multiple unrelated things  
❌ Create functions that do multiple operations  
❌ Over-engineer or prematurely optimize  
❌ Only write unit tests without integration tests  
❌ Skip input validation  
❌ Use unclear or abbreviated names  
❌ Add obvious comments  
❌ Write code before designing and getting approval  
❌ Commit without running tests and linters  

## Example: Complete Feature Implementation

### 1. Design Phase

```markdown
## Feature: Add Message Rate Limiting

### Overview
Prevent abuse by limiting message rate per session.

### Functions

1. `create_rate_limiter(max_per_second: u32) -> RateLimiter`
2. `check_rate_limit(limiter: &RateLimiter, session_id: &str) -> Result<()>`
3. `reset_rate_limit(limiter: &RateLimiter, session_id: &str) -> Result<()>`

### Control Flow
[Incoming Message] → [check_rate_limit] → Allowed? → [Process Message]
                                        ↓
                                        Denied → [Return Rate Limit Error]
```

### 2. Get Approval
*Wait for design approval...*

### 3. Implementation (Atomic Commits)

**Commit 1:**
```
feat(rate-limit): add rate limiter data structure

- Define RateLimiter struct with session tracking
- Implement creation and initialization logic
```

**Commit 2:**
```
feat(rate-limit): implement rate limit checking logic

- Add check_rate_limit function with sliding window algorithm
- Return appropriate errors when limit exceeded
```

**Commit 3:**
```
test(rate-limit): add end-to-end rate limiting tests

- Test successful message processing under limit
- Test rejection when limit exceeded
- Test rate limit reset functionality
```

### 4. Testing

```rust
#[tokio::test]
async fn test_rate_limiting_end_to_end() {
    // Setup: Create provider with rate limiting enabled
    let config = create_config_with_rate_limit(10); // 10 msg/sec
    let provider = WebSocketMessagingProvider::from_config(config).await.unwrap();
    
    // Action: Send messages within limit
    for i in 0..10 {
        let result = provider.publish("test-component", create_message(i)).await;
        assert!(result.is_ok(), "Message {} should be accepted", i);
    }
    
    // Action: Attempt to exceed limit
    let result = provider.publish("test-component", create_message(11)).await;
    assert!(result.is_err(), "Message 11 should be rate limited");
    
    // Verify: Check error type
    match result {
        Err(e) => assert!(e.to_string().contains("rate limit")),
        _ => panic!("Expected rate limit error"),
    }
    
    // Cleanup
    provider.shutdown().await;
}
```

## Conclusion

Following these guidelines ensures:
- **Maintainability**: Clear, focused code that's easy to modify
- **Security**: Robust validation and error handling
- **Reliability**: Comprehensive end-to-end testing
- **Collaboration**: Clear communication through design docs and conventional commits
- **Quality**: High code standards through reviews and automated checks

Always remember: **Security and readability are top priorities**. Write code that others (including your future self) can easily understand and trust.
