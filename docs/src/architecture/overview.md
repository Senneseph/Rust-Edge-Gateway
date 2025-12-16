# Architecture Overview

Rust Edge Gateway uses a dynamic library loading architecture with actor-based services for high performance and zero-downtime deployments.

## High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        Edge Gateway                              │
├─────────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐  │
│  │   Router    │  │   Admin     │  │    Handler Registry     │  │
│  │  (Axum)     │  │   API       │  │  (Dynamic Libraries)    │  │
│  └──────┬──────┘  └─────────────┘  └───────────┬─────────────┘  │
│         │                                       │                │
│         │         ┌─────────────────────────────┘                │
│         │         │                                              │
│         ▼         ▼                                              │
│  ┌─────────────────────────────────────────────────────────────┐│
│  │                    Service Actors                            ││
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐    ││
│  │  │ Database │  │  Cache   │  │ Storage  │  │  Email   │    ││
│  │  │  Actor   │  │  Actor   │  │  Actor   │  │  Actor   │    ││
│  │  └──────────┘  └──────────┘  └──────────┘  └──────────┘    ││
│  └─────────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────────┘
```

## Core Components

### Router (Axum)

The main HTTP server built on Axum:
- Receives incoming HTTP requests
- Matches requests to endpoints by path, method, and domain
- Dispatches to the appropriate handler
- Returns responses to clients

### Handler Registry

Manages loaded handler libraries:
- Loads dynamic libraries (`.so`, `.dll`, `.dylib`)
- Maintains a map of endpoint ID to handler function
- Supports hot-swapping with graceful draining
- Tracks active requests per handler

### Service Actors

Background tasks that manage backend connections:
- **Database Actor** - Connection pooling for SQL databases
- **Cache Actor** - Redis/Memcached connections
- **Storage Actor** - S3/MinIO object storage
- **Email Actor** - SMTP connections

Actors communicate via message-passing channels, providing isolation and thread-safety.

### Admin API

RESTful API for management:
- Create/update/delete endpoints
- Compile handler code
- Deploy/undeploy handlers
- Configure services
- View logs and metrics

## Request Flow

1. **Request arrives** at the Axum router
2. **Router matches** the request to an endpoint
3. **Handler Registry** looks up the handler by endpoint ID
4. **Request guard** is acquired (for draining support)
5. **Handler function** is called with Context and Request
6. **Handler accesses services** via Context (sends messages to actors)
7. **Response is returned** to the router
8. **Request guard** is dropped (decrements active count)
9. **Router sends** response to client

## Handler Compilation

When you compile a handler:

1. **Code is written** to `handlers/{id}/src/lib.rs`
2. **Cargo.toml** is generated with SDK dependency
3. **`cargo build --release`** compiles to dynamic library
4. **Library is stored** in `handlers/{id}/target/release/`

The generated library exports a `handler_entry` symbol that the registry loads.

## Hot Swapping

When you update a handler:

1. **New library is compiled**
2. **New library is loaded** into memory
3. **Registry atomically swaps** the handler pointer
4. **Old handler starts draining** (no new requests)
5. **Active requests complete** on old handler
6. **Old library is unloaded** when drained

This provides zero-downtime deployments.

## Service Actor Pattern

Services use the actor pattern for safety and efficiency:

```
┌──────────┐     ┌─────────────┐     ┌──────────────┐
│ Handler  │────▶│   Channel   │────▶│ Service Actor│
│          │     │  (mpsc)     │     │              │
│          │◀────│             │◀────│  (owns pool) │
└──────────┘     └─────────────┘     └──────────────┘
```

Benefits:
- **Thread-safe** - No shared mutable state
- **Isolated** - Actor failures don't crash handlers
- **Efficient** - Connection pools are reused
- **Backpressure** - Channel buffers prevent overload

## Comparison with v1

| Feature | v1 (Subprocess) | v2 (Dynamic Library) |
|---------|-----------------|----------------------|
| Execution | Child process | Direct function call |
| IPC | stdin/stdout JSON | None (in-process) |
| Latency | ~1-5ms overhead | ~0.01ms overhead |
| Memory | Separate per handler | Shared with gateway |
| Hot Swap | Restart process | Atomic pointer swap |
| Draining | Kill process | Graceful completion |

