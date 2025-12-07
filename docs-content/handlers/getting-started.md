# Handler Guide

Handlers are simple Rust files that process HTTP requests and return responses.

## Handler Structure

Every handler must define a `handle` function:

```rust
use edge_hive_sdk::prelude::*;

pub fn handle(req: Request) -> Response {
    // Your logic here
    Response::ok("Hello, World!")
}
```

## Request Object

The `Request` struct contains:

| Field | Type | Description |
|-------|------|-------------|
| `method` | `String` | HTTP method (GET, POST, etc.) |
| `path` | `String` | Request path |
| `query` | `HashMap<String, String>` | Query parameters |
| `headers` | `HashMap<String, String>` | HTTP headers |
| `body` | `Option<String>` | Request body |
| `params` | `HashMap<String, String>` | Path parameters |
| `request_id` | `String` | Unique request ID |

### Reading JSON Body

```rust
use edge_hive_sdk::prelude::*;

#[derive(Deserialize)]
struct CreateUser {
    name: String,
    email: String,
}

pub fn handle(req: Request) -> Response {
    let user: CreateUser = match req.json() {
        Ok(u) => u,
        Err(_) => return Response::bad_request("Invalid JSON"),
    };
    
    Response::created(json!({
        "id": 1,
        "name": user.name,
        "email": user.email
    }))
}
```

## Response Object

Create responses using helper methods:

```rust
// 200 OK with JSON
Response::ok(json!({"status": "success"}))

// 201 Created
Response::created(json!({"id": 123}))

// 204 No Content
Response::no_content()

// 400 Bad Request
Response::bad_request("Missing field: name")

// 404 Not Found
Response::not_found()

// 500 Internal Server Error
Response::internal_error("Database connection failed")

// Custom status with text
Response::text(418, "I'm a teapot")

// Add headers
Response::ok(data).with_header("X-Custom", "value")
```

## Path Parameters

Access path parameters from dynamic routes like `/users/{id}`:

```rust
pub fn handle(req: Request) -> Response {
    let user_id = req.path_param("id")
        .ok_or_else(|| Response::bad_request("Missing user ID"))?;
    
    Response::ok(json!({"user_id": user_id}))
}
```

## Query Parameters

Access query parameters:

```rust
pub fn handle(req: Request) -> Response {
    let page = req.query_param("page")
        .map(|p| p.parse::<u32>().unwrap_or(1))
        .unwrap_or(1);
    
    let limit = req.query_param("limit")
        .map(|l| l.parse::<u32>().unwrap_or(10))
        .unwrap_or(10);
    
    Response::ok(json!({
        "page": page,
        "limit": limit,
        "data": []
    }))
}
```

## Error Handling

Use Rust's `Result` pattern:

```rust
use edge_hive_sdk::prelude::*;

pub fn handle(req: Request) -> Response {
    match process_request(&req) {
        Ok(data) => Response::ok(data),
        Err(e) => e.to_response(),
    }
}

fn process_request(req: &Request) -> Result<JsonValue, HandlerError> {
    let body: MyData = req.json()
        .map_err(|e| HandlerError::ValidationError(e.to_string()))?;
    
    // Process...
    Ok(json!({"status": "ok"}))
}
```

