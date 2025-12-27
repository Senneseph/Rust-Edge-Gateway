# API Key Implementation Summary

## Overview

This document describes the API key enforcement system implemented for the Rust Edge Gateway. The system provides secure access control for both the admin API endpoints and the gateway endpoints.

## Route Structure

All API access is consolidated under `/api` with proper sub-paths:

| Route | Authentication | Description |
|-------|---------------|-------------|
| `/api/endpoints/*` | API Key (endpoints:*) | Endpoint management |
| `/api/services/*` | API Key (services:*) | Service management |
| `/api/domains/*` | API Key (domains:*) | Domain management |
| `/api/collections/*` | API Key (collections:*) | Collection management |
| `/api/admin/*` | Session (Admin UI) | Admin operations (API key management, imports, etc.) |
| `/api/health` | Public | Health check endpoint |
| `/auth/*` | Public | Authentication routes (login, logout, password change) |

## Features Implemented

### 1. API Key Management

- **Database Schema Enhancement**: Added proper deletion support and ensured creation time tracking
- **Key Creation**: API keys are shown in full only once (at creation time) for security
- **Key Masking**: All subsequent displays show only partial keys (e.g., `abcd...efgh`)
- **Key Management**: Full CRUD operations (Create, Read, Update, Delete, Enable/Disable)

### 2. Admin UI

- **API Keys Management Page**: Comprehensive UI at `/api/admin/api-keys/page`
- **Features**:
  - List all API keys with creation dates, status, and permissions
  - Create new API keys with configurable permissions and expiration
  - Enable/Disable API keys
  - Delete API keys permanently
  - Copy-to-clipboard functionality for newly created keys
  - Confirmation dialogs for destructive actions

### 3. Authentication Middleware

- **Granular API Key Middleware**: Resource-specific middleware for endpoints, services, domains, and collections
- **Session Authentication**: Admin UI operations protected by session-based authentication
- **Permission Checking**: API keys require specific resource permissions (e.g., `endpoints:read`, `services:write`)

### 4. API Endpoints

#### Admin API Endpoints (Session Authentication - Admin UI Only)

- `GET /api/admin/api-keys` - List all API keys (masked)
- `POST /api/admin/api-keys` - Create new API key (returns full key once)
- `POST /api/admin/api-keys/{id}/enable` - Enable an API key
- `POST /api/admin/api-keys/{id}/disable` - Disable an API key
- `DELETE /api/admin/api-keys/{id}` - Delete an API key
- `GET /api/admin/api-keys/page` - API Keys management UI
- `GET /api/admin/stats` - System statistics
- `POST /api/admin/import/openapi` - Import OpenAPI specification
- `POST /api/admin/import/bundle` - Import bundle

#### Resource API Endpoints (API Key Authentication)

**Endpoints** (requires `endpoints:read` or `endpoints:write` or `endpoints:*`):
- `GET /api/endpoints` - List endpoints
- `POST /api/endpoints` - Create endpoint
- `GET /api/endpoints/{id}` - Get endpoint
- `PUT /api/endpoints/{id}` - Update endpoint
- `DELETE /api/endpoints/{id}` - Delete endpoint
- `GET /api/endpoints/{id}/code` - Get endpoint code
- `PUT /api/endpoints/{id}/code` - Update endpoint code
- `POST /api/endpoints/{id}/compile` - Compile endpoint
- `POST /api/endpoints/{id}/start` - Start endpoint
- `POST /api/endpoints/{id}/stop` - Stop endpoint

**Services** (requires `services:read` or `services:write` or `services:*`):
- `GET /api/services` - List services
- `POST /api/services` - Create service
- `GET /api/services/{id}` - Get service
- `PUT /api/services/{id}` - Update service
- `DELETE /api/services/{id}` - Delete service
- `POST /api/services/{id}/test` - Test service
- `POST /api/services/{id}/activate` - Activate service
- `POST /api/services/{id}/deactivate` - Deactivate service

