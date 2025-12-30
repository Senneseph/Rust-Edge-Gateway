# Service Provider Architecture

## Overview

Rust Edge Gateway uses a **Service Provider** architecture where backend services (databases, caches, storage, etc.) are implemented as optional, long-running processes that can be dynamically loaded and managed.

## Key Concepts

### 1. Service Providers vs Core Services

**Core Services** (in `/crates/rust-edge-gateway/src/runtime/services/`):
- Essential services that are compiled into the gateway
- Provide the basic framework for service management
- Include generic interfaces like `Database`, `Cache`, `Storage`

**Service Providers** (in `/crates/rust-edge-gateway/src/services/`):
- Optional implementations that can be loaded dynamically
- Provide specific integrations (PostgreSQL, MySQL, Redis, MinIO, etc.)
- Should be abstract, long-running processes with message-passing interfaces

### 2. Message Passing Architecture

```
┌─────────────┐     ┌─────────────┐     ┌─────────────────┐
│ Endpoint    │     │ Rust Edge    │     │ Service         │
│ Handler     │────▶│ Gateway      │────▶│ Provider Actor  │
│             │     │ (Context)    │     │ (e.g., PostgreSQL│
└─────────────┘     └─────────────┘     │   connection    │
                                        │   pool)         │
                                        └─────────────────┘
```

- **Endpoint Handlers** request services via the `ctx` variable
- **Rust Edge Gateway** routes requests to the appropriate Service Provider
- **Service Provider Actors** execute commands and return results

### 3. Service Provider Lifecycle

1. **Configuration**: Service Provider is defined with connection parameters
2. **Registration**: Service Provider is registered with the gateway
3. **Activation**: Service Provider actor is spawned and connections are established
4. **Usage**: Endpoint Handlers use the Service Provider via Context
5. **Deactivation**: Service Provider actor is stopped gracefully

## Current Implementation Issues

### Problem: Services Baked into Main Binary

Currently, all service implementations in `/crates/rust-edge-gateway/src/services/` are compiled into the main application binary. This creates several issues:

1. **Bloat**: Unused services increase binary size
2. **Inflexibility**: Users cannot add custom Service Providers without recompiling
3. **Maintenance**: Core gateway updates require recompiling all Service Providers

### Solution: Optional Service Providers

Service Providers should be:

1. **Dynamically Loadable**: Loaded at runtime via API or configuration
2. **Optional**: Only loaded when needed by Endpoint Handlers
3. **Abstract**: Implement standard interfaces (Database, Cache, Storage)
4. **Long-running**: Maintain connection pools and state

## Refactoring Plan

### Phase 1: Separate Core from Optional Services

1. **Move current services** to `/examples/service-providers/` as reference implementations
2. **Create abstract interfaces** in core gateway for each service type
3. **Implement dynamic loading** mechanism for Service Providers

### Phase 2: Implement Service Provider API

1. **Service Provider Manifest**: JSON file describing capabilities
2. **Service Provider Interface**: Standardized message types
3. **Service Provider Registry**: Track available providers

### Phase 3: Update Documentation

1. **Service Provider Development Guide**: How to create custom providers
2. **Service Provider Usage Guide**: How to load and use providers
3. **Example Service Providers**: Reference implementations

## Service Provider Interface

### Database Service Provider

```rust
pub trait DatabaseServiceProvider: Send + Sync {
    /// Execute SQL query
    fn query(&self, sql: &str, params: &[&dyn ToSql]) -> Result<Vec<Row>>;
    
    /// Execute SQL query returning single row
    fn query_one(&self, sql: &str, params: &[&dyn ToSql]) -> Result<Option<Row>>;
    
    /// Execute SQL command (INSERT, UPDATE, DELETE)
    fn execute(&self, sql: &str, params: &[&dyn ToSql]) -> Result<u64>;
    
    /// Start transaction
    fn begin_transaction(&self) -> Result<Transaction>;
    
    /// Get connection info (sanitized)
    fn connection_info(&self) -> Value;
}
```

### Cache Service Provider

```rust
pub trait CacheServiceProvider: Send + Sync {
    /// Get value from cache
    fn get(&self, key: &str) -> Result<Option<Vec<u8>>>;
    
    /// Set value in cache with TTL
    fn set(&self, key: &str, value: &[u8], ttl_seconds: u32) -> Result<()>;
    
    /// Delete value from cache
    fn delete(&self, key: &str) -> Result<()>;
    
    /// Increment value
    fn increment(&self, key: &str, amount: i64) -> Result<i64>;
    
    /// Get connection info (sanitized)
    fn connection_info(&self) -> Value;
}
```

### Storage Service Provider

