//! Edge Hive SDK - Types and utilities for writing Edge Hive handlers
//!
//! This crate provides the core types and traits that handlers use to interact
//! with the Edge Hive platform.

pub mod request;
pub mod response;
pub mod services;
pub mod ipc;
pub mod error;

pub mod prelude {
    //! Common imports for Edge Hive handlers
    pub use crate::request::Request;
    pub use crate::response::Response;
    pub use crate::services::*;
    pub use crate::ipc::{read_request, send_response};
    pub use crate::error::HandlerError;
    pub use serde::{Deserialize, Serialize};
    pub use serde_json::{json, Value as JsonValue};
}

// Re-export key types at crate root
pub use request::Request;
pub use response::Response;
pub use error::HandlerError;

