# Handler Registry

The Handler Registry manages loaded handler libraries and provides hot-swapping with graceful draining.

## Overview

```rust
pub struct HandlerRegistry {
    /// Map of endpoint ID to loaded handler
    handlers: RwLock<HashMap<String, Arc<LoadedHandler>>>,
    
    /// Handlers that are draining (previous versions)
    draining_handlers: RwLock<Vec<Arc<LoadedHandler>>>,
    
    /// Directory where handler libraries are stored
    handlers_dir: PathBuf,
}
```

## Loading Handlers

When a handler is deployed, the registry:

1. **Locates the library** in the handlers directory
2. **Loads it with `libloading`** (cross-platform dynamic loading)
3. **Finds the `handler_entry` symbol** (function pointer)
4. **Stores in the handlers map** by endpoint ID

```rust
// Load a handler
registry.load("my-endpoint").await?;

// Load from specific path
registry.load_from("my-endpoint", Path::new("/path/to/lib.so")).await?;
```

## Executing Handlers

The registry provides execution methods with request tracking:

```rust
// Execute a handler
let response = registry.execute("my-endpoint", &ctx, request).await?;

// Execute with timeout
let response = registry.execute_with_timeout(
    "my-endpoint",
    &ctx,
    request,
    Duration::from_secs(30)
).await?;
```

Request tracking ensures graceful draining works correctly.

## Hot Swapping

### Immediate Swap

For quick updates where in-flight requests can be dropped:

```rust
registry.swap("my-endpoint", Path::new("/path/to/new-lib.so")).await?;
```

The old handler is dropped immediately.

### Graceful Swap

For zero-downtime updates:

```rust
let result = registry.swap_graceful(
    "my-endpoint",
    Path::new("/path/to/new-lib.so"),
    Duration::from_secs(30)  // drain timeout
).await?;

println!("Swapped: {}", result.swapped);
println!("Pending requests: {}", result.old_requests_pending);
println!("Draining: {}", result.draining);
```

The graceful swap:
1. Loads the new handler
2. Atomically swaps the active handler
3. Marks the old handler as draining
4. Waits for in-flight requests to complete
5. Unloads the old handler when drained

## Request Tracking

Each `LoadedHandler` tracks active requests:

```rust
pub struct LoadedHandler {
    // ... library and entry point ...
    
    /// Active request count
    active_requests: AtomicU64,
    
    /// Whether this handler is draining
    draining: AtomicBool,
}
```

When executing a handler:

```rust
// Acquire request guard (increments counter)
let guard = handler.acquire_request()?;

// Execute handler
let response = handler.execute(&ctx, request).await;

// Guard is dropped (decrements counter)
drop(guard);
```

If the handler is draining, `acquire_request()` returns `None`.

## Draining States

A handler can be in one of these states:

| State | `draining` | `active_requests` | Description |
|-------|------------|-------------------|-------------|
| Active | `false` | Any | Accepting new requests |
| Draining | `true` | > 0 | Finishing in-flight requests |
| Drained | `true` | 0 | Ready to unload |

## Monitoring

Get statistics about loaded handlers:

```rust
let stats = registry.stats().await;

println!("Loaded handlers: {}", stats.loaded_count);
println!("Draining handlers: {}", stats.draining_count);
println!("Active requests: {}", stats.active_requests);
println!("Draining requests: {}", stats.draining_requests);
```

## Cleanup

Periodically clean up fully drained handlers:

```rust
let removed = registry.cleanup_drained().await;
println!("Cleaned up {} drained handlers", removed);
```

This is typically called by a background task.

## Library Naming

Libraries are named by platform:

| Platform | Library Name |
|----------|--------------|
| Linux | `libhandler_{id}.so` |
| Windows | `handler_{id}.dll` |
| macOS | `libhandler_{id}.dylib` |

The registry handles this automatically based on the target platform.

