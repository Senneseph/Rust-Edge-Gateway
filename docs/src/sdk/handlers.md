# Handler Attribute

The SDK provides the `#[handler]` attribute macro for creating handler entry points.

## Quick Reference

| Pattern | Handler Signature | Use Case |
|---------|-------------------|----------|
| Basic | `async fn(&Context, Request) -> Response` | Standard handlers |
| With Result | `async fn(&Context, Request) -> Result<Response, HandlerError>` | Error handling with `?` |

## The Handler Attribute

The `#[handler]` attribute generates the dynamic library entry point:

```rust
use rust_edge_gateway_sdk::prelude::*;

#[handler]
pub async fn handle(ctx: &Context, req: Request) -> Response {
    Response::ok(json!({"path": req.path, "method": req.method}))
}
```

This generates a `handler_entry` symbol that the gateway loads and calls directly.

## Handler Signature

All handlers receive:
- `ctx: &Context` - Access to Service Actors (database, cache, storage)
- `req: Request` - The incoming HTTP request

And return:
- `Response` - The HTTP response to send

### Basic Handler

```rust
use rust_edge_gateway_sdk::prelude::*;

#[handler]
pub async fn handle(ctx: &Context, req: Request) -> Response {
    Response::ok(json!({
        "message": "Hello!",
        "path": req.path
    }))
}
```

### Handler with Error Handling

For handlers that use the `?` operator, return `Result<Response, HandlerError>`:

```rust
use rust_edge_gateway_sdk::prelude::*;

#[derive(Deserialize)]
struct CreateItem {
    name: String,
    price: f64,
}

#[handler]
pub async fn handle(ctx: &Context, req: Request) -> Result<Response, HandlerError> {
    // These all use ? operator - errors become HTTP responses
    let auth = req.require_header("Authorization")?;
    let item: CreateItem = req.json()?;

    if item.price < 0.0 {
        return Err(HandlerError::ValidationError("Price cannot be negative".into()));
    }

    Ok(Response::created(json!({"name": item.name})))
}
```

## Using Services via Context

The `Context` provides access to Service Actors:

```rust
use rust_edge_gateway_sdk::prelude::*;

#[handler]
pub async fn handle(ctx: &Context, req: Request) -> Result<Response, HandlerError> {
    // Database operations
    let db = ctx.database("main-db").await?;
    let users = db.query("SELECT * FROM users WHERE active = $1", &[&true]).await?;

    // Cache operations
    let cache = ctx.cache("redis").await?;
    if let Some(cached) = cache.get("users:all").await? {
        return Ok(Response::ok(cached));
    }

    // Storage operations
    let storage = ctx.storage("s3").await?;
    let file = storage.get("config.json").await?;

    Ok(Response::ok(json!({"users": users})))
}
```

## Async by Default

All handlers are async - the gateway runs them on a Tokio runtime:

```rust
#[handler]
pub async fn handle(ctx: &Context, req: Request) -> Response {
    // You can use .await directly
    let data = fetch_from_api().await;

    // Concurrent operations
    let (users, products) = tokio::join!(
        fetch_users(),
        fetch_products()
    );

    Response::ok(json!({"users": users, "products": products}))
}
```

## Example: Complete CRUD Handler

```rust
use rust_edge_gateway_sdk::prelude::*;

#[handler]
pub async fn handle(ctx: &Context, req: Request) -> Result<Response, HandlerError> {
    match (req.method.as_str(), req.path.as_str()) {
        ("GET", "/items") => list_items(ctx).await,
        ("POST", "/items") => create_item(ctx, &req).await,
        ("GET", _) if req.path.starts_with("/items/") => get_item(ctx, &req).await,
        ("DELETE", _) if req.path.starts_with("/items/") => delete_item(ctx, &req).await,
        _ => Err(HandlerError::MethodNotAllowed("Use GET, POST, or DELETE".into())),
    }
}

async fn list_items(ctx: &Context) -> Result<Response, HandlerError> {
    let db = ctx.database("main-db").await?;
    let items = db.query("SELECT * FROM items", &[]).await?;
    Ok(Response::ok(json!({"items": items})))
}

async fn create_item(ctx: &Context, req: &Request) -> Result<Response, HandlerError> {
    let item: NewItem = req.json()?;
    let db = ctx.database("main-db").await?;
    let id = db.execute("INSERT INTO items (name) VALUES ($1) RETURNING id", &[&item.name]).await?;
    Ok(Response::created(json!({"id": id})))
}

async fn get_item(ctx: &Context, req: &Request) -> Result<Response, HandlerError> {
    let id = req.path.strip_prefix("/items/").unwrap_or("");
    let db = ctx.database("main-db").await?;
    let item = db.query_one("SELECT * FROM items WHERE id = $1", &[&id]).await?;
    Ok(Response::ok(item))
}

async fn delete_item(ctx: &Context, req: &Request) -> Result<Response, HandlerError> {
    let id = req.path.strip_prefix("/items/").unwrap_or("");
    let db = ctx.database("main-db").await?;
    db.execute("DELETE FROM items WHERE id = $1", &[&id]).await?;
    Ok(Response::no_content())
}
```
