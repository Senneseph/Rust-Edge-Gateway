//! Router creation for admin authentication

use axum::{
    routing::{delete, get, post},
    Router,
};
use std::sync::Arc;

use crate::AppState;

use super::api_keys::{create_api_key, list_api_keys};
use super::api_keys_actions::{delete_api_key, disable_api_key, enable_api_key};
use super::handlers::{change_password, get_recaptcha_site_key, login, logout};

/// Create admin authentication routes (public routes only - login, password change)
pub fn create_admin_auth_router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/login", post(login))
        .route("/change-password", post(change_password))
        .route("/logout", get(logout).post(logout))
}

/// Create protected admin routes (requires authentication)
pub fn create_protected_admin_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api-keys", get(list_api_keys).post(create_api_key))
        .route("/api-keys/{id}/enable", post(enable_api_key))
        .route("/api-keys/{id}/disable", post(disable_api_key))
        .route("/api-keys/{id}", delete(delete_api_key))
        .route("/recaptcha-site-key", get(get_recaptcha_site_key))
}

