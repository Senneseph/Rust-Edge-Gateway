//! HTTP Response representation for handlers

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents an outgoing HTTP response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    /// HTTP status code
    pub status: u16,
    
    /// Response headers
    #[serde(default)]
    pub headers: HashMap<String, String>,
    
    /// Response body
    #[serde(default)]
    pub body: Option<String>,
}

impl Response {
    /// Create a new response with the given status code
    pub fn new(status: u16) -> Self {
        Self {
            status,
            headers: HashMap::new(),
            body: None,
        }
    }
    
    /// Create a 200 OK response with a body
    pub fn ok<T: Serialize>(body: T) -> Self {
        Self::json(200, body)
    }
    
    /// Create a JSON response with the given status code
    pub fn json<T: Serialize>(status: u16, body: T) -> Self {
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());
        
        Self {
            status,
            headers,
            body: serde_json::to_string(&body).ok(),
        }
    }
    
    /// Create a plain text response
    pub fn text(status: u16, body: impl Into<String>) -> Self {
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "text/plain".to_string());
        
        Self {
            status,
            headers,
            body: Some(body.into()),
        }
    }
    
    /// Create a 404 Not Found response
    pub fn not_found() -> Self {
        Self::json(404, serde_json::json!({"error": "Not Found"}))
    }
    
    /// Create a 400 Bad Request response
    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::json(400, serde_json::json!({"error": message.into()}))
    }
    
    /// Create a 500 Internal Server Error response
    pub fn internal_error(message: impl Into<String>) -> Self {
        Self::json(500, serde_json::json!({"error": message.into()}))
    }
    
    /// Create a 201 Created response
    pub fn created<T: Serialize>(body: T) -> Self {
        Self::json(201, body)
    }
    
    /// Create a 204 No Content response
    pub fn no_content() -> Self {
        Self::new(204)
    }
    
    /// Add a header to the response
    pub fn with_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }
    
    /// Set the body
    pub fn with_body(mut self, body: impl Into<String>) -> Self {
        self.body = Some(body.into());
        self
    }
}

impl Default for Response {
    fn default() -> Self {
        Self::new(200)
    }
}

