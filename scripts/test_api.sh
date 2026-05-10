#!/bin/bash
# API smoke test for axon
BASE="http://localhost:8080"

echo "=== Healthz ==="
curl -s "$BASE/healthz"
echo ""

echo "=== Unauthorized (expect 401) ==="
curl -s "$BASE/"
echo ""

echo "=== Authorized ==="
curl -s -H "Authorization: Bearer demo-token" "$BASE/"
echo ""
