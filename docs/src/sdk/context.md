# Context API

The `Context` provides access to Service Actors from within your handler. It's the bridge between your handler code and backend services like databases, caches, and storage.

## Overview

Every handler receives a `Context` reference:

```rust
#[handler]
pub async fn handle(ctx: &Context, req: Request) -> Response {
    // Use ctx to access services
    let db = ctx.database("main-db").await?;
    // ...
}
```

## Available Services

| Method | Returns | Description |
|--------|---------|-------------|
| `ctx.database(name)` | `DatabaseHandle` | SQL database connection |
| `ctx.cache(name)` | `CacheHandle` | Key-value cache (Redis) |
| `ctx.storage(name)` | `StorageHandle` | Object storage (S3/MinIO) |

## Database Access

```rust
#[handler]
pub async fn handle(ctx: &Context, req: Request) -> Result<Response, HandlerError> {
    let db = ctx.database("main-db").await?;
    
    // Query with parameters
    let users = db.query(
        "SELECT id, name, email FROM users WHERE active = $1",
        &[&true]
    ).await?;
    
    // Execute (INSERT, UPDATE, DELETE)
    let affected = db.execute(
        "UPDATE users SET last_login = NOW() WHERE id = $1",
        &[&user_id]
    ).await?;
    
    // Query single row
    let user = db.query_one(
        "SELECT * FROM users WHERE id = $1",
        &[&user_id]
    ).await?;
    
    Ok(Response::ok(json!({"users": users})))
}
```

## Cache Access

```rust
#[handler]
pub async fn handle(ctx: &Context, req: Request) -> Result<Response, HandlerError> {
    let cache = ctx.cache("redis").await?;
    
    // Try cache first
    if let Some(cached) = cache.get("users:all").await? {
        return Ok(Response::ok(cached));
    }
    
    // Cache miss - fetch from database
    let db = ctx.database("main-db").await?;
    let users = db.query("SELECT * FROM users", &[]).await?;
    
    // Store in cache with TTL (seconds)
    cache.set("users:all", &users, 300).await?;
    
    Ok(Response::ok(json!({"users": users})))
}
```

### Cache Operations

| Method | Description |
|--------|-------------|
| `get(key)` | Get value by key |
| `set(key, value, ttl)` | Set value with TTL in seconds |
| `delete(key)` | Delete a key |
| `exists(key)` | Check if key exists |
| `incr(key)` | Increment numeric value |
| `decr(key)` | Decrement numeric value |

## Storage Access

```rust
#[handler]
pub async fn handle(ctx: &Context, req: Request) -> Result<Response, HandlerError> {
    let storage = ctx.storage("s3").await?;
    
    // Upload file
    let data = req.body_bytes();
    storage.put("uploads/file.txt", data).await?;
    
    // Download file
    let content = storage.get("config/settings.json").await?;
    
    // List files
    let files = storage.list("uploads/").await?;
    
    // Delete file
    storage.delete("uploads/old-file.txt").await?;
    
    // Get signed URL (for direct client access)
    let url = storage.presigned_url("uploads/file.txt", 3600).await?;
    
    Ok(Response::ok(json!({"url": url})))
}
```

## Service Configuration

Services are configured in the Admin UI or via the Management API. Each service has a unique name that you use to access it:

```rust
// These names come from your service configuration
let main_db = ctx.database("main-db").await?;
let read_replica = ctx.database("read-replica").await?;
let session_cache = ctx.cache("sessions").await?;
let file_storage = ctx.storage("uploads").await?;
```

## Error Handling

Service operations return `Result` types that can be used with `?`:

```rust
#[handler]
pub async fn handle(ctx: &Context, req: Request) -> Result<Response, HandlerError> {
    // Service errors are automatically converted to HandlerError
    let db = ctx.database("main-db").await?;
    let users = db.query("SELECT * FROM users", &[]).await?;
    
    Ok(Response::ok(json!({"users": users})))
}
```

Common error types:
- `ServiceNotFound` - The named service doesn't exist
- `ConnectionError` - Failed to connect to the service
- `QueryError` - Database query failed
- `StorageError` - Storage operation failed

## Actor-Based Architecture

Under the hood, services use an actor-based architecture:

1. **Service Actors** run as background tasks
2. **Handlers send messages** to actors via channels
3. **Actors process requests** and send responses back
4. **Connection pooling** is handled automatically

This provides:
- **Isolation** - Service failures don't crash handlers
- **Concurrency** - Multiple handlers can share services safely
- **Efficiency** - Connection pools are reused across requests

