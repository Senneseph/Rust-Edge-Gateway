//! IPC protocol for communicating with the Edge Hive gateway
//!
//! Handlers communicate with the gateway using a simple length-prefixed JSON protocol
//! over stdin/stdout.

use crate::{Request, Response, HandlerError};
use serde::de::DeserializeOwned;
use std::io::{self, Read, Write};

/// Read a request from stdin (sent by the gateway)
pub fn read_request() -> Result<Request, HandlerError> {
    let stdin = io::stdin();
    let mut handle = stdin.lock();
    
    // Read length prefix (4 bytes, big-endian)
    let mut len_buf = [0u8; 4];
    if handle.read_exact(&mut len_buf).is_err() {
        return Err(HandlerError::IpcError("Failed to read length prefix".into()));
    }
    
    let len = u32::from_be_bytes(len_buf) as usize;
    
    // Read the JSON payload
    let mut payload = vec![0u8; len];
    if handle.read_exact(&mut payload).is_err() {
        return Err(HandlerError::IpcError("Failed to read payload".into()));
    }
    
    // Parse the request
    serde_json::from_slice(&payload)
        .map_err(|e| HandlerError::IpcError(format!("Failed to parse request: {}", e)))
}

/// Send a response to stdout (received by the gateway)
pub fn send_response(response: Response) -> Result<(), HandlerError> {
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    
    // Serialize the response
    let payload = serde_json::to_vec(&response)
        .map_err(|e| HandlerError::IpcError(format!("Failed to serialize response: {}", e)))?;
    
    // Write length prefix
    let len = payload.len() as u32;
    handle.write_all(&len.to_be_bytes())
        .map_err(|e| HandlerError::IpcError(format!("Failed to write length: {}", e)))?;
    
    // Write payload
    handle.write_all(&payload)
        .map_err(|e| HandlerError::IpcError(format!("Failed to write payload: {}", e)))?;
    
    handle.flush()
        .map_err(|e| HandlerError::IpcError(format!("Failed to flush: {}", e)))?;
    
    Ok(())
}

/// Call a service through the gateway (for DB, Redis, etc.)
/// This sends a service request and waits for the response
pub fn call_service<T: DeserializeOwned>(request: serde_json::Value) -> Result<T, HandlerError> {
    let stderr = io::stderr();
    let mut handle = stderr.lock();
    
    // Serialize the service request
    let payload = serde_json::to_vec(&request)
        .map_err(|e| HandlerError::IpcError(format!("Failed to serialize service request: {}", e)))?;
    
    // Write to stderr (service channel)
    let len = payload.len() as u32;
    handle.write_all(&len.to_be_bytes())
        .map_err(|e| HandlerError::IpcError(format!("Failed to write service request length: {}", e)))?;
    handle.write_all(&payload)
        .map_err(|e| HandlerError::IpcError(format!("Failed to write service request: {}", e)))?;
    handle.flush()
        .map_err(|e| HandlerError::IpcError(format!("Failed to flush service request: {}", e)))?;
    
    // Read response from stdin (interleaved with requests)
    // In practice, the gateway will handle this properly
    // For now, this is a placeholder
    
    // TODO: Implement proper bidirectional IPC
    Err(HandlerError::IpcError("Service calls not yet implemented".into()))
}

/// Convenience macro for running a handler loop
#[macro_export]
macro_rules! handler_loop {
    ($handler:expr) => {
        fn main() {
            loop {
                match $crate::ipc::read_request() {
                    Ok(req) => {
                        let response = $handler(req);
                        if let Err(e) = $crate::ipc::send_response(response) {
                            eprintln!("Failed to send response: {}", e);
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to read request: {}", e);
                        break;
                    }
                }
            }
        }
    };
}

