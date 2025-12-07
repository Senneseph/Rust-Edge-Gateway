//! Admin API endpoints

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::AppState;

/// Endpoint configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Endpoint {
    pub id: String,
    pub name: String,
    pub domain: String,
    pub path: String,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    pub compiled: bool,
    pub enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
}

/// Request to create a new endpoint
#[derive(Debug, Deserialize)]
pub struct CreateEndpointRequest {
    pub name: String,
    pub domain: String,
    pub path: String,
    #[serde(default = "default_method")]
    pub method: String,
    pub code: Option<String>,
}

fn default_method() -> String {
    "GET".to_string()
}

/// Request to update an endpoint
#[derive(Debug, Deserialize)]
pub struct UpdateEndpointRequest {
    pub name: Option<String>,
    pub domain: Option<String>,
    pub path: Option<String>,
    pub method: Option<String>,
    pub enabled: Option<bool>,
}

/// Code update request
#[derive(Debug, Deserialize)]
pub struct UpdateCodeRequest {
    pub code: String,
}

/// API response wrapper
#[derive(Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T> ApiResponse<T> {
    pub fn ok(data: T) -> Self {
        Self { success: true, data: Some(data), error: None }
    }
    
    pub fn err(error: impl Into<String>) -> Self {
        Self { success: false, data: None, error: Some(error.into()) }
    }
}

/// Health check endpoint
pub async fn health_check() -> Json<ApiResponse<&'static str>> {
    Json(ApiResponse::ok("healthy"))
}

/// Get system stats
#[derive(Serialize)]
pub struct Stats {
    pub endpoint_count: i64,
    pub active_workers: usize,
}

pub async fn get_stats(State(state): State<Arc<AppState>>) -> Json<ApiResponse<Stats>> {
    let endpoint_count = state.db.endpoint_count().unwrap_or(0);
    let workers = state.workers.read().await;
    let active_workers = workers.active_count();
    
    Json(ApiResponse::ok(Stats { endpoint_count, active_workers }))
}

/// List all endpoints
pub async fn list_endpoints(
    State(state): State<Arc<AppState>>,
) -> Result<Json<ApiResponse<Vec<Endpoint>>>, StatusCode> {
    match state.db.list_endpoints() {
        Ok(endpoints) => Ok(Json(ApiResponse::ok(endpoints))),
        Err(e) => {
            tracing::error!("Failed to list endpoints: {}", e);
            Ok(Json(ApiResponse::err(e.to_string())))
        }
    }
}

/// Create a new endpoint
pub async fn create_endpoint(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateEndpointRequest>,
) -> Result<Json<ApiResponse<Endpoint>>, StatusCode> {
    let endpoint = Endpoint {
        id: Uuid::new_v4().to_string(),
        name: req.name,
        domain: req.domain,
        path: req.path,
        method: req.method.to_uppercase(),
        code: req.code,
        compiled: false,
        enabled: false,
        created_at: None,
        updated_at: None,
    };
    
    match state.db.create_endpoint(&endpoint) {
        Ok(_) => Ok(Json(ApiResponse::ok(endpoint))),
        Err(e) => {
            tracing::error!("Failed to create endpoint: {}", e);
            Ok(Json(ApiResponse::err(e.to_string())))
        }
    }
}

