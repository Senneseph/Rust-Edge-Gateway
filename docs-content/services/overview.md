# Services Overview

Edge Hive provides injectable services for common infrastructure needs. Services are long-lived and maintain connection pools across requests.

## Available Services

| Service | Description |
|---------|-------------|
| `DbPool` | PostgreSQL database connection pool |
| `RedisPool` | Redis cache connection pool |
| `EnvVars` | Environment variables |

## Requesting Services

Declare services as parameters in your handler signature:

```rust
use edge_hive_sdk::prelude::*;

pub fn handle(req: Request, db: DbPool) -> Response {
    // Use db here
    Response::ok("OK")
}
```

Multiple services:

```rust
pub fn handle(req: Request, db: DbPool, redis: RedisPool) -> Response {
    // Use both
    Response::ok("OK")
}
```

## Database (DbPool)

### Query

```rust
pub fn handle(req: Request, db: DbPool) -> Response {
    let result = db.query(
        "SELECT id, name FROM users WHERE active = $1",
        &["true"]
    )?;
    
    Response::ok(json!({"users": result.rows}))
}
```

### Execute (INSERT/UPDATE/DELETE)

```rust
pub fn handle(req: Request, db: DbPool) -> Response {
    let rows_affected = db.execute(
        "UPDATE users SET last_login = NOW() WHERE id = $1",
        &["123"]
    )?;
    
    Response::ok(json!({"updated": rows_affected}))
}
```

## Redis (RedisPool)

### Get/Set

```rust
pub fn handle(req: Request, redis: RedisPool) -> Response {
    // Set with expiration
    redis.setex("session:123", "user_data", 3600)?;
    
    // Get
    let data = redis.get("session:123")?;
    
    Response::ok(json!({"cached": data}))
}
```

## Environment Variables

```rust
pub fn handle(req: Request, env: EnvVars) -> Response {
    let api_key = env.get("EXTERNAL_API_KEY")
        .ok_or_else(|| HandlerError::Internal("Missing API key".into()))?;
    
    Response::ok(json!({"has_key": true}))
}
```

## Configuring Services

Services are configured per-endpoint in the admin UI or via API:

```json
{
    "name": "user-api",
    "domain": "api.example.com",
    "path": "/users",
    "services": {
        "postgres": "postgres://user:pass@host/db",
        "redis": "redis://host:6379"
    }
}
```

Connection strings are stored securely and injected at runtime.

