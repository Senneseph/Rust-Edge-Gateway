//! API key management handlers

use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
};
use std::sync::Arc;
use tracing::{error, info};

use crate::db_admin::AdminDatabase;
use crate::AppState;

use super::types::{ApiKeyCreationResponse, CreateApiKeyData, JsonResponse, SafeApiKey};

/// Handler for listing all API keys
pub async fn list_api_keys(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let admin_db = AdminDatabase::new(&state.config.data_dir).map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to initialize admin database".to_string(),
        )
    })?;

    // Get the admin user ID to filter API keys
    let admin_user = admin_db
        .get_admin_by_username("admin")
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to get admin user".to_string(),
            )
        })?
        .ok_or((
            StatusCode::INTERNAL_SERVER_ERROR,
            "Admin user not found".to_string(),
        ))?;

    // List API keys for the admin user
    let api_keys = admin_db.list_api_keys(&admin_user.id).map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to query admin database".to_string(),
        )
    })?;

    // Create a safe response that masks the API keys
    let safe_keys: Vec<SafeApiKey> = api_keys
        .iter()
        .map(|key| {
            let key_partial = if key.key.len() > 8 {
                format!("{}...{}", &key.key[..4], &key.key[key.key.len() - 4..])
            } else {
                key.key.clone()
            };

            SafeApiKey {
                id: key.id.clone(),
                label: key.label.clone(),
                created_by: key.created_by.clone(),
                created_at: key.created_at.to_rfc3339(),
                expires_at: key.expires_at.map(|dt| dt.to_rfc3339()),
                enabled: key.enabled,
                permissions: key.permissions.clone(),
                key_partial,
            }
        })
        .collect();

    Ok(axum::Json(JsonResponse {
        success: true,
        data: serde_json::to_value(safe_keys).unwrap(),
    }))
}

/// Handler for creating a new API key
pub async fn create_api_key(
    State(state): State<Arc<AppState>>,
    axum::extract::Json(create_data): axum::extract::Json<CreateApiKeyData>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    info!("Starting API key creation process");

    let admin_db = AdminDatabase::new(&state.config.data_dir).map_err(|e| {
        error!("Failed to initialize admin database: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to initialize admin database: {}", e),
        )
    })?;

    info!("Successfully initialized admin database");

    // Get the admin user ID to use as created_by
    let admin_user = admin_db
        .get_admin_by_username("admin")
        .map_err(|e| {
            error!("Failed to get admin user: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to get admin user: {}", e),
            )
        })?
        .ok_or((
            StatusCode::INTERNAL_SERVER_ERROR,
            "Admin user not found".to_string(),
        ))?;

    info!("Found admin user with ID: {}", admin_user.id);

    // Check API key limit (256 max)
    let existing_keys = admin_db.list_api_keys(&admin_user.id).map_err(|e| {
        error!("Failed to query admin database for existing keys: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to query admin database: {}", e),
        )
    })?;

    info!("Found {} existing API keys", existing_keys.len());

    if existing_keys.len() >= 256 {
        return Err((
            StatusCode::BAD_REQUEST,
            "Maximum limit of 256 API keys reached. Please delete unused keys before creating new ones.".to_string(),
        ));
    }

    // Create the API key using the database method
    info!(
        "Creating API key with label: {}, permissions: {:?}, expires_days: {}",
        create_data.label, create_data.permissions, create_data.expires_days
    );

    let api_key = admin_db
        .create_api_key(
            &create_data.label,
            &admin_user.id,
            create_data.permissions,
            create_data.expires_days,
        )
        .map_err(|e| {
            error!("Failed to create API key in database: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to create API key: {}", e),
            )
        })?;

    info!(key = %api_key.key, "New API key created successfully");

    let key_partial = if api_key.key.len() > 8 {
        format!(
            "{}...{}",
            &api_key.key[..4],
            &api_key.key[api_key.key.len() - 4..]
        )
    } else {
        api_key.key.clone()
    };

    let response_data = ApiKeyCreationResponse {
        id: api_key.id,
        label: api_key.label,
        created_by: api_key.created_by,
        created_at: api_key.created_at.to_rfc3339(),
        expires_at: api_key.expires_at.map(|dt| dt.to_rfc3339()),
        enabled: api_key.enabled,
        permissions: api_key.permissions,
        key: api_key.key.clone(),
        key_partial,
    };

    Ok(axum::Json(JsonResponse {
        success: true,
        data: serde_json::to_value(response_data).unwrap(),
    }))
}