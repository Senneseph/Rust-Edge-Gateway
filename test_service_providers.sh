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
