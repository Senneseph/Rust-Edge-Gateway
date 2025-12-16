# Hello World

The simplest possible handler.

## Code

```rust
use rust_edge_gateway_sdk::prelude::*;

#[handler]
pub async fn handle(_ctx: &Context, _req: Request) -> Response {
    Response::ok(json!({
        "message": "Hello, World!"
    }))
}
```

## Endpoint Configuration

| Setting | Value |
|---------|-------|
| Path | `/hello` |
| Method | `GET` |
| Domain | `*` |

## Test

```bash
curl http://localhost:9080/hello
```

## Response

```json
{
  "message": "Hello, World!"
}
```

## Variations

### With Request Info

```rust
#[handler]
pub async fn handle(_ctx: &Context, req: Request) -> Response {
    Response::ok(json!({
        "message": "Hello, World!",
        "method": req.method,
        "path": req.path,
        "request_id": req.request_id,
    }))
}
```

### Plain Text

```rust
#[handler]
pub async fn handle(_ctx: &Context, _req: Request) -> Response {
    Response::text(200, "Hello, World!")
}
```

### HTML

```rust
#[handler]
pub async fn handle(_ctx: &Context, _req: Request) -> Response {
    Response::new(200)
        .with_header("Content-Type", "text/html")
        .with_body("<h1>Hello, World!</h1>")
}
```

