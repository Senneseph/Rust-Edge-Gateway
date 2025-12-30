# Service Provider Examples

This directory contains example service configurations that demonstrate how to configure and use various backend services with Rust Edge Gateway.

## Available Examples

- **PostgreSQL**: Example configuration for PostgreSQL database
- **MySQL**: Example configuration for MySQL database  
- **SQLite**: Example configuration for SQLite embedded database
- **Redis**: Example configuration for Redis cache
- **MinIO**: Example configuration for MinIO/S3 object storage
- **FTP/SFTP**: Example configuration for file transfer services
- **Email**: Example configuration for SMTP email sending

## How to Use These Examples

### 1. Via API

You can create services using these examples via the REST API:

```bash
# Create a PostgreSQL service
curl -X POST http://localhost:9081/api/services \
  -H "Content-Type: application/json" \
  -H "X-API-Key: your-api-key-here" \
  -d @examples/services/postgres.json

# Create a Redis service  
curl -X POST http://localhost:9081/api/services \
  -H "Content-Type: application/json" \
  -H "X-API-Key: your-api-key-here" \
  -d @examples/services/redis.json
```

### 2. Via Admin UI

1. Go to the Services section in the Admin UI
2. Click "Create Service"
3. Select the service type
4. Copy the configuration from the appropriate example file
5. Test and save the service

### 3. Testing Services

After creating a service, you can test the connection:

```bash
curl -X POST http://localhost:9081/api/services/{service-id}/test \
  -H "X-API-Key: your-api-key-here"
```

### 4. Activating Services

Services need to be activated to start the service actor:

```bash
curl -X POST http://localhost:9081/api/services/{service-id}/activate \
  -H "X-API-Key: your-api-key-here"
```

## Service Configuration Reference

Each service type has specific configuration requirements:

- **Database services** (PostgreSQL, MySQL, SQLite): Require connection strings/parameters
- **Cache services** (Redis, Memcached): Require host/port and optional authentication
- **Storage services** (MinIO): Require endpoint, access/secret keys, and bucket
- **File transfer services** (FTP/SFTP): Require host, credentials, and protocol
- **Email services** (SMTP): Require SMTP server details and authentication

See the individual example files for detailed configuration options.