# Handler Lifecycle

Understanding how handlers are compiled, loaded, and managed helps you write more reliable code.

## Endpoint States

An endpoint can be in one of these states:

| State | Description |
|-------|-------------|
| **Created** | Endpoint defined but code not yet compiled |
| **Compiled** | Code compiled to dynamic library, ready to load |
| **Loaded** | Handler library loaded into gateway, handling requests |
| **Draining** | Old handler finishing in-flight requests during update |
| **Error** | Compilation or runtime error occurred |

## Compilation

When you click "Compile", the gateway:

1. **Creates a Cargo project** in the handlers directory
2. **Writes your code** to `src/lib.rs`
3. **Generates Cargo.toml** with the SDK dependency and `cdylib` crate type
4. **Runs `cargo build --release`** to compile
5. **Produces a dynamic library** (`.so`, `.dll`, or `.dylib`)

### Generated Project Structure

```
handlers/
└── {endpoint-id}/
    ├── Cargo.toml
    ├── Cargo.lock
    ├── src/
    │   └── lib.rs    # Your handler code
    └── target/
        └── release/
            └── libhandler_{id}.so  # Compiled library (Linux)
```

### Cargo.toml

The generated Cargo.toml includes the SDK and configures a dynamic library:

```toml
[package]
name = "handler_{endpoint_id}"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
rust-edge-gateway-sdk = { path = "../../crates/rust-edge-gateway-sdk" }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

## Handler Loading

### Loading a Handler

When you deploy an endpoint:

1. Gateway loads the dynamic library using `libloading`
2. Locates the `handler_entry` symbol (function pointer)
3. Registers the handler in the `HandlerRegistry`
4. Status changes to "Loaded"

### Request Flow

```
┌─────────────┐     ┌──────────────────┐     ┌─────────────────┐
│   Request   │────▶│  HandlerRegistry │────▶│  handler_entry  │
│             │     │  (lookup by ID)  │     │  (fn pointer)   │
│             │◀────│                  │◀────│                 │
└─────────────┘     └──────────────────┘     └─────────────────┘
```

The handler is called directly via function pointer - no serialization or IPC overhead.

### Handler Function

Your handler is an async function that receives a Context and Request:

```rust
use rust_edge_gateway_sdk::prelude::*;

#[handler]
pub async fn handle(ctx: &Context, req: Request) -> Response {
    // Access services via ctx
    // Process request
    // Return response
    Response::ok(json!({"status": "success"}))
}
```

The `#[handler]` macro generates the `handler_entry` symbol that the gateway looks for.

## Hot Swapping with Graceful Draining

Rust Edge Gateway supports zero-downtime updates with graceful draining:

### How It Works

1. **Compile new version** - New handler library is compiled
2. **Load new handler** - New library is loaded into memory
3. **Atomic swap** - New handler starts receiving new requests
4. **Drain old handler** - Old handler finishes in-flight requests
5. **Unload old handler** - Once drained, old library is unloaded

### Request Tracking

Each handler tracks active requests:

```
┌─────────────────────────────────────────────────────────────┐
│                    Handler Update Timeline                   │
├─────────────────────────────────────────────────────────────┤
│  Time ──────────────────────────────────────────────────▶   │
│                                                              │
│  Old Handler:  ████████████░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░  │
│                (handling)  (draining)  (unloaded)            │
│                                                              │
│  New Handler:  ░░░░░░░░░░░░████████████████████████████████  │
│                            (handling new requests)           │
│                                                              │
│                     ▲                                        │
│                     │ Swap point                             │
└─────────────────────────────────────────────────────────────┘
```

### Drain Timeout

If the old handler doesn't drain within the timeout (default: 30 seconds), it is forcefully unloaded. Configure this based on your longest expected request duration.

## Error Handling

### Compilation Errors

If compilation fails:
- Error message is captured and displayed
- Endpoint stays in previous state
- Previous library (if any) remains loaded

### Runtime Errors

If your handler panics:
- The panic is caught by the gateway
- Error is logged
- Other handlers continue working
- The specific request returns a 500 error

### Graceful Error Handling

Always handle errors in your code:

```rust
#[handler]
pub async fn handle(ctx: &Context, req: Request) -> Response {
    match process_request(ctx, &req).await {
        Ok(data) => Response::ok(data),
        Err(e) => e.to_response(), // HandlerError -> Response
    }
}

async fn process_request(ctx: &Context, req: &Request) -> Result<JsonValue, HandlerError> {
    let body: MyInput = req.json()
        .map_err(|e| HandlerError::ValidationError(e.to_string()))?;

    // Use services via ctx
    // ... process ...

    Ok(json!({"result": "success"}))
}
```

