//! Data types for admin authentication

use crate::db_admin::{AdminUser, ApiKey};

/// Extract admin user from request headers
#[derive(Debug)]
pub struct AdminUserExtract {
    pub user: AdminUser,
}

/// Extract API key from request headers
#[derive(Debug)]
pub struct ApiKeyExtract {
    pub key: ApiKey,
}

/// Login data structure
#[derive(serde::Deserialize)]
pub struct LoginData {
    pub username: String,
    pub password: String,
    pub recaptcha_token: String,
}

/// Login response structure
#[derive(serde::Serialize)]
pub struct LoginResponse {
    pub success: bool,
    pub requires_password_change: bool,
    pub message: String,
}

/// Password change data structure
#[derive(serde::Deserialize)]
pub struct ChangePasswordData {
    pub username: String,
    pub current_password: String,
    pub new_password: String,
}

/// Password change response structure
#[derive(serde::Serialize)]
pub struct PasswordChangeResponse {
    pub success: bool,
    pub message: String,
}

/// API key creation data structure
#[derive(serde::Deserialize)]
pub struct CreateApiKeyData {
    pub label: String,
    pub enabled: bool,
    pub permissions: Vec<String>,
    pub expires_days: i32, // 0 means no expiration
}

/// Generic JSON response structure
#[derive(serde::Serialize)]
pub struct JsonResponse {
    pub success: bool,
    pub data: serde_json::Value,
}

/// Safe API key representation (for listing, hides full key)
#[derive(serde::Serialize)]
pub struct SafeApiKey {
    pub id: String,
    pub label: String,
    pub created_by: String,
    pub created_at: String,
    pub expires_at: Option<String>,
    pub enabled: bool,
    pub permissions: Vec<String>,
    pub key_partial: String, // Only show partial key for security
}

/// API key creation response (includes full key once)
#[derive(serde::Serialize)]
pub struct ApiKeyCreationResponse {
    pub id: String,
    pub label: String,
    pub created_by: String,
    pub created_at: String,
    pub expires_at: Option<String>,
    pub enabled: bool,
    pub permissions: Vec<String>,
    pub key: String,        // Full key shown only at creation
    pub key_partial: String, // Partial key for display
}

