# Service Provider API Testing Plan

## Overview

This document outlines the comprehensive testing plan for the Service Provider API. We will test creating, managing, and using Service Providers via the REST API.

## API Endpoints

Based on the current implementation in [`crates/rust-edge-gateway/src/api.rs`](crates/rust-edge-gateway/src/api.rs), the Service Provider API endpoints are:

- `GET /api/services` - List all services
- `POST /api/services` - Create a new service
- `GET /api/services/{id}` - Get service details
- `PUT /api/services/{id}` - Update a service
- `DELETE /api/services/{id}` - Delete a service
- `POST /api/services/{id}/test` - Test service connection
- `POST /api/services/{id}/activate` - Activate service (start actor)
- `POST /api/services/{id}/deactivate` - Deactivate service (stop actor)

## Authentication

All API requests require the API key:
```
RUST_EDGE_GATEWAY_API_KEY=e9e4d2b4-8dec-4b23-b15c-149cc182630e
```

## Test Environment

- **Base URL**: `https://rust-edge-gateway.iffuso.com`
- **Admin Port**: Typically `9081` for admin API
- **Gateway Port**: Typically `9080` for gateway traffic

## Test Cases

### 1. List Services (GET /api/services)

**Request:**
```bash
curl -X GET https://rust-edge-gateway.iffuso.com/api/services \
  -H "X-API-Key: e9e4d2b4-8dec-4b23-b15c-149cc182630e"
```

**Expected Response:**
```json
{
  "success": true,
  "data": []
}
```

### 2. Create PostgreSQL Service (POST /api/services)

**Request:**
```bash
curl -X POST https://rust-edge-gateway.iffuso.com/api/services \
  -H "Content-Type: application/json" \
  -H "X-API-Key: e9e4d2b4-8dec-4b23-b15c-149cc182630e" \
  -d @examples/services/postgres/postgres.json
```

**Expected Response:**
```json
{
  "success": true,
  "data": {
    "id": "generated-uuid",
    "name": "Main PostgreSQL Database",
    "service_type": "postgres",
    "config": {
      "host": "localhost",
      "port": 5432,
      "database": "myapp",
      "username": "app_user",
      "ssl_mode": "prefer",
      "pool_size": 10
    },
    "enabled": true,
    "created_at": "timestamp",
    "updated_at": "timestamp"
  }
}
```

### 3. Create MySQL Service (POST /api/services)

**Request:**
```bash
curl -X POST https://rust-edge-gateway.iffuso.com/api/services \
  -H "Content-Type: application/json" \
  -H "X-API-Key: e9e4d2b4-8dec-4b23-b15c-149cc182630e" \
  -d @examples/services/mysql/mysql.json
```

### 4. Create Redis Service (POST /api/services)

**Request:**
```bash
curl -X POST https://rust-edge-gateway.iffuso.com/api/services \
  -H "Content-Type: application/json" \
  -H "X-API-Key: e9e4d2b4-8dec-4b23-b15c-149cc182630e" \
  -d @examples/services/redis/redis.json
```

### 5. Create MinIO Service (POST /api/services)

**Request:**
```bash
curl -X POST https://rust-edge-gateway.iffuso.com/api/services \
  -H "Content-Type: application/json" \
  -H "X-API-Key: e9e4d2b4-8dec-4b23-b15c-149cc182630e" \
  -d @examples/services/minio/minio.json
```

### 6. Create Email Service (POST /api/services)

**Request:**
```bash
curl -X POST https://rust-edge-gateway.iffuso.com/api/services \
  -H "Content-Type: application/json" \
  -H "X-API-Key: e9e4d2b4-8dec-4b23-b15c-149cc182630e" \
  -d @examples/services/smtp/email.json
```

### 7. List All Services After Creation

**Request:**
```bash
curl -X GET https://rust-edge-gateway.iffuso.com/api/services \
  -H "X-API-Key: e9e4d2b4-8dec-4b23-b15c-149cc182630e"
```

**Expected Response:** Should show all 5 created services

### 8. Get Specific Service Details

**Request:**
```bash
# Replace {service-id} with the ID from creation response
curl -X GET https://rust-edge-gateway.iffuso.com/api/services/{service-id} \
  -H "X-API-Key: e9e4d2b4-8dec-4b23-b15c-149cc182630e"
```

### 9. Test Service Connection

**Request:**
```bash
# Replace {service-id} with the ID from creation response
curl -X POST https://rust-edge-gateway.iffuso.com/api/services/{service-id}/test \
  -H "X-API-Key: e9e4d2b4-8dec-4b23-b15c-149cc182630e"
```

**Expected Response:**
```json
{
  "success": true,
  "data": {
    "id": "service-id",
    "name": "service-name",
    "service_type": "postgres",
    "connected": true,
    "error": null,
    "info": {
      "type": "postgres",
      "host": "localhost",
      "port": 5432,
      "database": "myapp",
      "username": "app_user",
      "ssl_mode": "prefer",
      "pool_size": 10
    }
  }
}
```

### 10. Activate Service (Start Actor)

