//! MinIO Service Actor
//!
//! Provides isolated, async message-passing interface to MinIO/S3 storage.
//! All operations are enqueued and executed cleanly with proper error handling.

use tokio::sync::{mpsc, oneshot};
use anyhow::Result;
use bytes::Bytes;
use serde::{Deserialize, Serialize};

use super::super::actor::{ActorMessage, ActorError};

/// MinIO configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinioConfig {
    pub endpoint: String,
    pub access_key: String,
    #[serde(skip_serializing)]
    pub secret_key: String,
    pub bucket: String,
    #[serde(default)]
    pub use_ssl: bool,
    #[serde(default = "default_region")]
    pub region: String,
}

fn default_region() -> String {
    "us-east-1".to_string()
}

/// Object metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectInfo {
    pub key: String,
    pub size: u64,
    pub last_modified: String,
    pub etag: Option<String>,
    pub content_type: Option<String>,
}

/// MinIO operation messages
#[derive(Debug)]
pub enum MinioOp {
    GetObject {
        bucket: String,
        key: String,
        reply: oneshot::Sender<Result<Bytes, ActorError>>,
    },
    PutObject {
        bucket: String,
        key: String,
        data: Bytes,
        content_type: Option<String>,
        reply: oneshot::Sender<Result<(), ActorError>>,
    },
    DeleteObject {
        bucket: String,
        key: String,
        reply: oneshot::Sender<Result<(), ActorError>>,
    },
    ListObjects {
        bucket: String,
        prefix: String,
        reply: oneshot::Sender<Result<Vec<ObjectInfo>, ActorError>>,
    },
    HealthCheck {
        reply: oneshot::Sender<Result<bool, ActorError>>,
    },
    Shutdown,
}

impl ActorMessage for MinioOp {}

/// Handle to communicate with MinIO Service Actor
#[derive(Clone)]
pub struct MinioHandle {
    sender: mpsc::Sender<MinioOp>,
    pub default_bucket: String,
}

impl MinioHandle {
    pub fn new(sender: mpsc::Sender<MinioOp>, default_bucket: String) -> Self {
        Self { sender, default_bucket }
    }

    pub async fn get_object(&self, bucket: &str, key: &str) -> Result<Bytes, ActorError> {
        let (tx, rx) = oneshot::channel();
        self.sender.send(MinioOp::GetObject {
            bucket: bucket.to_string(),
            key: key.to_string(),
            reply: tx,
        }).await?;
        rx.await?
    }

    pub async fn put_object(&self, bucket: &str, key: &str, data: Bytes, content_type: Option<&str>) -> Result<(), ActorError> {
        let (tx, rx) = oneshot::channel();
        self.sender.send(MinioOp::PutObject {
            bucket: bucket.to_string(),
            key: key.to_string(),
            data,
            content_type: content_type.map(String::from),
            reply: tx,
        }).await?;
        rx.await?
    }

    pub async fn delete_object(&self, bucket: &str, key: &str) -> Result<(), ActorError> {
        let (tx, rx) = oneshot::channel();
        self.sender.send(MinioOp::DeleteObject {
            bucket: bucket.to_string(),
            key: key.to_string(),
            reply: tx,
        }).await?;
        rx.await?
    }

    pub async fn list_objects(&self, bucket: &str, prefix: &str) -> Result<Vec<ObjectInfo>, ActorError> {
        let (tx, rx) = oneshot::channel();
        self.sender.send(MinioOp::ListObjects {
            bucket: bucket.to_string(),
            prefix: prefix.to_string(),
            reply: tx,
        }).await?;
        rx.await?
    }

    pub async fn health_check(&self) -> Result<bool, ActorError> {
        let (tx, rx) = oneshot::channel();
        self.sender.send(MinioOp::HealthCheck { reply: tx }).await?;
        rx.await?
    }

    pub fn is_alive(&self) -> bool {
        !self.sender.is_closed()
    }
}

impl std::fmt::Debug for MinioHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MinioHandle")
            .field("default_bucket", &self.default_bucket)
            .field("is_alive", &self.is_alive())
            .finish()
    }
}

/// MinIO Service Actor - runs as a tokio task processing operations
pub struct MinioServiceActor {
    config: MinioConfig,
    bucket: s3::Bucket,
}

