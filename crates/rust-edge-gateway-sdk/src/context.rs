//! Handler Context
//!
//! The Context provides handlers with access to service actors.
//! This is passed to handlers at runtime and provides the interface
//! for communicating with MinIO, databases, caches, etc.

use std::sync::Arc;
use crate::services::{MinioClient, SqliteClient as SqliteService};

/// Handler context containing service clients
/// 
/// This is the main interface handlers use to access services.
/// Each service client uses message-passing to communicate with
/// the corresponding service actor in the gateway.
#[derive(Clone)]
pub struct Context {
    /// MinIO/S3 object storage client
    pub minio: Option<Arc<dyn MinioClient>>,
    
    /// SQLite database client
    pub sqlite: Option<Arc<dyn SqliteService>>,
    
    /// Request-scoped metadata
    pub request_id: String,
}

impl Context {
    /// Create a new empty context
    pub fn new(request_id: String) -> Self {
        Self {
            minio: None,
            sqlite: None,
            request_id,
        }
    }
    
    /// Get the MinIO client, panics if not configured
    pub fn minio(&self) -> &dyn MinioClient {
        self.minio.as_ref()
            .expect("MinIO service not configured")
            .as_ref()
    }
    
    /// Get the MinIO client if available
    pub fn try_minio(&self) -> Option<&dyn MinioClient> {
        self.minio.as_ref().map(|m| m.as_ref())
    }
    
    /// Get the SQLite client, panics if not configured
    pub fn sqlite(&self) -> &dyn SqliteService {
        self.sqlite.as_ref()
            .expect("SQLite service not configured")
            .as_ref()
    }
    
    /// Get the SQLite client if available
    pub fn try_sqlite(&self) -> Option<&dyn SqliteService> {
        self.sqlite.as_ref().map(|s| s.as_ref())
    }
}

impl std::fmt::Debug for Context {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Context")
            .field("minio", &self.minio.is_some())
            .field("sqlite", &self.sqlite.is_some())
            .field("request_id", &self.request_id)
            .finish()
    }
}