```rust
pub trait StorageServiceProvider: Send + Sync {
    /// Upload object
    fn put_object(&self, key: &str, data: &[u8], content_type: &str) -> Result<()>;
    
    /// Get object
    fn get_object(&self, key: &str) -> Result<Vec<u8>>;
    
    /// Delete object
    fn delete_object(&self, key: &str) -> Result<()>;
    
    /// List objects with prefix
    fn list_objects(&self, prefix: &str) -> Result<Vec<ObjectInfo>>;
    
    /// Generate presigned URL
    fn presigned_url(&self, key: &str, expires_in: u32) -> Result<String>;
    
    /// Get connection info (sanitized)
    fn connection_info(&self) -> Value;
}
```

## Service Provider Manifest

Each Service Provider should include a manifest file:

```json
{
  "name": "postgres-provider",
  "version": "1.0.0",
  "type": "database",
  "description": "PostgreSQL database service provider",
  "author": "Rust Edge Gateway Team",
  "license": "MIT",
  "capabilities": [
    "sql",
    "transactions",
    "connection_pooling"
  ],
  "config_schema": {
    "host": "string",
    "port": "number",
    "database": "string",
    "username": "string",
    "password": "string",
    "ssl_mode": "string",
    "pool_size": "number"
  }
}
```

## Implementation Strategy

### 1. Create Abstract Service Provider Interfaces

```rust
// In crates/rust-edge-gateway/src/runtime/services/mod.rs

pub trait ServiceProvider: Send + Sync + 'static {
    fn service_type(&self) -> ServiceType;
    fn test_connection(&self) -> Result<()>;
    fn connection_info(&self) -> Value;
    fn as_any(&self) -> &dyn Any;
}

pub trait DatabaseProvider: ServiceProvider {
    fn query(&self, sql: &str, params: &[&dyn ToSql]) -> Result<Vec<Row>>;
    fn query_one(&self, sql: &str, params: &[&dyn ToSql]) -> Result<Option<Row>>;
    fn execute(&self, sql: &str, params: &[&dyn ToSql]) -> Result<u64>;
}
```

### 2. Implement Dynamic Loading

```rust
pub struct ServiceProviderRegistry {
    providers: HashMap<String, Arc<dyn ServiceProvider>>,
}

impl ServiceProviderRegistry {
    pub fn register(&mut self, name: String, provider: Arc<dyn ServiceProvider>) {
        self.providers.insert(name, provider);
    }
    
    pub fn get_database(&self, name: &str) -> Result<&dyn DatabaseProvider> {
        self.providers.get(name)
            .and_then(|p| p.as_any().downcast_ref::<dyn DatabaseProvider>())
            .ok_or(ServiceError::NotConfigured(name))
    }
}
```

### 3. Update Context API

```rust
pub struct Context {
    // ... other fields
    service_registry: Arc<ServiceProviderRegistry>,
}

impl Context {
    pub async fn database(&self, name: &str) -> Result<&dyn DatabaseProvider> {
        self.service_registry.get_database(name)
    }
    
    pub async fn cache(&self, name: &str) -> Result<&dyn CacheProvider> {
        self.service_registry.get_cache(name)
    }
}
```

## Migration Plan

### Step 1: Create Service Provider Examples

Move existing service implementations to `/examples/service-providers/`:

```bash
examples/service-providers/
├── postgres/
│   ├── Cargo.toml
│   ├── src/
│   │   └── lib.rs
│   └── manifest.json
├── mysql/
│   ├── Cargo.toml
│   ├── src/
│   │   └── lib.rs
│   └── manifest.json
├── redis/
│   ├── Cargo.toml
│   ├── src/
│   │   └── lib.rs
│   └── manifest.json
└── minio/
    ├── Cargo.toml
    ├── src/
    │   └── lib.rs
    └── manifest.json
```

### Step 2: Update Core Gateway

1. Remove specific service implementations from core
2. Add abstract Service Provider interfaces
3. Implement dynamic loading mechanism
4. Update API to support Service Provider management

### Step 3: Update Documentation

1. Add Service Provider development guide
2. Update getting started guide
3. Add examples for loading Service Providers
4. Update API documentation

## Benefits of Service Provider Architecture

1. **Modularity**: Add/remove Service Providers without recompiling core
2. **Extensibility**: Users can create custom Service Providers
3. **Maintainability**: Core gateway is smaller and more focused
4. **Flexibility**: Different deployments can use different Service Providers
5. **Isolation**: Service Provider failures don't crash the gateway

## Next Steps

1. ✅ Create example Service Provider configurations
2. ⏳ Refactor core services to use Service Provider architecture
3. ⏳ Implement dynamic loading mechanism
4. ⏳ Update documentation and examples
5. ⏳ Test with petstore example