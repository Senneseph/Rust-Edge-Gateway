# Your First Handler

This guide explains the structure of a handler and how to work with the Context, Request, and Response.

## Handler Structure

Every handler follows the same pattern:

```rust
use rust_edge_gateway_sdk::prelude::*;

handler!(async fn handle(ctx: &Context, req: Request) -> Response {
    // Your logic here
    Response::ok(json!({"status": "success"}))
});
```

### The Prelude

The `prelude` module imports everything you typically need:

```rust
use rust_edge_gateway_sdk::prelude::*;

// This imports:
// - Context for service access
// - Request, Response types
// - serde::{Deserialize, Serialize}
// - serde_json::{json, Value as JsonValue}
// - HandlerError for error handling
// - The #[handler] attribute macro
```

### The Handler Function

Your handler function receives a `Context` and `Request`, and returns a `Response`:

```rust
handler!(async fn handle(ctx: &Context, req: Request) -> Response {
    // Access request data
    let method = &req.method;  // "GET", "POST", etc.
    let path = &req.path;      // "/users/123"

    // Access services via ctx.services
    // let db = ctx.services.require_db()?;

    // Return a response
    Response::ok(json!({"received": path}))
});
```

### The Handler Attribute

The `#[handler]` attribute macro generates the entry point for the dynamic library:

```rust
handler!(async fn handle(ctx: &Context, req: Request) -> Response {
    // ...
});

// This generates:
// #[no_mangle]
// pub extern "C" fn handler_entry(ctx: &Context, req: Request) -> Pin<Box<dyn Future<Output = Response> + Send>> {
//     Box::pin(handle(ctx, req))
// }
```

## Working with Requests

### Accessing the Body

For POST/PUT requests, parse the JSON body:

```rust
#[derive(Deserialize)]
struct CreateUser {
    name: String,
    email: String,
}

#[handler]
pub async fn handle(ctx: &Context, req: Request) -> Response {
    // Parse JSON body
    let user: CreateUser = match req.json() {
        Ok(u) => u,
        Err(e) => return Response::bad_request(format!("Invalid JSON: {}", e)),
    };

    Response::created(json!({
        "id": "new-user-id",
        "name": user.name,
        "email": user.email,
    }))
}
```

### Path Parameters

Extract dynamic path segments (e.g., `/users/{id}`):

```rust
#[handler]
pub async fn handle(ctx: &Context, req: Request) -> Response {
    let user_id = req.path_param("id")
        .ok_or_else(|| "Missing user ID")?;

    Response::ok(json!({"user_id": user_id}))
}
```

### Query Parameters

Access query string values (e.g., `?page=1&limit=10`):

```rust
#[handler]
pub async fn handle(ctx: &Context, req: Request) -> Response {
    let page = req.query_param("page")
        .map(|s| s.parse::<u32>().unwrap_or(1))
        .unwrap_or(1);

    let limit = req.query_param("limit")
        .map(|s| s.parse::<u32>().unwrap_or(10))
        .unwrap_or(10);

    Response::ok(json!({
        "page": page,
        "limit": limit,
    }))
}
```

### Headers

Access HTTP headers (case-insensitive):

```rust
#[handler]
pub async fn handle(ctx: &Context, req: Request) -> Response {
    let auth = req.header("Authorization");
    let content_type = req.header("Content-Type");

    if auth.is_none() {
        return Response::json(401, json!({"error": "Unauthorized"}));
    }

    Response::ok(json!({"authenticated": true}))
}
```

## Working with Responses

### JSON Responses

The most common response type:

```rust
// 200 OK with JSON
Response::ok(json!({"status": "success"}))

// 201 Created
Response::created(json!({"id": "123"}))

// Custom status with JSON
Response::json(418, json!({"error": "I'm a teapot"}))
```

### Error Responses

Built-in error response helpers:

```rust
Response::bad_request("Invalid input")      // 400
Response::not_found()                        // 404
Response::internal_error("Something broke")  // 500
```

### Custom Headers

Add headers to any response:

```rust
Response::ok(json!({"data": "value"}))
    .with_header("X-Custom-Header", "custom-value")
    .with_header("Cache-Control", "max-age=3600")
```

### Text Responses

For non-JSON responses:

```rust
Response::text(200, "Hello, World!")
Response::text(200, "<html><body>Hello</body></html>")
    .with_header("Content-Type", "text/html")
```

## Using the Context

The `Context` provides access to Service Actors:

```rust
handler_result!(async fn handle(ctx: &Context, req: Request) -> Result<Response, HandlerError> {
    // Access database service
    let db = ctx.services.require_db()?;
    let users = db.query("SELECT * FROM users", &[]).await?;

    // Access cache service
    let cache = ctx.services.require_cache()?;
    cache.set("key", "value", Some(300)).await?;

    // Access storage service
    let storage = ctx.services.require_storage()?;
    storage.put("file.txt", data).await?;

    Response::ok(json!({"users": users}))
});
```

## Next Steps

- [Handler Lifecycle](./lifecycle.md) - Compilation, loading, and hot-swapping
- [Context API](../sdk/context.md) - Service access via Context
- [Error Handling](../sdk/errors.md) - Structured error handling
- [Examples](../examples/hello-world.md) - More code examples