**Request:**
```bash
# Replace {service-id} with the ID from creation response
curl -X POST https://rust-edge-gateway.iffuso.com/api/services/{service-id}/activate \
  -H "X-API-Key: e9e4d2b4-8dec-4b23-b15c-149cc182630e"
```

**Expected Response:**
```json
{
  "success": true,
  "data": {
    "id": "service-id",
    "name": "service-name",
    "service_type": "postgres",
    "active": true,
    "message": "PostgreSQL service actor started successfully"
  }
}
```

### 11. Update Service Configuration

**Request:**
```bash
# Replace {service-id} with the ID from creation response
curl -X PUT https://rust-edge-gateway.iffuso.com/api/services/{service-id} \
  -H "Content-Type: application/json" \
  -H "X-API-Key: e9e4d2b4-8dec-4b23-b15c-149cc182630e" \
  -d '{
    "name": "Updated Service Name",
    "config": {
      "host": "new-host.example.com",
      "pool_size": 20
    }
  }'
```

### 12. Deactivate Service (Stop Actor)

**Request:**
```bash
# Replace {service-id} with the ID from creation response
curl -X POST https://rust-edge-gateway.iffuso.com/api/services/{service-id}/deactivate \
  -H "X-API-Key: e9e4d2b4-8dec-4b23-b15c-149cc182630e"
```

**Expected Response:**
```json
{
  "success": true,
  "data": {
    "id": "service-id",
    "name": "service-name",
    "service_type": "postgres",
    "active": false,
    "message": "Service deactivated"
  }
}
```

### 13. Delete Service

**Request:**
```bash
# Replace {service-id} with the ID from creation response
curl -X DELETE https://rust-edge-gateway.iffuso.com/api/services/{service-id} \
  -H "X-API-Key: e9e4d2b4-8dec-4b23-b15c-149cc182630e"
```

**Expected Response:**
```json
{
  "success": true,
  "data": null
}
```

## Error Handling Tests

### 14. Invalid API Key

**Request:**
```bash
curl -X GET https://rust-edge-gateway.iffuso.com/api/services \
  -H "X-API-Key: invalid-key"
```

**Expected Response:** `401 Unauthorized`

### 15. Invalid Service Type

**Request:**
```bash
curl -X POST https://rust-edge-gateway.iffuso.com/api/services \
  -H "Content-Type: application/json" \
  -H "X-API-Key: e9e4d2b4-8dec-4b23-b15c-149cc182630e" \
  -d '{
    "name": "Invalid Service",
    "service_type": "invalid-type",
    "config": {}
  }'
```

**Expected Response:**
```json
{
  "success": false,
  "data": null,
  "error": "Unknown service type: invalid-type"
}
```

### 16. Missing Required Fields

**Request:**
```bash
curl -X POST https://rust-edge-gateway.iffuso.com/api/services \
  -H "Content-Type: application/json" \
  -H "X-API-Key: e9e4d2b4-8dec-4b23-b15c-149cc182630e" \
  -d '{
    "service_type": "postgres",
    "config": {}
  }'
```

**Expected Response:** Error about missing required fields

### 17. Non-existent Service ID

**Request:**
```bash
curl -X GET https://rust-edge-gateway.iffuso.com/api/services/non-existent-id \
  -H "X-API-Key: e9e4d2b4-8dec-4b23-b15c-149cc182630e"
```

**Expected Response:**
```json
{
  "success": false,
  "data": null,
  "error": "Service not found"
}
```

## Integration Testing

### 18. Create Endpoint that Uses Service

After creating and activating a PostgreSQL service:

1. Create an endpoint that uses the database service
2. Test that the endpoint can access the service via Context
3. Verify that service errors are handled properly

### 19. Service Provider Lifecycle

1. Create service
2. Test connection (should work)
3. Activate service (start actor)
4. Use service from endpoint
5. Deactivate service (stop actor)
6. Try to use service from endpoint (should fail gracefully)
7. Reactivate service
8. Use service again (should work)

## Test Script

Create a comprehensive test script:

