# API Key Implementation Summary

## Overview

This document describes the API key enforcement system implemented for the Rust Edge Gateway. The system provides secure access control for both the admin API endpoints and the gateway endpoints.

## Features Implemented

### 1. API Key Management

- **Database Schema Enhancement**: Added proper deletion support and ensured creation time tracking
- **Key Creation**: API keys are shown in full only once (at creation time) for security
- **Key Masking**: All subsequent displays show only partial keys (e.g., `abcd...efgh`)
- **Key Management**: Full CRUD operations (Create, Read, Update, Delete, Enable/Disable)

### 2. Admin UI

- **API Keys Management Page**: Comprehensive UI at `/admin/api-keys/page`
- **Features**:
  - List all API keys with creation dates, status, and permissions
  - Create new API keys with configurable permissions and expiration
  - Enable/Disable API keys
  - Delete API keys permanently
  - Copy-to-clipboard functionality for newly created keys
  - Confirmation dialogs for destructive actions

### 3. Authentication Middleware

- **Gateway API Key Middleware**: Existing middleware enhanced for gateway endpoints
- **Admin API Key Middleware**: New middleware specifically for admin API endpoints
- **Session + API Key Support**: Admin endpoints support both session-based and API key-based authentication

### 4. API Endpoints

#### Admin API Endpoints (API Key Management)

- `GET /admin/api-keys` - List all API keys (masked)
- `POST /admin/api-keys` - Create new API key (returns full key once)
- `POST /admin/api-keys/{id}/enable` - Enable an API key
- `POST /admin/api-keys/{id}/disable` - Disable an API key
- `DELETE /admin/api-keys/{id}` - Delete an API key
- `GET /admin/api-keys/page` - API Keys management UI

#### Admin API Endpoints (Programmatic Access)

All admin API endpoints are now available via API key authentication at `/api-key/*`:

- `/api-key/domains`, `/api-key/collections`, `/api-key/services`, etc.
- Requires API key with `admin` permission
- Uses Bearer token authentication

#### Gateway Endpoints

- All gateway endpoints require API key authentication
- Existing middleware enforced at `/router.rs`
- Rate limiting applied to prevent abuse

## Security Measures

### 1. Key Security

- **One-time Display**: Full API keys shown only at creation time
- **Masking**: Partial display for all subsequent operations
- **Secure Storage**: Keys stored hashed in database
- **Expiration**: Optional expiration dates for keys

### 2. Authentication

- **Bearer Token**: Standard `Authorization: Bearer <API_KEY>` header
- **Permission Checking**: Admin endpoints require `admin` permission
- **Rate Limiting**: 100 requests per minute per API key
- **Status Checking**: Disabled keys are rejected
- **Expiration Checking**: Expired keys are rejected

### 3. Error Handling

- **Clear Error Messages**: Appropriate HTTP status codes and messages
- **Logging**: All key operations logged with key identifiers
- **Validation**: Input validation for all API operations

## Usage Examples

### Creating an API Key

```bash
# Login to admin panel
curl -X POST http://localhost:8081/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"yourpassword","recaptcha_token":"your_token"}'

# Create API key (via UI or API)
curl -X POST http://localhost:8081/admin/api-keys \
  -H "Content-Type: application/json" \
  -H "Cookie: session_id=your_session_id" \
  -d '{"label":"Production App","enabled":true,"permissions":["read","write"],"expires_days":30}'

# Use the API key for programmatic access
curl -X GET http://localhost:8081/api-key/domains \
  -H "Authorization: Bearer your_api_key_here"
```

### Using API Key with Gateway

```bash
# Make a request to a gateway endpoint
curl -X GET http://localhost:8080/your-endpoint \
  -H "Authorization: Bearer your_api_key_here"
```

## Database Schema

### api_keys Table

```sql
CREATE TABLE api_keys (
    id TEXT PRIMARY KEY,
    key TEXT NOT NULL UNIQUE,
    label TEXT NOT NULL,
    created_by TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    expires_at TEXT,
    enabled INTEGER NOT NULL DEFAULT 1,
    permissions TEXT,
    FOREIGN KEY (created_by) REFERENCES admin_users(id)
);
```

## Configuration

No additional configuration required. The system uses the existing:

- `DEFAULT_ADMIN_PASSWORD` for initial admin setup
- `data_dir` for database storage
- Standard rate limiting configuration

## Testing

The implementation includes:

1. **Unit Tests**: Database operations tested in `db_admin.rs`
2. **Integration Testing**: Manual testing via UI and API endpoints
3. **Error Scenarios**: Invalid keys, expired keys, disabled keys
4. **Permission Testing**: Admin vs non-admin permissions

## Deployment Notes

1. **Migration**: Existing API keys will continue to work
2. **Backward Compatibility**: Session authentication still supported
3. **Security**: Ensure proper HTTPS configuration in production
4. **Monitoring**: API key usage can be monitored via existing logging

## Future Enhancements

- API key rotation functionality
- More granular permissions (per endpoint/resource)
- Usage statistics and analytics
- IP whitelisting for API keys
- Webhook notifications for key operations