//! API key enable/disable/delete handlers

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use std::sync::Arc;
use tracing::info;

use crate::db_admin::AdminDatabase;
use crate::AppState;

use super::types::JsonResponse;

/// Handler for disabling an API key by ID
pub async fn disable_api_key(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let admin_db = AdminDatabase::new(&state.config.data_dir).map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to initialize admin database".to_string(),
        )
    })?;

    // First get the key to log it, then disable by ID
    if let Some(key) = admin_db.get_api_key_by_id(&id).map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to query admin database".to_string(),
        )
    })? {
        admin_db.disable_api_key(&key.key).map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to disable API key".to_string(),
            )
        })?;

        info!(key = %key.key, "API key disabled");
    } else {
        return Err((StatusCode::NOT_FOUND, "API key not found".to_string()));
    }

    Ok(axum::Json(JsonResponse {
        success: true,
        data: serde_json::Value::String("API key disabled".to_string()),
    }))
}

/// Handler for enabling an API key by ID
pub async fn enable_api_key(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let admin_db = AdminDatabase::new(&state.config.data_dir).map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to initialize admin database".to_string(),
        )
    })?;

    // First get the key to log it, then enable by ID
    if let Some(key) = admin_db.get_api_key_by_id(&id).map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to query admin database".to_string(),
        )
    })? {
        admin_db.enable_api_key(&key.key).map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to enable API key".to_string(),
            )
        })?;

        info!(key = %key.key, "API key enabled");
    } else {
        return Err((StatusCode::NOT_FOUND, "API key not found".to_string()));
    }

    Ok(axum::Json(JsonResponse {
        success: true,
        data: serde_json::Value::String("API key enabled".to_string()),
    }))
}

/// Handler for deleting an API key by ID
pub async fn delete_api_key(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let admin_db = AdminDatabase::new(&state.config.data_dir).map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to initialize admin database".to_string(),
        )
    })?;

    // First get the key to log it, then delete by ID
    if let Some(key) = admin_db.get_api_key_by_id(&id).map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to query admin database".to_string(),
        )
    })? {
        admin_db.delete_api_key(&key.key).map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to delete API key".to_string(),
            )
        })?;

        info!(key = %key.key, "API key deleted");
    } else {
        return Err((StatusCode::NOT_FOUND, "API key not found".to_string()));
    }

    Ok(axum::Json(JsonResponse {
        success: true,
        data: serde_json::Value::String("API key deleted".to_string()),
    }))
}

