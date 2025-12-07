# Rust-Powered Serverless Edge Platform

## Overview

A lightweight, high-performance serverless framework that mirrors the simplicity of AWS Lambda while eliminating cold starts and maximizing performance. Each function is a persistent Rust service with long-lived connections, hot workers, and API‑Gateway-style routing driven by simple declarative configuration.

## Core Objectives

* **Minimal Memory Footprint** using native Rust binaries.
* **Instant Response** through persistent, warm worker processes.
* **Long-Lived Connections** for Postgres, Redis, Memcache, etc.
* **Declarative API Gateway** style routing.
* **Easy Deployment Model:** Only provide a request handler.
* **Uniform, Minimal Runtime Environment** for edge nodes.

## Architecture

### 1. Gateway Layer

* Reverse proxy routes HTTP requests to worker processes.
* Reads routing configuration from manifest.
* Supports multiple domains and REST-style paths.

### 2. Controller / Supervisor

* Launches worker binaries.
* Maintains persistent worker lifecycles.
* Manages IPC channels to workers.
* Restarts workers on crash.
* Handles hot reloads.

### 3. Worker Processes (Rust Binaries)

* Built as small Rust executables.
* Maintain persistent state and connections.
* Use a simple request/response IPC protocol.
* Expose handler functions for incoming events.

### 4. IPC Protocol

* Length-prefixed binary frames.
* Supports JSON or MessagePack payloads.
* Enables high-speed communication between gateway and workers.

## Declarative Configuration Model

```yaml
service: edge-platform

functions:
  auth_login:
    domains: ["auth.example.com"]
    routes:
      - path: "/login"
        method: POST
    binary: "bin/auth_login"
    resources:
      postgres: "postgres://..."
      redis: "redis://..."

  items_api:
    domains: ["api.example.com"]
    routes:
      - path: "/items"
        method: GET
      - path: "/items/{id}"
        method: GET
    binary: "bin/items"
    resources:
      postgres: "postgres://..."
```

## Handler Model

Each handler is a Rust program exposing an entrypoint for request handling.

Features:

* Automatic JSON parsing.
* Unified Request/Response types.
* Access to pooled connections.
* Long-lived state.

## Example Handler Skeleton

```rust
fn main() {
    let mut state = AppState::new();

    loop {
        let req = ipc::read_request();
        let res = handle_request(&mut state, req);
        ipc::send_response(res);
    }
}

fn handle_request(state: &mut AppState, req: Request) -> Response {
    match (req.method.as_str(), req.path.as_str()) {
        ("GET", "/items") => get_items(state, req),
        _ => Response::not_found(),
    }
}
```

## Persistent State and Connection Pools

* Workers create DB/Redis pools at startup.
* Pools persist across all requests.
* Eliminates connection overhead.
* Enables caching, memoization, and in-memory state.

## Benefits

* **Sub-megabyte workers** with no WASM runtime overhead.
* **No cold starts** due to long-lived workers.
* **Low latency** through Unix socket IPC.
* **Edge-ready** with minimal resource usage.
* **Developer simplicity** similar to providing only a Lambda handler.

## Future Enhancements

* Hot module swapping.
* Stateful actor-like workers.
* Built-in metrics, tracing, logging.
* Code generation from OpenAPI.

## Summary

This system merges the simplicity of serverless with the performance of native Rust. By keeping workers hot, maintaining persistent connections, and using declarative domain + route configuration, the platform offers a powerful edge-native alternative to AWS Lambda with significantly lower latency and resource usage.