```bash
#!/bin/bash

# Service Provider API Test Script
# Usage: ./test_service_providers.sh

API_KEY="e9e4d2b4-8dec-4b23-b15c-149cc182630e"
BASE_URL="https://rust-edge-gateway.iffuso.com"

# Test 1: List services (should be empty initially)
echo "=== Test 1: List Services (Empty) ==="
curl -X GET "$BASE_URL/api/services" \
  -H "X-API-Key: $API_KEY" \
  -s | jq .

# Test 2: Create PostgreSQL Service
echo "\n=== Test 2: Create PostgreSQL Service ==="
POSTGRES_RESPONSE=$(curl -X POST "$BASE_URL/api/services" \
  -H "Content-Type: application/json" \
  -H "X-API-Key: $API_KEY" \
  -d @examples/services/postgres/postgres.json \
  -s)
echo "Response:" | jq .
POSTGRES_ID=$(echo "$POSTGRES_RESPONSE" | jq -r '.data.id')
echo "Created PostgreSQL Service with ID: $POSTGRES_ID"

# Test 3: Create Redis Service
echo "\n=== Test 3: Create Redis Service ==="
REDIS_RESPONSE=$(curl -X POST "$BASE_URL/api/services" \
  -H "Content-Type: application/json" \
  -H "X-API-Key: $API_KEY" \
  -d @examples/services/redis/redis.json \
  -s)
echo "Response:" | jq .
REDIS_ID=$(echo "$REDIS_RESPONSE" | jq -r '.data.id')
echo "Created Redis Service with ID: $REDIS_ID"

# Test 4: List services (should show 2 services)
echo "\n=== Test 4: List Services (Should Show 2) ==="
curl -X GET "$BASE_URL/api/services" \
  -H "X-API-Key: $API_KEY" \
  -s | jq .

# Test 5: Test PostgreSQL Connection
echo "\n=== Test 5: Test PostgreSQL Connection ==="
curl -X POST "$BASE_URL/api/services/$POSTGRES_ID/test" \
  -H "X-API-Key: $API_KEY" \
  -s | jq .

# Test 6: Test Redis Connection
echo "\n=== Test 6: Test Redis Connection ==="
curl -X POST "$BASE_URL/api/services/$REDIS_ID/test" \
  -H "X-API-Key: $API_KEY" \
  -s | jq .

# Test 7: Activate PostgreSQL Service
echo "\n=== Test 7: Activate PostgreSQL Service ==="
curl -X POST "$BASE_URL/api/services/$POSTGRES_ID/activate" \
  -H "X-API-Key: $API_KEY" \
  -s | jq .

# Test 8: Activate Redis Service
echo "\n=== Test 8: Activate Redis Service ==="
curl -X POST "$BASE_URL/api/services/$REDIS_ID/activate" \
  -H "X-API-Key: $API_KEY" \
  -s | jq .

# Test 9: Update PostgreSQL Service
echo "\n=== Test 9: Update PostgreSQL Service ==="
curl -X PUT "$BASE_URL/api/services/$POSTGRES_ID" \
  -H "Content-Type: application/json" \
  -H "X-API-Key: $API_KEY" \
  -d '{"name": "Updated PostgreSQL", "config": {"pool_size": 15}}' \
  -s | jq .

# Test 10: Get PostgreSQL Service Details
echo "\n=== Test 10: Get PostgreSQL Service Details ==="
curl -X GET "$BASE_URL/api/services/$POSTGRES_ID" \
  -H "X-API-Key: $API_KEY" \
  -s | jq .

# Test 11: Deactivate PostgreSQL Service
echo "\n=== Test 11: Deactivate PostgreSQL Service ==="
curl -X POST "$BASE_URL/api/services/$POSTGRES_ID/deactivate" \
  -H "X-API-Key: $API_KEY" \
  -s | jq .

# Test 12: Delete Redis Service
echo "\n=== Test 12: Delete Redis Service ==="
curl -X DELETE "$BASE_URL/api/services/$REDIS_ID" \
  -H "X-API-Key: $API_KEY" \
  -s | jq .

# Test 13: List services (should show 1 service)
echo "\n=== Test 13: List Services (Should Show 1) ==="
curl -X GET "$BASE_URL/api/services" \
  -H "X-API-Key: $API_KEY" \
  -s | jq .

# Test 14: Delete PostgreSQL Service
echo "\n=== Test 14: Delete PostgreSQL Service ==="
curl -X DELETE "$BASE_URL/api/services/$POSTGRES_ID" \
  -H "X-API-Key: $API_KEY" \
  -s | jq .

# Test 15: List services (should be empty again)
echo "\n=== Test 15: List Services (Should Be Empty) ==="
curl -X GET "$BASE_URL/api/services" \
  -H "X-API-Key: $API_KEY" \
  -s | jq .

echo "\n=== All Tests Completed ==="
```

Save this as `test_service_providers.sh` and make it executable:

```bash
chmod +x test_service_providers.sh
```

## Expected Outcomes

1. **Success**: All API endpoints respond correctly with appropriate JSON
2. **Error Handling**: Invalid requests return proper error messages
3. **Service Lifecycle**: Services can be created, activated, used, deactivated, and deleted
4. **Data Integrity**: Service configurations are stored and retrieved correctly

## Troubleshooting

### Connection Issues

- Verify the gateway is running: `curl https://rust-edge-gateway.iffuso.com/health`
- Check API key is correct: `e9e4d2b4-8dec-4b23-b15c-149cc182630e`
- Verify port is correct (typically 9081 for admin API)

### Authentication Issues

- Ensure API key is included in `X-API-Key` header
- Check that the API key has `services:*` permissions

### Service Activation Issues

- Verify service configuration is valid
- Check that backend service (PostgreSQL, Redis, etc.) is accessible
- Review gateway logs for connection errors

## Next Steps

1. ✅ Create comprehensive testing plan
2. ⏳ Execute test script against live environment
3. ⏳ Document results and any issues
4. ⏳ Update API documentation based on findings
5. ⏳ Create example endpoints that use Service Providers