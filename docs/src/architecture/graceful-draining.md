# Graceful Draining

Graceful draining enables zero-downtime deployments by allowing old handlers to complete in-flight requests while new handlers receive new requests.

## The Problem

Without graceful draining, updating a handler can cause:
- **Dropped requests** - In-flight requests are terminated
- **Connection resets** - Clients see connection errors
- **Data corruption** - Partial operations may leave inconsistent state

## The Solution

Graceful draining solves this by:
1. **Loading the new handler** before removing the old one
2. **Routing new requests** to the new handler immediately
3. **Allowing old requests** to complete on the old handler
4. **Unloading the old handler** only when fully drained

## Timeline

```
Time ──────────────────────────────────────────────────────────▶

Old Handler:  ████████████░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░
              (handling)  (draining)  (unloaded)

New Handler:  ░░░░░░░░░░░░████████████████████████████████████████
                          (handling new requests)

                    ▲
                    │ Swap point
```

## How It Works

### 1. Request Tracking

Each handler tracks active requests using atomic counters:

```rust
pub struct LoadedHandler {
    active_requests: AtomicU64,
    draining: AtomicBool,
}
```

### 2. Request Guards

When a request starts, a guard is acquired:

```rust
let guard = handler.acquire_request()?;
// Request is processed...
// Guard is dropped when request completes
```

The guard:
- Increments `active_requests` on creation
- Decrements `active_requests` on drop
- Returns `None` if handler is draining

### 3. Graceful Swap

When swapping handlers:

```rust
let result = registry.swap_graceful(
    "my-endpoint",
    new_library_path,
    Duration::from_secs(30)  // drain timeout
).await?;
```

This:
1. Loads the new handler
2. Atomically swaps the active handler
3. Marks the old handler as draining
4. Spawns a background task to monitor draining
5. Returns immediately (non-blocking)

### 4. Drain Monitoring

A background task monitors the old handler:

```rust
while !old_handler.is_drained() {
    if elapsed > drain_timeout {
        // Force unload after timeout
        break;
    }
    tokio::time::sleep(Duration::from_millis(100)).await;
}
// Old handler is now safe to unload
```

## API

### Swap with Draining

```rust
let result = registry.swap_graceful(
    endpoint_id,
    new_path,
    drain_timeout
).await?;

// Result contains:
// - swapped: bool - Whether swap succeeded
// - old_requests_pending: u64 - Requests still in flight
// - draining: bool - Whether old handler is draining
```

### Check Draining Status

```rust
// Is handler accepting new requests?
let accepting = !handler.is_draining();

// Is handler fully drained?
let drained = handler.is_drained();

// How many requests are in flight?
let active = handler.active_request_count();
```

### Get Statistics

```rust
let stats = registry.stats().await;

println!("Active handlers: {}", stats.loaded_count);
println!("Draining handlers: {}", stats.draining_count);
println!("Active requests: {}", stats.active_requests);
println!("Draining requests: {}", stats.draining_requests);
```

## Drain Timeout

The drain timeout determines how long to wait for requests to complete:

| Timeout | Use Case |
|---------|----------|
| 5s | Fast APIs with quick responses |
| 30s | Standard web applications |
| 60s | Long-running operations |
| 300s | File uploads, batch processing |

If the timeout expires, the old handler is forcefully unloaded. Any remaining requests will fail.

## Best Practices

### 1. Set Appropriate Timeouts

Match your drain timeout to your longest expected request:

```rust
// For a file upload endpoint
registry.swap_graceful(
    "upload-endpoint",
    new_path,
    Duration::from_secs(300)  // 5 minutes for large uploads
).await?;
```

### 2. Monitor Draining

Log draining status for observability:

```rust
if result.draining {
    tracing::info!(
        endpoint = endpoint_id,
        pending = result.old_requests_pending,
        "Handler draining"
    );
}
```

### 3. Handle Drain Rejection

When a handler is draining, new requests are rejected:

```rust
match handler.acquire_request() {
    Some(guard) => {
        // Process request
    }
    None => {
        // Handler is draining, return 503
        return Response::service_unavailable("Handler updating, retry shortly");
    }
}
```

### 4. Cleanup Drained Handlers

Periodically clean up fully drained handlers:

```rust
// In a background task
loop {
    registry.cleanup_drained().await;
    tokio::time::sleep(Duration::from_secs(60)).await;
}
```

