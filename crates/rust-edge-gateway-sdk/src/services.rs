//! Service types for dependency injection
//!
//! These types represent the services that can be injected into handlers.
//! The actual implementations are provided by the Rust Edge Gateway runtime.
//!
//! Handlers use trait objects (dyn MinioClient, dyn SqliteClient) which
//! are implemented by the gateway's service actors.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

/// Result type for async service operations
pub type ServiceResult<T> = Result<T, ServiceError>;

/// Future type for async service operations
pub type ServiceFuture<'a, T> = Pin<Box<dyn Future<Output = ServiceResult<T>> + Send + 'a>>;

/// Service error type
#[derive(Debug, Clone)]
pub enum ServiceError {
    /// Service not available
    NotAvailable(String),
    /// Operation failed
    OperationFailed(String),
    /// Connection error
    ConnectionError(String),
    /// Timeout
    Timeout,
}

impl std::fmt::Display for ServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServiceError::NotAvailable(s) => write!(f, "Service not available: {}", s),
            ServiceError::OperationFailed(s) => write!(f, "Operation failed: {}", s),
            ServiceError::ConnectionError(s) => write!(f, "Connection error: {}", s),
            ServiceError::Timeout => write!(f, "Operation timed out"),
        }
    }
}

impl std::error::Error for ServiceError {}

/// Object metadata for MinIO/S3 objects
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectInfo {
    pub key: String,
    pub size: u64,
    pub last_modified: String,
    pub etag: Option<String>,
    pub content_type: Option<String>,
}

/// MinIO/S3 client trait
///
/// Handlers receive a reference to this trait and call methods on it.
/// The gateway provides the actual implementation that communicates
/// with the MinIO service actor via message passing.
pub trait MinioClient: Send + Sync {
    /// Get an object's contents
    fn get_object<'a>(&'a self, bucket: &'a str, key: &'a str) -> ServiceFuture<'a, Vec<u8>>;

    /// Put an object
    fn put_object<'a>(&'a self, bucket: &'a str, key: &'a str, data: Vec<u8>, content_type: Option<&'a str>) -> ServiceFuture<'a, ()>;

    /// Delete an object
    fn delete_object<'a>(&'a self, bucket: &'a str, key: &'a str) -> ServiceFuture<'a, ()>;

    /// List objects with prefix
    fn list_objects<'a>(&'a self, bucket: &'a str, prefix: &'a str) -> ServiceFuture<'a, Vec<ObjectInfo>>;

    /// Get the default bucket name
    fn default_bucket(&self) -> &str;
}

/// SQLite client trait
pub trait SqliteClient: Send + Sync {
    /// Execute a query and return results
    fn query<'a>(&'a self, sql: &'a str, params: Vec<String>) -> ServiceFuture<'a, Vec<HashMap<String, serde_json::Value>>>;

    /// Execute a statement (INSERT, UPDATE, DELETE)
    fn execute<'a>(&'a self, sql: &'a str, params: Vec<String>) -> ServiceFuture<'a, u64>;
}

/// Configuration for connecting to services
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceConfig {
    /// Service type (e.g., "postgres", "redis", "kafka")
    pub service_type: String,

    /// Connection string or configuration
    pub connection: String,

    /// Additional options
    #[serde(default)]
    pub options: HashMap<String, String>,
}

/// Database query result placeholder
/// In the actual runtime, this will be populated by the gateway
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbResult {
    pub rows: Vec<HashMap<String, serde_json::Value>>,
    pub rows_affected: u64,
}

/// Database service handle
/// Handlers receive this and use it to make database calls via IPC
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbPool {
    /// Identifier for this pool in the runtime
    pub pool_id: String,
}

impl DbPool {
    /// Execute a query (sends IPC message to gateway)
    pub fn query(&self, sql: &str, params: &[&str]) -> Result<DbResult, crate::HandlerError> {
        let request = serde_json::json!({
            "service": "db",
            "pool_id": self.pool_id,
            "action": "query",
            "sql": sql,
            "params": params,
        });
        
        crate::ipc::call_service(request)
    }
    
    /// Execute a statement (INSERT, UPDATE, DELETE)
    pub fn execute(&self, sql: &str, params: &[&str]) -> Result<u64, crate::HandlerError> {
        let result = self.query(sql, params)?;
        Ok(result.rows_affected)
    }
}

/// Redis service handle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisPool {
    pub pool_id: String,
}

impl RedisPool {
    /// Get a value
    pub fn get(&self, key: &str) -> Result<Option<String>, crate::HandlerError> {
        let request = serde_json::json!({
            "service": "redis",
            "pool_id": self.pool_id,
            "action": "get",
            "key": key,
        });
        
        let result: serde_json::Value = crate::ipc::call_service(request)?;
        Ok(result.as_str().map(String::from))
    }
    
    /// Set a value
    pub fn set(&self, key: &str, value: &str) -> Result<(), crate::HandlerError> {
        let request = serde_json::json!({
            "service": "redis",
            "pool_id": self.pool_id,
            "action": "set",
            "key": key,
            "value": value,
        });
        
        crate::ipc::call_service::<()>(request)?;
        Ok(())
    }
    
    /// Set a value with expiration
    pub fn setex(&self, key: &str, value: &str, seconds: u64) -> Result<(), crate::HandlerError> {
        let request = serde_json::json!({
            "service": "redis",
            "pool_id": self.pool_id,
            "action": "setex",
            "key": key,
            "value": value,
            "seconds": seconds,
        });
        
        crate::ipc::call_service::<()>(request)?;
        Ok(())
    }
}

/// Environment variables available to the handler
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EnvVars {
    pub vars: HashMap<String, String>,
}

impl EnvVars {
    pub fn get(&self, key: &str) -> Option<&String> {
        self.vars.get(key)
    }
}

