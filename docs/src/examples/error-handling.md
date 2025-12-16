# Error Handling

Robust error handling patterns for handlers.

## Basic Pattern

```rust
use rust_edge_gateway_sdk::prelude::*;

#[handler]
pub async fn handle(_ctx: &Context, req: Request) -> Result<Response, HandlerError> {
    let data = process_request(&req)?;
    Ok(Response::ok(data))
}

fn process_request(req: &Request) -> Result<JsonValue, HandlerError> {
    let input: CreateUser = req.json()
        .map_err(|e| HandlerError::ValidationError(e.to_string()))?;

    // Process and return result
    Ok(json!({"id": "123", "name": input.name}))
}

#[derive(Deserialize)]
struct CreateUser {
    name: String,
}
```

## Error Types and Status Codes

```rust
fn process(req: &Request) -> Result<JsonValue, HandlerError> {
    // 400 Bad Request - Invalid input
    if req.body.is_none() {
        return Err(HandlerError::ValidationError("Body required".into()));
    }
    
    // 401 Unauthorized - Missing/invalid auth
    if req.header("Authorization").is_none() {
        return Err(HandlerError::Unauthorized("Token required".into()));
    }
    
    // 404 Not Found - Resource doesn't exist
    let user = find_user("123");
    if user.is_none() {
        return Err(HandlerError::NotFound("User not found".into()));
    }
    
    // 503 Service Unavailable - Backend down
    if !database_available() {
        return Err(HandlerError::ServiceUnavailable("Database down".into()));
    }
    
    // 500 Internal Error - Unexpected error
    if something_broke() {
        return Err(HandlerError::Internal("Unexpected error".into()));
    }
    
    Ok(json!({"status": "ok"}))
}
```

## Input Validation

```rust
#[derive(Deserialize)]
struct RegisterUser {
    email: String,
    password: String,
    name: String,
}

fn validate_input(input: &RegisterUser) -> Result<(), HandlerError> {
    // Email validation
    if !input.email.contains('@') {
        return Err(HandlerError::ValidationError(
            "Invalid email format".into()
        ));
    }

    // Password validation
    if input.password.len() < 8 {
        return Err(HandlerError::ValidationError(
            "Password must be at least 8 characters".into()
        ));
    }

    // Name validation
    if input.name.trim().is_empty() {
        return Err(HandlerError::ValidationError(
            "Name is required".into()
        ));
    }

    Ok(())
}

#[handler]
pub async fn handle(_ctx: &Context, req: Request) -> Result<Response, HandlerError> {
    let input: RegisterUser = req.json()?;
    validate_input(&input)?;
    Ok(Response::created(json!({"email": input.email})))
}
```

## Custom Error Type

```rust
enum AppError {
    UserNotFound(String),
    EmailTaken(String),
    InvalidCredentials,
    RateLimited,
    DatabaseError(String),
}

impl From<AppError> for HandlerError {
    fn from(e: AppError) -> Self {
        match e {
            AppError::UserNotFound(id) => 
                HandlerError::NotFound(format!("User {} not found", id)),
            AppError::EmailTaken(email) => 
                HandlerError::ValidationError(format!("Email {} already registered", email)),
            AppError::InvalidCredentials => 
                HandlerError::Unauthorized("Invalid email or password".into()),
            AppError::RateLimited => 
                HandlerError::Internal("Rate limit exceeded".into()),
            AppError::DatabaseError(msg) => 
                HandlerError::DatabaseError(msg),
        }
    }
}

fn process(req: &Request) -> Result<JsonValue, AppError> {
    // Business logic with custom errors
    Err(AppError::InvalidCredentials)
}

#[handler]
pub async fn handle(_ctx: &Context, req: Request) -> Result<Response, HandlerError> {
    let data = process(&req)?;
    Ok(Response::ok(data))
}
```

## Logging Errors

Always log errors for debugging:

```rust
#[handler]
pub async fn handle(_ctx: &Context, req: Request) -> Result<Response, HandlerError> {
    match process(&req) {
        Ok(data) => Ok(Response::ok(data)),
        Err(e) => {
            // Log with request ID for tracing
            eprintln!("[{}] Error: {}", req.request_id, e);

            // Log stack trace for internal errors
            if matches!(e, HandlerError::Internal(_) | HandlerError::DatabaseError(_)) {
                eprintln!("[{}] Request path: {}", req.request_id, req.path);
                eprintln!("[{}] Request body: {:?}", req.request_id, req.body);
            }

            Err(e)
        }
    }
}
```

## Graceful Degradation

Handle service failures gracefully:

```rust
#[handler]
pub async fn handle(ctx: &Context, _req: Request) -> Result<Response, HandlerError> {
    let cache = ctx.cache("redis").await;
    let db = ctx.database("main").await?;

    // Try cache first (non-fatal if unavailable)
    if let Ok(cache) = cache {
        match cache.get("data:key").await {
            Ok(Some(data)) => {
                return Ok(Response::ok(json!({"source": "cache", "data": data})));
            }
            Ok(None) => { /* Cache miss, continue */ }
            Err(e) => {
                // Log but don't fail - Redis being down shouldn't break the app
                eprintln!("Redis error (non-fatal): {}", e);
            }
        }
    }

    // Fallback to database
    match db.query("SELECT * FROM data", &[]).await {
        Ok(result) => Ok(Response::ok(json!({"source": "db", "data": result}))),
        Err(e) => {
            eprintln!("Database error: {}", e);
            Ok(Response::json(503, json!({
                "error": "Service temporarily unavailable",
                "retry_after": 5,
            })))
        }
    }
}
```

