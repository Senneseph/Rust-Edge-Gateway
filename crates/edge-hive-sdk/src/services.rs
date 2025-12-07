//! Service types for dependency injection
//!
//! These types represent the services that can be injected into handlers.
//! The actual implementations are provided by the Edge Hive runtime.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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

