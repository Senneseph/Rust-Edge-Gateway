# Path Parameters

Extract dynamic values from URL paths.

## Basic Example

**Endpoint Path:** `/users/{id}`

```rust
use rust_edge_gateway_sdk::prelude::*;

#[handler]
pub async fn handle(_ctx: &Context, req: Request) -> Response {
    // Extract the {id} parameter
    let user_id = match req.path_param("id") {
        Some(id) => id,
        None => return Response::bad_request("Missing user ID"),
    };

    Response::ok(json!({
        "user_id": user_id,
        "message": format!("Fetching user {}", user_id),
    }))
}
```

## Test

```bash
curl http://localhost:9080/users/123
```

## Response

```json
{
  "user_id": "123",
  "message": "Fetching user 123"
}
```

## Multiple Parameters

**Endpoint Path:** `/users/{user_id}/posts/{post_id}`

```rust
#[handler]
pub async fn handle(_ctx: &Context, req: Request) -> Response {
    let user_id = req.path_param("user_id");
    let post_id = req.path_param("post_id");

    match (user_id, post_id) {
        (Some(uid), Some(pid)) => {
            Response::ok(json!({
                "user_id": uid,
                "post_id": pid,
            }))
        }
        _ => Response::bad_request("Missing parameters"),
    }
}
```

### Test

```bash
curl http://localhost:9080/users/42/posts/7
```

### Response

```json
{
  "user_id": "42",
  "post_id": "7"
}
```

## Type Conversion

Path parameters are always strings. Convert them to other types:

```rust
#[handler]
pub async fn handle(_ctx: &Context, req: Request) -> Response {
    let id_str = match req.path_param("id") {
        Some(id) => id,
        None => return Response::bad_request("Missing ID"),
    };

    // Parse to integer
    let id: i64 = match id_str.parse() {
        Ok(n) => n,
        Err(_) => return Response::bad_request("ID must be a number"),
    };

    // Parse to UUID
    let uuid_str = match req.path_param("uuid") {
        Some(u) => u,
        None => return Response::bad_request("Missing UUID"),
    };

    let uuid = match uuid::Uuid::parse_str(uuid_str) {
        Ok(u) => u,
        Err(_) => return Response::bad_request("Invalid UUID format"),
    };

    Response::ok(json!({
        "id": id,
        "uuid": uuid.to_string(),
    }))
}
```

## Optional Parameters with Defaults

```rust
#[handler]
pub async fn handle(_ctx: &Context, req: Request) -> Response {
    // Get page number, default to 1
    let page: u32 = req.path_param("page")
        .and_then(|p| p.parse().ok())
        .unwrap_or(1);

    Response::ok(json!({"page": page}))
}
```

## Route Patterns

| Pattern | Matches | Parameters |
|---------|---------|------------|
| `/users/{id}` | `/users/123` | `id: "123"` |
| `/api/{version}/items` | `/api/v2/items` | `version: "v2"` |
| `/files/{path}` | `/files/docs` | `path: "docs"` |
| `/{org}/{repo}/issues/{num}` | `/acme/proj/issues/42` | `org: "acme"`, `repo: "proj"`, `num: "42"` |

## Common Patterns

### Resource by ID

```rust
// GET /users/{id}
#[handler]
pub async fn get_user(ctx: &Context, req: Request) -> Result<Response, HandlerError> {
    let id = req.path_param("id").unwrap();
    let db = ctx.database("main-db").await?;
    let user = db.query_one("SELECT * FROM users WHERE id = $1", &[&id]).await?;
    Ok(Response::ok(user))
}
```

### Nested Resources

```rust
// GET /organizations/{org_id}/teams/{team_id}/members
#[handler]
pub async fn get_team_members(ctx: &Context, req: Request) -> Result<Response, HandlerError> {
    let org_id = req.path_param("org_id").unwrap();
    let team_id = req.path_param("team_id").unwrap();

    let db = ctx.database("main-db").await?;
    let members = db.query(
        "SELECT * FROM members WHERE org_id = $1 AND team_id = $2",
        &[&org_id, &team_id]
    ).await?;

    Ok(Response::ok(json!({
        "organization": org_id,
        "team": team_id,
        "members": members,
    })))
}
```

### Slug-based Routes

```rust
// GET /blog/{slug}
#[handler]
pub async fn get_blog_post(ctx: &Context, req: Request) -> Result<Response, HandlerError> {
    let slug = req.path_param("slug").unwrap();

    let db = ctx.database("main-db").await?;
    let post = db.query_one("SELECT * FROM posts WHERE slug = $1", &[&slug]).await?;

    Ok(Response::ok(post))
}
```
