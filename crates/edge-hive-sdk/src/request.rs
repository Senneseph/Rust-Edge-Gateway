//! HTTP Request representation for handlers

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents an incoming HTTP request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Request {
    /// HTTP method (GET, POST, PUT, DELETE, etc.)
    pub method: String,
    
    /// Request path (e.g., "/items/123")
    pub path: String,
    
    /// Query parameters
    #[serde(default)]
    pub query: HashMap<String, String>,
    
    /// HTTP headers
    #[serde(default)]
    pub headers: HashMap<String, String>,
    
    /// Request body (raw bytes as base64 for binary, or string for text)
    #[serde(default)]
    pub body: Option<String>,
    
    /// Path parameters extracted from route (e.g., {id} -> "123")
    #[serde(default)]
    pub params: HashMap<String, String>,
    
    /// Client IP address
    #[serde(default)]
    pub client_ip: Option<String>,
    
    /// Request ID for tracing
    #[serde(default)]
    pub request_id: String,
}

impl Request {
    /// Parse the body as JSON
    pub fn json<T: for<'de> Deserialize<'de>>(&self) -> Result<T, serde_json::Error> {
        match &self.body {
            Some(body) => serde_json::from_str(body),
            None => serde_json::from_str("null"),
        }
    }
    
    /// Get a query parameter
    pub fn query_param(&self, key: &str) -> Option<&String> {
        self.query.get(key)
    }
    
    /// Get a path parameter
    pub fn path_param(&self, key: &str) -> Option<&String> {
        self.params.get(key)
    }
    
    /// Get a header value (case-insensitive)
    pub fn header(&self, key: &str) -> Option<&String> {
        let key_lower = key.to_lowercase();
        self.headers.iter()
            .find(|(k, _)| k.to_lowercase() == key_lower)
            .map(|(_, v)| v)
    }
    
    /// Check if request method matches
    pub fn is_method(&self, method: &str) -> bool {
        self.method.eq_ignore_ascii_case(method)
    }
}

impl Default for Request {
    fn default() -> Self {
        Self {
            method: "GET".to_string(),
            path: "/".to_string(),
            query: HashMap::new(),
            headers: HashMap::new(),
            body: None,
            params: HashMap::new(),
            client_ip: None,
            request_id: String::new(),
        }
    }
}