impl MinioServiceActor {
    /// Create a new MinIO service actor
    pub async fn new(config: MinioConfig) -> Result<Self> {
        let region = s3::Region::Custom {
            region: config.region.clone(),
            endpoint: if config.use_ssl {
                format!("https://{}", config.endpoint)
            } else {
                format!("http://{}", config.endpoint)
            },
        };

        let credentials = s3::creds::Credentials::new(
            Some(&config.access_key),
            Some(&config.secret_key),
            None,
            None,
            None,
        ).map_err(|e| anyhow::anyhow!("Credentials error: {}", e))?;

        let bucket = s3::Bucket::new(
            &config.bucket,
            region,
            credentials,
        ).map_err(|e| anyhow::anyhow!("Bucket creation error: {}", e))?
         .with_path_style();

        Ok(Self { config, bucket: *bucket })
    }

    /// Spawn the actor and return a handle
    pub async fn spawn(config: MinioConfig) -> Result<MinioHandle> {
        let actor = Self::new(config.clone()).await?;
        let default_bucket = config.bucket.clone();
        let (tx, rx) = mpsc::channel(256);

        tokio::spawn(async move {
            actor.run(rx).await;
        });

        Ok(MinioHandle::new(tx, default_bucket))
    }

    /// Main actor loop - processes operations one at a time
    async fn run(self, mut rx: mpsc::Receiver<MinioOp>) {
        tracing::info!("MinIO Service Actor started for bucket: {}", self.config.bucket);

        while let Some(op) = rx.recv().await {
            match op {
                MinioOp::GetObject { bucket, key, reply } => {
                    let result = self.get_object_impl(&bucket, &key).await;
                    let _ = reply.send(result);
                }
                MinioOp::PutObject { bucket, key, data, content_type, reply } => {
                    let result = self.put_object_impl(&bucket, &key, data, content_type).await;
                    let _ = reply.send(result);
                }
                MinioOp::DeleteObject { bucket, key, reply } => {
                    let result = self.delete_object_impl(&bucket, &key).await;
                    let _ = reply.send(result);
                }
                MinioOp::ListObjects { bucket, prefix, reply } => {
                    let result = self.list_objects_impl(&bucket, &prefix).await;
                    let _ = reply.send(result);
                }
                MinioOp::HealthCheck { reply } => {
                    let result = self.health_check_impl().await;
                    let _ = reply.send(result);
                }
                MinioOp::Shutdown => {
                    tracing::info!("MinIO Service Actor shutting down");
                    break;
                }
            }
        }
    }

    async fn get_object_impl(&self, _bucket: &str, key: &str) -> Result<Bytes, ActorError> {
        // Note: rust-s3 bucket is pre-configured, bucket param is for future multi-bucket support
        let response = self.bucket.get_object(key).await
            .map_err(|e| ActorError::OperationFailed(format!("GetObject failed: {}", e)))?;
        Ok(Bytes::from(response.to_vec()))
    }

    async fn put_object_impl(&self, _bucket: &str, key: &str, data: Bytes, content_type: Option<String>) -> Result<(), ActorError> {
        let ct = content_type.as_deref().unwrap_or("application/octet-stream");
        self.bucket.put_object_with_content_type(key, &data, ct).await
            .map_err(|e| ActorError::OperationFailed(format!("PutObject failed: {}", e)))?;
        Ok(())
    }

    async fn delete_object_impl(&self, _bucket: &str, key: &str) -> Result<(), ActorError> {
        self.bucket.delete_object(key).await
            .map_err(|e| ActorError::OperationFailed(format!("DeleteObject failed: {}", e)))?;
        Ok(())
    }

    async fn list_objects_impl(&self, _bucket: &str, prefix: &str) -> Result<Vec<ObjectInfo>, ActorError> {
        let list = self.bucket.list(prefix.to_string(), None).await
            .map_err(|e| ActorError::OperationFailed(format!("ListObjects failed: {}", e)))?;

        let objects = list.iter()
            .flat_map(|result| result.contents.iter())
            .map(|obj| ObjectInfo {
                key: obj.key.clone(),
                size: obj.size,
                last_modified: obj.last_modified.clone(),
                etag: obj.e_tag.clone(),
                content_type: None,
            })
            .collect();
        Ok(objects)
    }

    async fn health_check_impl(&self) -> Result<bool, ActorError> {
        // Try to list with empty prefix to check connectivity
        self.bucket.list(String::new(), Some("/".to_string())).await
            .map_err(|e| ActorError::OperationFailed(format!("Health check failed: {}", e)))?;
        Ok(true)
    }
}

