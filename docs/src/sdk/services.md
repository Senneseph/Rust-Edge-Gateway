# Service Providers

Rust Edge Gateway connects your handlers to backend services via **Service Providers**. Service Providers are optional, long-running processes that implement standardized interfaces for databases, caches, storage, and other backend services.

## Overview

Service Providers are:
1. **Optional** - Only loaded when needed by your endpoints
2. **Dynamic** - Can be loaded/unloaded at runtime via API
3. **Abstract** - Implement standard interfaces (Database, Cache, Storage)
4. **Long-running** - Maintain connection pools and state
5. **Message-passing** - Communicate via async message passing for thread safety

## Available Service Provider Types

Service Providers implement standardized interfaces. The following types are supported:

| Service Provider Type | Interface | Description | Use Cases |
|----------------------|-----------|-------------|-----------|
| **Database** | `DatabaseProvider` | Relational databases | Data storage, complex queries |
| **Cache** | `CacheProvider` | Key-value stores | Caching, sessions, temporary data |
| **Storage** | `StorageProvider` | Object storage | File uploads, media, large objects |
| **Email** | `EmailProvider` | Email sending | Notifications, alerts, reports |

### Database Service Providers

| Provider | Description | Interface |
|----------|-------------|----------|
| **PostgreSQL** | Advanced relational database | `DatabaseProvider` |
| **MySQL** | Popular relational database | `DatabaseProvider` |
| **SQLite** | Embedded SQL database | `DatabaseProvider` |

### Cache Service Providers

| Provider | Description | Interface |
|----------|-------------|----------|
| **Redis** | In-memory data store | `CacheProvider` |
| **Memcached** | Distributed memory cache | `CacheProvider` |

### Storage Service Providers

| Provider | Description | Interface |
|----------|-------------|----------|
| **MinIO** | S3-compatible storage | `StorageProvider` |
| **FTP/SFTP** | File transfer protocols | `StorageProvider` |

### Email Service Providers

| Provider | Description | Interface |
|----------|-------------|----------|
| **SMTP** | Email sending | `EmailProvider` |

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

handler_result!(async fn handle(ctx: &Context, req: Request) -> Result<Response, HandlerError> {
    let db = ctx.services.require_db()?;

    // Query with parameters
    let users = db.query(
        "SELECT id, name FROM users WHERE active = $1",
        &[&true]
    ).await?;

    Ok(Response::ok(json!({"users": users})))
});
```

### Cache Example

```rust
use rust_edge_gateway_sdk::prelude::*;

handler_result!(async fn handle(ctx: &Context, req: Request) -> Result<Response, HandlerError> {
    let cache = ctx.services.require_cache()?;

    // Try cache first
    if let Some(cached) = cache.get("user:123").await? {
        return Ok(Response::ok(json!({"source": "cache", "data": cached})));
    }

    // Cache miss - fetch from database
    let db = ctx.services.require_db()?;
    let user = db.query_one("SELECT * FROM users WHERE id = $1", &[&123]).await?;

    // Store in cache (TTL in seconds)
    cache.set("user:123", &user, 300).await?;

    Ok(Response::ok(json!({"source": "db", "data": user})))
});
```

### Storage Example

```rust
use rust_edge_gateway_sdk::prelude::*;

handler_result!(async fn handle(ctx: &Context, req: Request) -> Result<Response, HandlerError> {
    let storage = ctx.services.require_storage()?;

    // Upload file
    let data = req.body_bytes();
    storage.put("uploads/file.txt", data).await?;

    // Get presigned URL for client download
    let url = storage.presigned_url("uploads/file.txt", 3600).await?;

    Ok(Response::ok(json!({"download_url": url})))
});
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
