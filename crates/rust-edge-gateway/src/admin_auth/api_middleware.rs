//! API key authentication middleware for resource-specific endpoints

use axum::{
    extract::{Request, State},
    http::{header::AUTHORIZATION, Method, StatusCode},
    middleware::Next,
    response::Response,
};
use std::sync::Arc;

use crate::db_admin::{AdminDatabase, ApiKey};
use crate::AppState;

/// Helper function to validate API key and check permissions
/// Returns the validated API key on success.
fn validate_api_key_with_permission_sync(
    state: &Arc<AppState>,
    auth_header: &str,
    method: &Method,
    resource: &str,
) -> Result<ApiKey, (StatusCode, String)> {
    // Extract API key from Bearer auth
    if !auth_header.starts_with("Bearer ") {
        return Err((StatusCode::UNAUTHORIZED, "API key required".to_string()));
    }

    let api_key_str = &auth_header[7..];

    // Get API key from database
    let admin_db = AdminDatabase::new(&state.config.data_dir).map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to initialize admin database".to_string(),
        )
    })?;

    let key = admin_db
        .get_api_key_by_value(api_key_str)
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to query admin database".to_string(),
            )
        })?
        .ok_or((StatusCode::UNAUTHORIZED, "Invalid API key".to_string()))?;

    // Check if API key is enabled
    if !key.enabled {
        return Err((StatusCode::UNAUTHORIZED, "API key is disabled".to_string()));
    }

    // Check if API key has expired
    if let Some(expires_at) = key.expires_at {
        if chrono::Utc::now() > expires_at {
            return Err((StatusCode::UNAUTHORIZED, "API key has expired".to_string()));
        }
    }

    // Determine required permission based on HTTP method
    let permission_type = if *method == Method::GET { "read" } else { "write" };

    let required_permission = format!("{}:{}", resource, permission_type);
    let wildcard_permission = format!("{}:*", resource);

    // Check if API key has the required permission
    if !key.permissions.contains(&required_permission)
        && !key.permissions.contains(&wildcard_permission)
    {
        return Err((
            StatusCode::FORBIDDEN,
            format!(
                "API key does not have '{}' permission (requires '{}' or '{}')",
                resource, required_permission, wildcard_permission
            ),
        ));
    }

    Ok(key)
}

/// API key authentication middleware for endpoints API
pub async fn endpoints_api_key_auth(
    State(state): State<Arc<AppState>>,
    request: Request,
    next: Next,
) -> Result<Response, (StatusCode, String)> {
    let auth_header = request
        .headers()
        .get(AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .unwrap_or("")
        .to_string();
    let method = request.method().clone();
    validate_api_key_with_permission_sync(&state, &auth_header, &method, "endpoints")?;
    Ok(next.run(request).await)
}

/// API key authentication middleware for services API
pub async fn services_api_key_auth(
    State(state): State<Arc<AppState>>,
    request: Request,
    next: Next,
) -> Result<Response, (StatusCode, String)> {
    let auth_header = request
        .headers()
        .get(AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .unwrap_or("")
        .to_string();
    let method = request.method().clone();
    validate_api_key_with_permission_sync(&state, &auth_header, &method, "services")?;
    Ok(next.run(request).await)
}

/// API key authentication middleware for domains API
pub async fn domains_api_key_auth(
    State(state): State<Arc<AppState>>,
    request: Request,
    next: Next,
) -> Result<Response, (StatusCode, String)> {
    let auth_header = request
        .headers()
        .get(AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .unwrap_or("")
        .to_string();
    let method = request.method().clone();
    validate_api_key_with_permission_sync(&state, &auth_header, &method, "domains")?;
    Ok(next.run(request).await)
}

/// API key authentication middleware for collections API
pub async fn collections_api_key_auth(
    State(state): State<Arc<AppState>>,
    request: Request,
    next: Next,
) -> Result<Response, (StatusCode, String)> {
    let auth_header = request
        .headers()
        .get(AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .unwrap_or("")
        .to_string();
    let method = request.method().clone();
    validate_api_key_with_permission_sync(&state, &auth_header, &method, "collections")?;
    Ok(next.run(request).await)
}
