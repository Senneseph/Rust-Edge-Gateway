//! Worker process management
//!
//! Manages the lifecycle of handler worker processes.

use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::process::{Child, Command, Stdio};
use std::time::Duration;

use crate::api::Endpoint;
use crate::config::AppConfig;

/// Represents a running worker process
pub struct Worker {
    pub endpoint_id: String,
    pub process: Child,
    pub binary_path: String,
}

impl Worker {
    /// Send a request to the worker and get the response
    pub fn handle_request(&mut self, request: &edge_hive_sdk::Request) -> Result<edge_hive_sdk::Response> {
        let stdin = self.process.stdin.as_mut()
            .ok_or_else(|| anyhow!("Worker stdin not available"))?;
        let stdout = self.process.stdout.as_mut()
            .ok_or_else(|| anyhow!("Worker stdout not available"))?;
        
        // Serialize request
        let payload = serde_json::to_vec(request)?;
        
        // Write length-prefixed request
        let len = payload.len() as u32;
        stdin.write_all(&len.to_be_bytes())?;
        stdin.write_all(&payload)?;
        stdin.flush()?;
        
        // Read length-prefixed response
        let mut len_buf = [0u8; 4];
        stdout.read_exact(&mut len_buf)?;
        let len = u32::from_be_bytes(len_buf) as usize;
        
        let mut response_buf = vec![0u8; len];
        stdout.read_exact(&mut response_buf)?;
        
        let response: edge_hive_sdk::Response = serde_json::from_slice(&response_buf)?;
        Ok(response)
    }
    
    /// Check if the worker is still running
    pub fn is_alive(&mut self) -> bool {
        matches!(self.process.try_wait(), Ok(None))
    }
    
    /// Kill the worker process
    pub fn kill(&mut self) {
        let _ = self.process.kill();
        let _ = self.process.wait();
    }
}

/// Manages all worker processes
pub struct WorkerManager {
    workers: HashMap<String, Worker>,
    config: AppConfig,
}

impl WorkerManager {
    pub fn new(config: &AppConfig) -> Self {
        Self {
            workers: HashMap::new(),
            config: config.clone(),
        }
    }
    
    /// Get the number of active workers
    pub fn active_count(&self) -> usize {
        self.workers.len()
    }
    
    /// Start a worker for an endpoint
    pub fn start_worker(&mut self, endpoint: &Endpoint) -> Result<()> {
        // Stop existing worker if any
        self.stop_worker(&endpoint.id);

        // Binary name matches package name: handler_{id with - replaced by _}
        let binary_name = format!("handler_{}", endpoint.id.replace('-', "_"));
        let binary_path = self.config.handlers_dir
            .join(&endpoint.id)
            .join("target")
            .join("release")
            .join(&binary_name);

        if !binary_path.exists() {
            return Err(anyhow!("Binary not found at {:?}", binary_path));
        }
        
        let process = Command::new(&binary_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        
        let worker = Worker {
            endpoint_id: endpoint.id.clone(),
            process,
            binary_path: binary_path.to_string_lossy().to_string(),
        };
        
        self.workers.insert(endpoint.id.clone(), worker);
        tracing::info!("Started worker for endpoint {}", endpoint.id);
        
        Ok(())
    }
    
    /// Stop a worker
    pub fn stop_worker(&mut self, endpoint_id: &str) {
        if let Some(mut worker) = self.workers.remove(endpoint_id) {
            worker.kill();
            tracing::info!("Stopped worker for endpoint {}", endpoint_id);
        }
    }
    
    /// Get a mutable reference to a worker
    pub fn get_worker(&mut self, endpoint_id: &str) -> Option<&mut Worker> {
        self.workers.get_mut(endpoint_id)
    }
    
    /// Handle a request for an endpoint
    pub fn handle_request(
        &mut self,
        endpoint_id: &str,
        request: &edge_hive_sdk::Request,
        _timeout: Duration,
    ) -> Result<edge_hive_sdk::Response> {
        let worker = self.workers.get_mut(endpoint_id)
            .ok_or_else(|| anyhow!("No worker for endpoint {}", endpoint_id))?;
        
        // Check if worker is still alive
        if !worker.is_alive() {
            return Err(anyhow!("Worker died unexpectedly"));
        }
        
        // TODO: Implement proper timeout handling
        // For now, just send the request
        worker.handle_request(request)
    }
}

impl Drop for WorkerManager {
    fn drop(&mut self) {
        // Kill all workers on shutdown
        for (_, mut worker) in self.workers.drain() {
            worker.kill();
        }
    }
}
