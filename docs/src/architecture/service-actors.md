# Service Actors

Service Actors provide thread-safe access to backend services using the actor pattern.

## Actor Pattern

Each service runs as an independent actor:

```
┌──────────────────────────────────────────────────────────────┐
│                      Service Actor                            │
├──────────────────────────────────────────────────────────────┤
│  ┌─────────────┐     ┌─────────────┐     ┌─────────────┐    │
│  │   Inbox     │────▶│   Actor     │────▶│  Backend    │    │
│  │  (Channel)  │     │   Loop      │     │  (Pool)     │    │
│  └─────────────┘     └─────────────┘     └─────────────┘    │
│         ▲                   │                                │
│         │                   ▼                                │
│  ┌─────────────┐     ┌─────────────┐                        │
│  │  Handlers   │◀────│  Response   │                        │
│  │  (Callers)  │     │  Channel    │                        │
│  └─────────────┘     └─────────────┘                        │
└──────────────────────────────────────────────────────────────┘
```

## How It Works

1. **Handler sends a command** to the actor's inbox channel
2. **Actor receives the command** in its event loop
3. **Actor executes the operation** using its connection pool
4. **Actor sends the result** back via a oneshot channel
5. **Handler receives the result** and continues

## Actor Types

### Database Actor

Manages SQL database connections:

```rust
pub enum DatabaseCommand {
    Query {
        sql: String,
        params: Vec<Value>,
        reply: oneshot::Sender<Result<Vec<Row>>>,
    },
    Execute {
        sql: String,
        params: Vec<Value>,
        reply: oneshot::Sender<Result<u64>>,
    },
}
```

### Cache Actor

Manages Redis/Memcached connections:

```rust
pub enum CacheCommand {
    Get {
        key: String,
        reply: oneshot::Sender<Result<Option<String>>>,
    },
    Set {
        key: String,
        value: String,
        ttl: Option<u64>,
        reply: oneshot::Sender<Result<()>>,
    },
    Delete {
        key: String,
        reply: oneshot::Sender<Result<bool>>,
    },
}
```

### Storage Actor

Manages object storage (S3/MinIO):

```rust
pub enum StorageCommand {
    Get {
        key: String,
        reply: oneshot::Sender<Result<Vec<u8>>>,
    },
    Put {
        key: String,
        data: Vec<u8>,
        reply: oneshot::Sender<Result<()>>,
    },
    Delete {
        key: String,
        reply: oneshot::Sender<Result<()>>,
    },
    List {
        prefix: String,
        reply: oneshot::Sender<Result<Vec<String>>>,
    },
}
```

## Actor Handle

Handlers interact with actors through handles:

```rust
pub struct ActorHandle<C> {
    sender: mpsc::Sender<C>,
}

impl<C> ActorHandle<C> {
    pub async fn send(&self, command: C) -> Result<()> {
        self.sender.send(command).await?;
        Ok(())
    }
}
```

## Benefits

### Thread Safety

Actors own their resources exclusively:
- No shared mutable state
- No locks needed
- No data races possible

### Isolation

Actor failures are contained:
- A crashed actor doesn't crash handlers
- Actors can be restarted independently
- Errors are returned as `Result` values

### Backpressure

Channel buffers provide natural backpressure:
- If an actor is overloaded, senders wait
- Prevents resource exhaustion
- Configurable buffer sizes

### Connection Pooling

Actors manage connection pools:
- Connections are reused across requests
- Pool size is configurable
- Automatic reconnection on failure

## Configuration

Actors are configured via the Admin UI or API:

```json
{
  "name": "main-db",
  "type": "postgres",
  "config": {
    "host": "db.example.com",
    "port": 5432,
    "database": "myapp",
    "username": "app",
    "password": "secret",
    "pool_size": 10
  }
}
```

## Lifecycle

1. **Gateway starts** - Service actors are spawned
2. **Handlers execute** - Send commands to actors
3. **Actors process** - Execute operations, return results
4. **Gateway stops** - Actors are gracefully shut down

Actors run for the lifetime of the gateway and are shared across all handlers.