**Domains** (requires `domains:read` or `domains:write` or `domains:*`):
- `GET /api/domains` - List domains
- `POST /api/domains` - Create domain
- `GET /api/domains/{id}` - Get domain
- `PUT /api/domains/{id}` - Update domain
- `DELETE /api/domains/{id}` - Delete domain
- `GET /api/domains/{id}/collections` - List domain collections

**Collections** (requires `collections:read` or `collections:write` or `collections:*`):
- `GET /api/collections` - List collections
- `POST /api/collections` - Create collection
- `GET /api/collections/{id}` - Get collection
- `PUT /api/collections/{id}` - Update collection
- `DELETE /api/collections/{id}` - Delete collection

#### Gateway Endpoints

- All gateway endpoints require API key authentication
- Existing middleware enforced at `/router.rs`
- Rate limiting applied to prevent abuse

## Permissions Model

API keys use a granular permission system based on resource types and access levels:

### Permission Format

Permissions follow the format: `{resource}:{access_level}`

**Resources:**
- `endpoints` - Endpoint management
- `services` - Service management
- `domains` - Domain management
- `collections` - Collection management

**Access Levels:**
- `read` - Read-only access (GET requests)
- `write` - Write access (POST, PUT, DELETE requests)
- `*` - Full access (both read and write)

### Examples

| Permission | Description |
|------------|-------------|
| `endpoints:read` | Read-only access to endpoints |
| `endpoints:write` | Write access to endpoints |
| `endpoints:*` | Full access to endpoints |
| `services:*` | Full access to services |
| `domains:read` | Read-only access to domains |

### Permission Checking

- GET requests require `{resource}:read` or `{resource}:*`
- POST/PUT/DELETE requests require `{resource}:write` or `{resource}:*`

## Security Measures

### 1. Key Security

- **One-time Display**: Full API keys shown only at creation time
- **Masking**: Partial display for all subsequent operations
- **Secure Storage**: Keys stored hashed in database
- **Expiration**: Optional expiration dates for keys

### 2. Authentication

- **Bearer Token**: Standard `Authorization: Bearer <API_KEY>` header
- **Granular Permissions**: Resource-specific permissions (e.g., `endpoints:read`, `services:write`)
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

# Create API key with granular permissions (via UI or API)
curl -X POST http://localhost:8081/api/admin/api-keys \
  -H "Content-Type: application/json" \
  -H "Cookie: session_id=your_session_id" \
  -d '{"label":"CI/CD Pipeline","enabled":true,"permissions":["endpoints:*","services:read"],"expires_days":30}'
```

### Using API Key for Programmatic Access

```bash
# List endpoints (requires endpoints:read or endpoints:*)
curl -X GET http://localhost:8081/api/endpoints \
  -H "Authorization: Bearer your_api_key_here"

# Create a service (requires services:write or services:*)
curl -X POST http://localhost:8081/api/services \
  -H "Authorization: Bearer your_api_key_here" \
  -H "Content-Type: application/json" \
  -d '{"name":"my-service","url":"http://backend:3000"}'

# List domains (requires domains:read or domains:*)
curl -X GET http://localhost:8081/api/domains \
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
    permissions TEXT,  -- JSON array: ["endpoints:*", "services:read"]
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
4. **Permission Testing**: Granular permission checking per resource

## Deployment Notes

1. **Migration**: Existing API keys with old permissions (`read`, `write`, `admin`) will need to be updated to use the new granular permissions
2. **Backward Compatibility**: Session authentication still supported for Admin UI
3. **Security**: Ensure proper HTTPS configuration in production
4. **Monitoring**: API key usage can be monitored via existing logging

## Future Enhancements

- API key rotation functionality
- Usage statistics and analytics
- IP whitelisting for API keys
- Webhook notifications for key operations
- Audit logging for API key operations