# JSON API

Build a RESTful JSON API endpoint.

## Code

```rust
use rust_edge_gateway_sdk::prelude::*;

#[derive(Deserialize)]
struct CreateItem {
    name: String,
    description: Option<String>,
    price: f64,
}

#[derive(Serialize)]
struct Item {
    id: String,
    name: String,
    description: Option<String>,
    price: f64,
}

#[handler]
pub async fn handle(_ctx: &Context, req: Request) -> Response {
    // Parse request body
    let input: CreateItem = match req.json() {
        Ok(data) => data,
        Err(e) => return Response::bad_request(format!("Invalid JSON: {}", e)),
    };

    // Validate
    if input.name.is_empty() {
        return Response::bad_request("Name is required");
    }

    if input.price < 0.0 {
        return Response::bad_request("Price must be non-negative");
    }

    // Create item (in real app, save to database via ctx)
    let item = Item {
        id: uuid::Uuid::new_v4().to_string(),
        name: input.name,
        description: input.description,
        price: input.price,
    };

    // Return 201 Created
    Response::created(item)
}
```

## Endpoint Configuration

| Setting | Value |
|---------|-------|
| Path | `/items` |
| Method | `POST` |
| Domain | `*` |

## Test

```bash
curl -X POST http://localhost:9080/items \
  -H "Content-Type: application/json" \
  -d '{"name": "Widget", "description": "A useful widget", "price": 19.99}'
```

## Response

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "name": "Widget",
  "description": "A useful widget",
  "price": 19.99,
  "created_at": "2024-01-15T10:30:00.000Z"
}
```

## Full CRUD Example

For a complete CRUD API with database access:

### GET /items - List Items

```rust
#[handler]
pub async fn handle(ctx: &Context, _req: Request) -> Result<Response, HandlerError> {
    let db = ctx.database("main-db").await?;
    let items = db.query("SELECT id, name FROM items", &[]).await?;

    Ok(Response::ok(json!({
        "items": items,
        "count": items.len(),
    })))
}
```

### GET /items/{id} - Get Item

```rust
#[handler]
pub async fn handle(ctx: &Context, req: Request) -> Result<Response, HandlerError> {
    let id = req.path_param("id")
        .ok_or_else(|| HandlerError::ValidationError("Missing ID".into()))?;

    let db = ctx.database("main-db").await?;
    let item = db.query_one("SELECT * FROM items WHERE id = $1", &[&id]).await?;

    Ok(Response::ok(item))
}
```

### PUT /items/{id} - Update Item

```rust
#[derive(Deserialize)]
struct UpdateItem {
    name: Option<String>,
    price: Option<f64>,
}

#[handler]
pub async fn handle(ctx: &Context, req: Request) -> Result<Response, HandlerError> {
    let id = req.path_param("id")
        .ok_or_else(|| HandlerError::ValidationError("Missing ID".into()))?;

    let update: UpdateItem = req.json()?;

    let db = ctx.database("main-db").await?;
    db.execute(
        "UPDATE items SET name = COALESCE($1, name), price = COALESCE($2, price) WHERE id = $3",
        &[&update.name, &update.price, &id]
    ).await?;

    Ok(Response::ok(json!({"id": id, "updated": true})))
}
```

### DELETE /items/{id} - Delete Item

```rust
#[handler]
pub async fn handle(ctx: &Context, req: Request) -> Result<Response, HandlerError> {
    let id = req.path_param("id")
        .ok_or_else(|| HandlerError::ValidationError("Missing ID".into()))?;

    let db = ctx.database("main-db").await?;
    db.execute("DELETE FROM items WHERE id = $1", &[&id]).await?;

    Ok(Response::no_content())
}
```
