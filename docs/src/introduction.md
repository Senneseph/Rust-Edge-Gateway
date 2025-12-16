# Rust Edge Gateway

**Rust Edge Gateway** is a high-performance API gateway that lets you write request handlers in Rust. Your handlers are compiled to native dynamic libraries and loaded directly into the gateway process, providing:

- 🚀 **Native Performance** - Handlers compile to optimized native code (.so/.dll)
- ⚡ **Zero-Copy Execution** - Direct function calls, no serialization overhead
- 🔄 **Hot Reload** - Swap handlers without restarting the gateway
- 🎭 **Actor-Based Services** - Database, cache, and storage via message-passing
- 🔀 **Graceful Draining** - Zero-downtime deployments with request draining
- 🛠️ **Simple SDK** - Easy-to-use Context, Request, and Response API

## How It Works

```
┌─────────────┐     ┌──────────────────┐     ┌─────────────────┐
│   Client    │────▶│  Edge Gateway    │────▶│  Your Handler   │
│  (Browser,  │     │  (Routes &       │     │  (Dynamic       │
│   API, etc) │◀────│   Manages)       │◀────│   Library .so)  │
└─────────────┘     └──────────────────┘     └─────────────────┘
                            │
                            ▼
                    ┌───────────────┐
                    │ Service Actors│
                    │  (DB, Cache,  │
                    │   Storage)    │
                    └───────────────┘
```

1. **Gateway receives request** - The gateway matches the incoming request to an endpoint
2. **Handler is invoked** - The compiled handler library is called directly via function pointer
3. **Handler processes** - Your code runs with access to the Context API and Service Actors
4. **Response returned** - The handler returns a Response directly to the gateway

## Getting Started

The fastest way to get started is to:

1. Access the Admin UI at `/admin/`
2. Create a new endpoint
3. Write your handler code
4. Compile and deploy

See the [Quick Start](./getting-started/quick-start.md) guide for detailed instructions.

## SDK Overview

Your handler code uses the `rust-edge-gateway-sdk` crate:

```rust
use rust_edge_gateway_sdk::prelude::*;

#[handler]
pub async fn handle(ctx: &Context, req: Request) -> Response {
    Response::ok(json!({
        "message": "Hello, World!",
        "path": req.path,
        "method": req.method,
    }))
}
```

The SDK provides:

- **[Context](./sdk/context.md)** - Access to Service Actors (database, cache, storage)
- **[Request](./sdk/request.md)** - Access HTTP method, path, headers, body, query params
- **[Response](./sdk/response.md)** - Build HTTP responses with JSON, text, or custom content
- **[HandlerError](./sdk/errors.md)** - Structured error handling with HTTP status codes
- **[Services](./sdk/services.md)** - Database, cache, and storage service actors

## Architecture

Rust Edge Gateway uses a dynamic library loading model with actor-based services:

- **Main Gateway** - Axum-based HTTP server handling routing
- **Handler Registry** - Manages loaded handler libraries with hot-swap support
- **Dynamic Libraries** - Your compiled handlers as `.so` (Linux), `.dll` (Windows), or `.dylib` (macOS)
- **Service Actors** - Message-passing based services for database, cache, and storage
- **Graceful Draining** - Old handlers complete in-flight requests during updates

This architecture provides:

- **Performance** - Direct function calls with zero serialization overhead
- **Hot Swapping** - Replace handlers without gateway restart
- **Zero Downtime** - Graceful draining ensures no dropped requests during updates
- **Scalability** - Async handlers with Tokio runtime for high concurrency

