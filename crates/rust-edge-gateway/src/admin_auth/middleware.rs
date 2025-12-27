//! Authentication middleware for admin routes

use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use base64::Engine;
use std::sync::Arc;
use tracing::info;

use crate::db_admin::{AdminDatabase, ApiKey};
use crate::AppState;

use super::types::ApiKeyExtract;

/// Authentication middleware for admin routes (Basic Auth)
pub async fn admin_auth(
    State(state): State<Arc<AppState>>,
    request: Request,
    next: Next,
) -> Result<Response, (StatusCode, String)> {
    let headers = request.headers();

    // Get authorization header
    let auth_header = headers
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("");

    // Extract username and password from Basic auth
    if !auth_header.starts_with("Basic ") {
        return Err((
            StatusCode::UNAUTHORIZED,
            "Authorization header required".to_string(),
        ));
    }

    let encoded = &auth_header[6..];
    let decoded = match base64::engine::general_purpose::STANDARD.decode(encoded) {
        Ok(decoded) => decoded,
        Err(_) => {
            return Err((
                StatusCode::UNAUTHORIZED,
                "Invalid authorization header".to_string(),
            ))
        }
    };

    let auth_str = match std::str::from_utf8(&decoded) {
        Ok(s) => s,
        Err(_) => {
            return Err((
                StatusCode::UNAUTHORIZED,
                "Invalid authorization header".to_string(),
            ))
        }
    };

    let parts: Vec<&str> = auth_str.splitn(2, ':').collect();
    if parts.len() != 2 {
        return Err((
            StatusCode::UNAUTHORIZED,
            "Invalid authorization header".to_string(),
        ));
    }

    let username = parts[0];
    let password = parts[1];

    // Get admin user from database
    let admin_db = AdminDatabase::new(&state.config.data_dir).map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to initialize admin database".to_string(),
        )
    })?;

    let user = admin_db
        .get_admin_by_username(username)
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to query admin database".to_string(),
            )
        })?
        .ok_or((
            StatusCode::UNAUTHORIZED,
            "Invalid username or password".to_string(),
        ))?;

    // Verify password
    if !bcrypt::verify(password, &user.password_hash).map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Password verification failed".to_string(),
        )
    })? {
        return Err((
            StatusCode::UNAUTHORIZED,
            "Invalid username or password".to_string(),
        ));
    }

    // If password change is required, redirect to password change page
    if user.requires_password_change {
        info!(username = %username, "Admin user requires password change");
        let response = axum::response::Redirect::to("/admin/change-password.html").into_response();
        return Ok(response);
    }

    // Continue to the next middleware/handler
    Ok(next.run(request).await)
}

/// API key validation middleware for API requests
pub async fn api_key_auth(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<ApiKeyExtract, (StatusCode, String)> {
    // Get authorization header
    let auth_header = headers
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("");

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

    Ok(ApiKeyExtract { key })
}

