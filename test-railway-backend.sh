#!/bin/bash
# ============================================================
# Railway Backend Verification Script
# Tests all endpoints after deployment
# ============================================================

set -e

GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${BLUE}╔══════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║  SuperNode Backend Verification            ║${NC}"
echo -e "${BLUE}╚══════════════════════════════════════════════╝${NC}"
echo ""

# Get Railway URL
if [ -z "$1" ]; then
    echo -e "${YELLOW}Usage: $0 <your-railway-url>${NC}"
    echo "Example: $0 https://super-nodes-xyz.up.railway.app"
    echo ""
    
    # Try to get from railway CLI
    if command -v railway &> /dev/null; then
        RAILWAY_URL=$(railway domain 2>/dev/null | head -1)
        if [ -n "$RAILWAY_URL" ]; then
            echo -e "${GREEN}✅ Found Railway URL: ${RAILWAY_URL}${NC}"
            BASE_URL="$RAILWAY_URL"
        else
            echo -e "${RED}❌ No Railway URL found. Please provide it as argument.${NC}"
            exit 1
        fi
    else
        exit 1
    fi
else
    BASE_URL="$1"
fi

echo ""
echo -e "${BLUE}Testing backend at: ${BASE_URL}${NC}"
echo ""

# Test 1: Health Check
echo -e "${YELLOW}[1/5] Testing health endpoint...${NC}"
HEALTH_RESPONSE=$(curl -s -w "\n%{http_code}" "${BASE_URL}/health" 2>/dev/null || echo "000")
HTTP_CODE=$(echo "$HEALTH_RESPONSE" | tail -n1)
BODY=$(echo "$HEALTH_RESPONSE" | head -n-1)

if [ "$HTTP_CODE" = "200" ]; then
    echo -e "${GREEN}✅ Health check passed (HTTP $HTTP_CODE)${NC}"
    echo "Response: $BODY" | jq . 2>/dev/null || echo "$BODY"
else
    echo -e "${RED}❌ Health check failed (HTTP $HTTP_CODE)${NC}"
    echo "The backend might not be running yet. Check Railway logs."
fi
echo ""

# Test 2: List Algorithms
echo -e "${YELLOW}[2/5] Testing algorithms list...${NC}"
ALGO_RESPONSE=$(curl -s -w "\n%{http_code}" "${BASE_URL}/api/algorithms" 2>/dev/null || echo "000")
HTTP_CODE=$(echo "$ALGO_RESPONSE" | tail -n1)
BODY=$(echo "$ALGO_RESPONSE" | head -n-1)

if [ "$HTTP_CODE" = "200" ]; then
    echo -e "${GREEN}✅ Algorithms endpoint working${NC}"
    echo "Available algorithms:"
    echo "$BODY" | jq . 2>/dev/null || echo "$BODY"
else
    echo -e "${RED}❌ Algorithms endpoint failed (HTTP $HTTP_CODE)${NC}"
fi
echo ""

# Test 3: Run Algorithm (Dijkstra)
echo -e "${YELLOW}[3/5] Testing Dijkstra algorithm...${NC}"
RUN_RESPONSE=$(curl -s -X POST \
    -H "Content-Type: application/json" \
    -d '{"algorithm":"dijkstra","params":{}}' \
    -w "\n%{http_code}" \
    "${BASE_URL}/api/algorithms/run" 2>/dev/null || echo "000")
HTTP_CODE=$(echo "$RUN_RESPONSE" | tail -n1)
BODY=$(echo "$RUN_RESPONSE" | head -n-1)

if [ "$HTTP_CODE" = "200" ]; then
    echo -e "${GREEN}✅ Algorithm execution started${NC}"
    echo "Response: $BODY" | jq . 2>/dev/null || echo "$BODY"
else
    echo -e "${RED}❌ Algorithm execution failed (HTTP $HTTP_CODE)${NC}"
fi
echo ""

# Test 4: Algorithm Status
echo -e "${YELLOW}[4/5] Testing algorithm status...${NC}"
STATUS_RESPONSE=$(curl -s -w "\n%{http_code}" "${BASE_URL}/api/algorithms/dijkstra/status" 2>/dev/null || echo "000")
HTTP_CODE=$(echo "$STATUS_RESPONSE" | tail -n1)
BODY=$(echo "$STATUS_RESPONSE" | head -n-1)

if [ "$HTTP_CODE" = "200" ]; then
    echo -e "${GREEN}✅ Status endpoint working${NC}"
    echo "Status: $BODY" | jq . 2>/dev/null || echo "$BODY"
else
    echo -e "${RED}❌ Status endpoint failed (HTTP $HTTP_CODE)${NC}"
fi
echo ""

# Test 5: Metrics (optional, may not be exposed)
echo -e "${YELLOW}[5/5] Testing metrics endpoint...${NC}"
METRICS_RESPONSE=$(curl -s -w "\n%{http_code}" "${BASE_URL}/metrics" 2>/dev/null || echo "000")
HTTP_CODE=$(echo "$METRICS_RESPONSE" | tail -n1)

if [ "$HTTP_CODE" = "200" ]; then
    echo -e "${GREEN}✅ Metrics endpoint accessible${NC}"
else
    echo -e "${YELLOW}⚠️  Metrics endpoint not exposed (this is normal)${NC}"
    echo "Metrics are typically only available internally on port 9090"
fi
echo ""

# Summary
echo -e "${BLUE}══════════════════════════════════════════════${NC}"
echo -e "${BLUE}Summary:${NC}"
echo -e "${BLUE}══════════════════════════════════════════════${NC}"
echo ""
echo "Your Railway backend URL: ${BASE_URL}"
echo ""
echo "Test commands:"
echo "  curl ${BASE_URL}/health"
echo "  curl ${BASE_URL}/api/algorithms"
echo "  curl -X POST -H 'Content-Type: application/json' -d '{\"algorithm\":\"dijkstra\"}' ${BASE_URL}/api/algorithms/run"
echo ""
echo "WebSocket URL for frontend:"
echo "  wss://$(echo $BASE_URL | sed 's|https://||')/ws/algorithms"
echo ""
echo -e "${GREEN}✅ Verification complete!${NC}"