/// Get an endpoint by ID
pub async fn get_endpoint(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<Endpoint>>, StatusCode> {
    match state.db.get_endpoint(&id) {
        Ok(Some(endpoint)) => Ok(Json(ApiResponse::ok(endpoint))),
        Ok(None) => Ok(Json(ApiResponse::err("Endpoint not found"))),
        Err(e) => Ok(Json(ApiResponse::err(e.to_string()))),
    }
}

/// Update an endpoint
pub async fn update_endpoint(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(req): Json<UpdateEndpointRequest>,
) -> Result<Json<ApiResponse<Endpoint>>, StatusCode> {
    let existing = match state.db.get_endpoint(&id) {
        Ok(Some(e)) => e,
        Ok(None) => return Ok(Json(ApiResponse::err("Endpoint not found"))),
        Err(e) => return Ok(Json(ApiResponse::err(e.to_string()))),
    };

    let updated = Endpoint {
        id: existing.id,
        name: req.name.unwrap_or(existing.name),
        domain: req.domain.unwrap_or(existing.domain),
        path: req.path.unwrap_or(existing.path),
        method: req.method.map(|m| m.to_uppercase()).unwrap_or(existing.method),
        code: existing.code,
        compiled: existing.compiled,
        enabled: req.enabled.unwrap_or(existing.enabled),
        created_at: existing.created_at,
        updated_at: existing.updated_at,
    };

    match state.db.update_endpoint(&updated) {
        Ok(_) => Ok(Json(ApiResponse::ok(updated))),
        Err(e) => Ok(Json(ApiResponse::err(e.to_string()))),
    }
}

/// Delete an endpoint
pub async fn delete_endpoint(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<()>>, StatusCode> {
    // Stop the worker if running
    {
        let mut workers = state.workers.write().await;
        workers.stop_worker(&id);
    }

    match state.db.delete_endpoint(&id) {
        Ok(_) => Ok(Json(ApiResponse::ok(()))),
        Err(e) => Ok(Json(ApiResponse::err(e.to_string()))),
    }
}

/// Get endpoint code
pub async fn get_endpoint_code(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    match state.db.get_endpoint(&id) {
        Ok(Some(endpoint)) => Ok(Json(ApiResponse::ok(endpoint.code.unwrap_or_default()))),
        Ok(None) => Ok(Json(ApiResponse::err("Endpoint not found"))),
        Err(e) => Ok(Json(ApiResponse::err(e.to_string()))),
    }
}

/// Update endpoint code
pub async fn update_endpoint_code(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(req): Json<UpdateCodeRequest>,
) -> Result<Json<ApiResponse<()>>, StatusCode> {
    match state.db.update_endpoint_code(&id, &req.code) {
        Ok(_) => Ok(Json(ApiResponse::ok(()))),
        Err(e) => Ok(Json(ApiResponse::err(e.to_string()))),
    }
}

/// Compile an endpoint
pub async fn compile_endpoint(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    let endpoint = match state.db.get_endpoint(&id) {
        Ok(Some(e)) => e,
        Ok(None) => return Ok(Json(ApiResponse::err("Endpoint not found"))),
        Err(e) => return Ok(Json(ApiResponse::err(e.to_string()))),
    };

    let code = match endpoint.code {
        Some(c) => c,
        None => return Ok(Json(ApiResponse::err("No code to compile"))),
    };

    // Compile the handler
    match crate::compiler::compile_handler(&state.config, &id, &code).await {
        Ok(binary_path) => {
            state.db.mark_compiled(&id, true).ok();
            Ok(Json(ApiResponse::ok(format!("Compiled to {}", binary_path))))
        }
        Err(e) => Ok(Json(ApiResponse::err(format!("Compilation failed: {}", e)))),
    }
}

/// Start an endpoint (spawn worker)
pub async fn start_endpoint(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<()>>, StatusCode> {
    let endpoint = match state.db.get_endpoint(&id) {
        Ok(Some(e)) if e.compiled => e,
        Ok(Some(_)) => return Ok(Json(ApiResponse::err("Endpoint not compiled"))),
        Ok(None) => return Ok(Json(ApiResponse::err("Endpoint not found"))),
        Err(e) => return Ok(Json(ApiResponse::err(e.to_string()))),
    };

    let mut workers = state.workers.write().await;
    match workers.start_worker(&endpoint) {
        Ok(_) => {
            state.db.update_endpoint(&Endpoint { enabled: true, ..endpoint }).ok();
            Ok(Json(ApiResponse::ok(())))
        }
        Err(e) => Ok(Json(ApiResponse::err(e.to_string()))),
    }
}

/// Stop an endpoint (kill worker)
pub async fn stop_endpoint(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<()>>, StatusCode> {
    let mut workers = state.workers.write().await;
    workers.stop_worker(&id);
    Ok(Json(ApiResponse::ok(())))
}

