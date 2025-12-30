# Service Providers API

Service Providers represent optional, long-running backend service integrations (databases, caches, storage) that handlers can use. Service Providers are dynamically loaded and managed via this API.

## List Service Providers

List all configured Service Providers.

```bash
GET /api/services
```

**Response:**

```json
{
  "ok": true,
  "data": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "name": "Main Database",
      "service_type": "postgres",
      "config": {
        "host": "db.example.com",
        "port": 5432,
        "database": "myapp"
      },
      "enabled": true,
      "created_at": "2024-01-15T10:30:00Z",
      "updated_at": "2024-01-15T10:30:00Z"
    }
  ]
}
```

## Create Service Provider

Create a new Service Provider with the specified configuration.

```bash
POST /api/services
Content-Type: application/json

{
  "name": "Main Database",
  "service_type": "postgres",
  "config": {
    "host": "db.example.com",
    "port": 5432,
    "database": "myapp",
    "username": "app_user",
    "password": "secret"
  },
  "enabled": true
}
```

**Note**: Service Providers are created in a stopped state. You must activate them using the activation endpoint before they can be used by handlers.

**Request Body:**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | string | Yes | Display name |
| `service_type` | string | Yes | Type of service (see below) |
| `config` | object | Yes | Service-specific configuration |
| `enabled` | bool | No | Whether service is active (default: true) |

**Service Types:**

| Type | Description |
|------|-------------|
| `sqlite` | SQLite embedded database |
| `postgres` | PostgreSQL database |
| `mysql` | MySQL database |
| `redis` | Redis cache/store |
| `mongodb` | MongoDB document database |
| `minio` | MinIO/S3 object storage |
| `memcached` | Memcached cache |
| `ftp` | FTP/FTPS/SFTP file transfer |
| `email` | SMTP email sending |

**Response:**

```json
{
  "ok": true,
  "data": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "name": "Main Database",
    "service_type": "postgres",
    "config": { ... },
    "enabled": true,
    "created_at": "2024-01-15T10:30:00Z",
    "updated_at": "2024-01-15T10:30:00Z"
  }
}
```

## Get Service

```bash
GET /api/services/{id}
```

## Update Service

```bash
PUT /api/services/{id}
Content-Type: application/json

{
  "name": "Updated Name",
  "config": {
    "host": "new-db.example.com"
  },
  "enabled": false
}
```

## Delete Service

```bash
DELETE /api/services/{id}
```

## Activate Service Provider

Start the Service Provider actor. This spawns an async task that establishes and manages connections to the backend service. The Service Provider becomes available for use by handlers after activation.

```bash
POST /api/services/{id}/activate
```

**Note**: Only activated Service Providers can be used by endpoint handlers. If a handler tries to use a non-activated Service Provider, it will receive a `ServiceNotAvailable` error.

**Response:**

```json
{
  "success": true,
  "data": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "name": "my-storage",
    "service_type": "minio",
    "active": true,
    "message": "MinIO service actor started successfully"
  }
}
```

## Deactivate Service

Stop the service actor. In-flight operations complete before shutdown.

```bash
POST /api/services/{id}/deactivate
```

**Response:**

```json
{
  "success": true,
  "data": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "name": "my-storage",
    "service_type": "minio",
    "active": false,
    "message": "Service deactivated"
  }
}
```

## Test Service Connection

Test if the service is reachable and properly configured.

```bash
POST /api/services/{id}/test
```

**Response (Success):**

```json
{
  "ok": true,
  "data": {
    "connected": true,
    "latency_ms": 5,
    "message": "Connection successful"
  }
}
```

**Response (Failure):**

```json
{
  "ok": true,
  "data": {
    "connected": false,
    "error": "Connection refused"
  }
}
```

## MinIO File Operations

When a MinIO service is activated, the following endpoints become available for file operations:

### List Objects

```bash
GET /api/minio/objects
GET /api/minio/objects?prefix=uploads/
```

**Response:**

```json
{
  "bucket": "my-bucket",
  "prefix": "",
  "objects": [
    {
      "key": "uploads/file.txt",
      "size": 1234,
      "last_modified": "2025-12-17T00:29:55.205Z"
    }
  ]
}
```

### Upload Object

Upload a file using multipart form data.

```bash
POST /api/minio/objects
Content-Type: multipart/form-data

file: (binary data)
key: uploads/myfile.txt
```

**Response:**

```json
{
  "key": "uploads/myfile.txt",
  "bucket": "my-bucket",
  "size": 1234,
  "message": "Upload successful"
}
```

### Download Object

```bash
GET /api/minio/objects/{key}
GET /api/minio/objects/uploads/myfile.txt
```

Returns the file content with appropriate Content-Type header based on file extension.

### Delete Object

```bash
DELETE /api/minio/objects/{key}
DELETE /api/minio/objects/uploads/myfile.txt
```

**Response:**

```json
{
  "key": "uploads/myfile.txt",
  "bucket": "my-bucket",
  "deleted": true
}
```

## Using Service Providers in Handlers

Once a Service Provider is created and activated, it can be used in endpoint handlers via the Context API:

### Database Service Provider Example

