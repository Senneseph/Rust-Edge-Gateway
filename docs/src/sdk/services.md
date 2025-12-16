# Services

Rust Edge Gateway connects your handlers to backend services via Service Actors. Services are accessed through the Context API using an actor-based message-passing architecture.

## Overview

Services are:
1. **Configured** in the Admin UI or via API
2. **Started as actors** when the gateway launches
3. **Accessed via Context** in your handler code
4. **Thread-safe** through message-passing

## Available Service Types

| Service | Description | Use Cases |
|---------|-------------|-----------|
| **PostgreSQL** | Advanced relational database | Complex queries, transactions |
| **MySQL** | Popular relational database | Web applications, compatibility |
| **SQLite** | Embedded SQL database | Local data, caching, simple apps |
| **Redis** | In-memory data store | Caching, sessions, pub/sub |
| **MinIO/S3** | Object storage | File uploads, media storage |
| **FTP/SFTP** | File transfer protocols | File uploads, vendor integrations |
| **Email** | SMTP email sending | Notifications, alerts, reports |

## Configuring Services

### Via Admin UI

1. Go to **Services** in the admin panel
2. Click **Create Service**
3. Select service type and configure connection
4. Test the connection
5. Save the service

### Via API

```bash
curl -X POST http://localhost:9081/api/services \
  -H "Content-Type: application/json" \
  -d '{
    "name": "main-db",
    "service_type": "postgres",
    "config": {
      "host": "db.example.com",
      "port": 5432,
      "database": "myapp",
      "username": "app_user",
      "password": "secret",
      "pool_size": 10
    }
  }'
```

## Using Services in Handlers

Services are accessed through the Context:

### Database Example

```rust
use rust_edge_gateway_sdk::prelude::*;

#[handler]
pub async fn handle(ctx: &Context, req: Request) -> Result<Response, HandlerError> {
    let db = ctx.database("main-db").await?;

    // Query with parameters
    let users = db.query(
        "SELECT id, name FROM users WHERE active = $1",
        &[&true]
    ).await?;

    Ok(Response::ok(json!({"users": users})))
}
```

### Cache Example

```rust
use rust_edge_gateway_sdk::prelude::*;

#[handler]
pub async fn handle(ctx: &Context, req: Request) -> Result<Response, HandlerError> {
    let cache = ctx.cache("redis").await?;

    // Try cache first
    if let Some(cached) = cache.get("user:123").await? {
        return Ok(Response::ok(json!({"source": "cache", "data": cached})));
    }

    // Cache miss - fetch from database
    let db = ctx.database("main-db").await?;
    let user = db.query_one("SELECT * FROM users WHERE id = $1", &[&123]).await?;

    // Store in cache (TTL in seconds)
    cache.set("user:123", &user, 300).await?;

    Ok(Response::ok(json!({"source": "db", "data": user})))
}
```

### Storage Example

```rust
use rust_edge_gateway_sdk::prelude::*;

#[handler]
pub async fn handle(ctx: &Context, req: Request) -> Result<Response, HandlerError> {
    let storage = ctx.storage("s3").await?;

    // Upload file
    let data = req.body_bytes();
    storage.put("uploads/file.txt", data).await?;

    // Get presigned URL for client download
    let url = storage.presigned_url("uploads/file.txt", 3600).await?;

    Ok(Response::ok(json!({"download_url": url})))
}
```

## Actor-Based Architecture

Services use the actor pattern for thread-safety:

```
┌──────────┐     ┌─────────────┐     ┌──────────────┐
│ Handler  │────▶│   Channel   │────▶│ Service Actor│
│          │     │  (command)  │     │              │
│          │◀────│  (response) │◀────│  (owns pool) │
└──────────┘     └─────────────┘     └──────────────┘
```

Benefits:
- **Thread-safe** - No shared mutable state
- **Isolated** - Actor failures don't crash handlers
- **Efficient** - Connection pools are reused
- **Backpressure** - Channel buffers prevent overload

## Service Names

Services are identified by name in your handler code:

```rust
// These names come from your service configuration
let main_db = ctx.database("main-db").await?;
let read_replica = ctx.database("read-replica").await?;
let session_cache = ctx.cache("sessions").await?;
let file_storage = ctx.storage("uploads").await?;
```

This allows the same handler code to use different service instances in different environments.

## Next Steps

- [Context API](./context.md) - Full Context reference
- [Database Service Details](./services/database.md)
- [Cache (Redis) Details](./services/redis.md)
- [Storage Details](./services/storage.md)
- [Architecture: Service Actors](../architecture/service-actors.md)
