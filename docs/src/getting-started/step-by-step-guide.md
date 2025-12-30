# Step-by-Step Guide: Creating Endpoint Handlers and Service Providers

This guide provides clear, step-by-step instructions for creating Endpoint Handlers and using Service Providers in Rust Edge Gateway.

## Table of Contents

- [Prerequisites](#prerequisites)
- [Creating Your First Handler](#creating-your-first-handler)
- [Using Service Providers](#using-service-providers)
- [Handler Lifecycle](#handler-lifecycle)
- [Best Practices](#best-practices)

## Prerequisites

Before you begin, ensure you have:

1. Rust toolchain installed (stable or nightly)
2. Rust Edge Gateway SDK added to your project
3. Basic understanding of async Rust programming

## Creating Your First Handler

### Step 1: Set up your handler project

```bash
# Create a new Rust library project
cargo new --lib my_handler
cd my_handler
```

### Step 2: Add dependencies to Cargo.toml

```toml
[package]
name = "my_handler"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]  # Important: This creates a dynamic library

[dependencies]
rust-edge-gateway-sdk = { path = "../rust-edge-gateway-sdk" }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1.0", features = ["full"] }
```

### Step 3: Create a basic handler

Create `src/lib.rs` with the following content:

```rust
use rust_edge_gateway_sdk::prelude::*;

// Basic handler that returns a simple JSON response
handler!(async fn hello_world(ctx: &Context, req: Request) -> Response {
    Response::ok(json!({
        "message": "Hello, World!",
        "path": req.path,
        "method": req.method
    }))
});
```

### Step 4: Build your handler

```bash
cargo build --release
```

This will create a dynamic library in `target/release/` that can be loaded by the gateway.

## Using Service Providers

### Step 1: Accessing Database Services

```rust
use rust_edge_gateway_sdk::prelude::*;

handler_result!(async fn get_users(ctx: &Context, req: Request) -> Result<Response, HandlerError> {
    // Access the database service
    let db = ctx.services.require_db()?;
    
    // Execute a query
    let users = db.query(
        "SELECT id, name, email FROM users WHERE active = $1",
        &[&true]
    ).await?;
    
    Ok(Response::ok(json!({
        "count": users.len(),
        "users": users
    })))
});
```

### Step 2: Using Cache Services

```rust
use rust_edge_gateway_sdk::prelude::*;

handler_result!(async fn cached_data(ctx: &Context, req: Request) -> Result<Response, HandlerError> {
    let cache = ctx.services.require_cache()?;
    let cache_key = "api_response_data";
    
    // Try to get data from cache
    if let Some(cached) = cache.get(cache_key).await? {
        return Ok(Response::ok(json!({
            "source": "cache",
            "data": cached
        })));
    }
    
    // Cache miss - fetch fresh data
    let fresh_data = fetch_expensive_data().await?;
    
    // Store in cache for 5 minutes (300 seconds)
    cache.set(cache_key, &fresh_data, 300).await?;
    
    Ok(Response::ok(json!({
        "source": "fresh",
        "data": fresh_data
    })))
});
```

### Step 3: Working with Storage Services

```rust
use rust_edge_gateway_sdk::prelude::*;

handler_result!(async fn upload_file(ctx: &Context, req: Request) -> Result<Response, HandlerError> {
    let storage = ctx.services.require_storage()?;
    
    // Get file data from request
    let file_data = req.body_bytes();
    let file_name = req.header("X-Filename")
        .ok_or_else(|| HandlerError::BadRequest("Missing filename".into()))?;
    
    // Upload to storage
    storage.put(&file_name, file_data).await?;
    
    // Generate download URL
    let download_url = storage.presigned_url(&file_name, 3600).await?;
    
    Ok(Response::created(json!({
        "filename": file_name,
        "download_url": download_url,
        "expires_in": 3600
    })))
});
```

## Handler Lifecycle

### Loading and Execution

1. **Compilation**: Handlers are compiled as dynamic libraries (`.so`, `.dll`, `.dylib`)
2. **Loading**: Gateway loads handlers from the handlers directory
3. **Execution**: Gateway calls the `handler_entry` function when requests arrive
4. **Unloading**: Handlers can be hot-swapped without restarting the gateway

### Hot Swapping Example

```bash
# Deploy a new version of your handler
cp target/release/libmy_handler.so /path/to/gateway/handlers/my_endpoint/

# Gateway automatically loads the new version
# Old requests complete with the old version
# New requests use the new version
```

## Best Practices

### Error Handling

```rust
handler_result!(async fn safe_handler(ctx: &Context, req: Request) -> Result<Response, HandlerError> {
    // Use ? operator for automatic error conversion
    let user_id: i64 = req.require_path_param("id")?;
    let auth_token = req.require_header("Authorization")?;
    
    // Validate input
    if user_id < 0 {
        return Err(HandlerError::ValidationError("Invalid user ID".into()));
    }
    
    // Database operations
    let db = ctx.services.require_db()?;
    let user = db.query_one(
        "SELECT * FROM users WHERE id = $1",
        &[&user_id]
    ).await?;
    
    Ok(Response::ok(user))
});
```

### Performance Tips

1. **Use async/await**: All handlers are async by default
2. **Batch operations**: Use `tokio::join!` for concurrent operations
3. **Cache aggressively**: Use cache services for expensive operations
4. **Keep handlers small**: Focus on single responsibilities
5. **Use connection pooling**: Services provide built-in connection pooling

### Logging and Debugging

```rust
handler!(async fn debug_handler(ctx: &Context, req: Request) -> Response {
    // Log with request ID for tracing
    tracing::info!(request_id = %ctx.request_id, "Processing request");
    
    // Debug service availability
    tracing::debug!("Services available: {:?}", ctx.services);
    
    Response::ok(json!({"status": "debug"}))
});
```

## Troubleshooting

### Common Issues

1. **Handler not loading**: Ensure `crate-type = ["cdylib"]` in Cargo.toml
2. **Service not available**: Check service configuration in gateway admin
3. **Permission errors**: Verify file permissions for handler libraries
4. **Missing symbols**: Ensure you're using the correct handler macros

### Debugging Tips

```bash
# Check gateway logs
journalctl -u rust-edge-gateway -f

# Test handler loading
curl -X POST http://localhost:9081/api/handlers/load/my_endpoint

# Check handler status
curl http://localhost:9081/api/handlers/status/my_endpoint
```

## Next Steps

- [Handler Lifecycle](./lifecycle.md) - Learn about compilation, loading, and hot-swapping
- [Context API](../sdk/context.md) - Full Context reference
- [Error Handling](../sdk/errors.md) - Structured error handling
- [Service Configuration](../architecture/service-actors.md) - Configure services in the gateway