```rust
use rust_edge_gateway_sdk::prelude::*;

handler_result!(async fn handle(ctx: &Context, req: Request) -> Result<Response, HandlerError> {
    // Get the database service by name
    let db = ctx.services.database("main-db").await?;
    
    // Execute a query
    let users = db.query(
        "SELECT id, name FROM users WHERE active = $1",
        &[&true]
    ).await?;
    
    Ok(Response::ok(json!({"users": users})))
});
```

### Cache Service Provider Example

```rust
use rust_edge_gateway_sdk::prelude::*;

handler_result!(async fn handle(ctx: &Context, req: Request) -> Result<Response, HandlerError> {
    // Get the cache service by name
    let cache = ctx.services.cache("session-cache").await?;
    
    // Try to get from cache
    if let Some(cached_data) = cache.get("user:123").await? {
        return Ok(Response::ok(json!({"source": "cache", "data": cached_data})));
    }
    
    // Cache miss - fetch from database
    let db = ctx.services.database("main-db").await?;
    let user = db.query_one("SELECT * FROM users WHERE id = $1", &[&123]).await?;
    
    // Store in cache with 300 second TTL
    cache.set("user:123", &user, 300).await?;
    
    Ok(Response::ok(json!({"source": "db", "data": user})))
});
```

### Storage Service Provider Example

```rust
use rust_edge_gateway_sdk::prelude::*;

handler_result!(async fn handle(ctx: &Context, req: Request) -> Result<Response, HandlerError> {
    // Get the storage service by name
    let storage = ctx.services.storage("file-storage").await?;
    
    // Upload file
    let file_data = req.body_bytes();
    storage.put("uploads/file.txt", file_data, "application/octet-stream").await?;
    
    // Generate presigned URL
    let download_url = storage.presigned_url("uploads/file.txt", 3600).await?;
    
    Ok(Response::ok(json!({"download_url": download_url})))
});
```

## Service Provider Lifecycle

Service Providers follow this lifecycle:

1. **Created**: Service Provider is configured and stored in database
2. **Activated**: Service Provider actor is spawned and connections are established
3. **Available**: Service Provider is ready for use by handlers
4. **Deactivated**: Service Provider actor is stopped gracefully
5. **Deleted**: Service Provider is removed from database

## Error Handling

### Service Provider Not Found

If a handler tries to use a Service Provider that doesn't exist:

```json
{
  "success": false,
  "error": "ServiceProviderNotFound: main-db"
}
```

### Service Provider Not Activated

If a handler tries to use a Service Provider that hasn't been activated:

```json
{
  "success": false,
  "error": "ServiceProviderNotActivated: main-db"
}
```

### Service Provider Connection Failed

If a Service Provider fails to connect to the backend service:

```json
{
  "success": false,
  "error": "ServiceProviderConnectionFailed: main-db - Connection refused"
}
```

## Best Practices

1. **Name Services Clearly**: Use descriptive names like `main-db`, `session-cache`, `file-storage`
2. **Activate Before Use**: Always activate Service Providers before creating endpoints that depend on them
3. **Handle Errors Gracefully**: Implement proper error handling for Service Provider failures
4. **Monitor Connections**: Use the test endpoint to verify Service Provider health
5. **Clean Up Unused Services**: Deactivate and delete Service Providers that are no longer needed

## Service Configuration Examples

### PostgreSQL

```json
{
  "service_type": "postgres",
  "config": {
    "host": "localhost",
    "port": 5432,
    "database": "myapp",
    "username": "app_user",
    "password": "secret",
    "ssl_mode": "prefer",
    "pool_size": 10
  }
}
```

### MySQL

```json
{
  "service_type": "mysql",
  "config": {
    "host": "localhost",
    "port": 3306,
    "database": "myapp",
    "username": "app_user",
    "password": "secret",
    "use_ssl": false,
    "pool_size": 10
  }
}
```

### Redis

```json
{
  "service_type": "redis",
  "config": {
    "host": "localhost",
    "port": 6379,
    "password": null,
    "database": 0,
    "use_tls": false,
    "pool_size": 10
  }
}
```

### SQLite

```json
{
  "service_type": "sqlite",
  "config": {
    "path": "/data/app.db",
    "create_if_missing": true
  }
}
```

### MinIO

```json
{
  "service_type": "minio",
  "config": {
    "endpoint": "minio.example.com:9000",
    "access_key": "minioadmin",
    "secret_key": "minioadmin",
    "use_ssl": true,
    "bucket": "uploads"
  }
}
```

### FTP/SFTP

```json
{
  "service_type": "ftp",
  "config": {
    "host": "sftp.example.com",
    "port": 22,
    "username": "user",
    "password": "secret",
    "protocol": "sftp",
    "base_path": "/uploads",
    "timeout_seconds": 30
  }
}
```

### Email (SMTP)

```json
{
  "service_type": "email",
  "config": {
    "host": "smtp.example.com",
    "port": 587,
    "username": "sender@example.com",
    "password": "app-password",
    "encryption": "starttls",
    "from_address": "noreply@example.com",
    "from_name": "My App"
  }
}
```
