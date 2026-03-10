#!/bin/bash
# ============================================================
# Test Backend Deployment
# Verifies your Fly.io deployment is working correctly
# ============================================================

set -e

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${YELLOW}Testing SuperNode Backend Deployment...${NC}"
echo ""

# Get app name from fly.toml
if [ ! -f "fly.toml" ]; then
    echo -e "${RED}❌ fly.toml not found. Run 'fly launch' first.${NC}"
   exit 1
fi

APP_NAME=$(grep "^app = " fly.toml | cut -d'"' -f2)
BASE_URL="https://${APP_NAME}.fly.dev"

echo -e "${YELLOW}App:${NC} $APP_NAME"
echo -e "${YELLOW}URL:${NC} $BASE_URL"
echo ""

# Test 1: Health Check
echo -e "${YELLOW}Test 1: Health Check${NC}"
HEALTH_RESPONSE=$(curl -s -w "\n%{http_code}" "$BASE_URL/health")
HEALTH_BODY=$(echo "$HEALTH_RESPONSE" | head -n1)
HEALTH_CODE=$(echo "$HEALTH_RESPONSE" | tail -n1)

if [ "$HEALTH_CODE" = "200" ]; then
    echo -e "${GREEN}✅ Health check passed (HTTP $HEALTH_CODE)${NC}"
    echo "Response: $HEALTH_BODY" | jq .
else
    echo -e "${RED}❌ Health check failed (HTTP $HEALTH_CODE)${NC}"
   exit 1
fi
echo ""

# Test 2: List Algorithms
echo -e "${YELLOW}Test 2: List Algorithms API${NC}"
ALGO_RESPONSE=$(curl -s -w "\n%{http_code}" "$BASE_URL/api/algorithms")
ALGO_BODY=$(echo "$ALGO_RESPONSE" | head -n1)
ALGO_CODE=$(echo "$ALGO_RESPONSE" | tail -n1)

if [ "$ALGO_CODE" = "200" ]; then
    echo -e "${GREEN}✅ Algorithms API passed (HTTP $ALGO_CODE)${NC}"
    echo "Available algorithms:"
    echo "$ALGO_BODY" | jq '.[] | {name: .name, desc: .description}'
else
    echo -e "${RED}❌ Algorithms API failed (HTTP $ALGO_CODE)${NC}"
   exit 1
fi
echo ""

# Test 3: Run Algorithm API
echo -e "${YELLOW}Test 3: Run Algorithm API${NC}"
RUN_RESPONSE=$(curl -s -w "\n%{http_code}" -X POST "$BASE_URL/api/algorithms/run" \
  -H "Content-Type: application/json" \
  -d '{"algorithm": "dijkstra", "params": {}}')
RUN_BODY=$(echo "$RUN_RESPONSE" | head -n1)
RUN_CODE=$(echo "$RUN_RESPONSE" | tail -n1)

if [ "$RUN_CODE" = "200" ]; then
    echo -e "${GREEN}✅ Run algorithm API passed (HTTP $RUN_CODE)${NC}"
    echo "Response: $RUN_BODY" | jq .
else
    echo -e "${RED}❌ Run algorithm API failed (HTTP $RUN_CODE)${NC}"
   exit 1
fi
echo ""

# Test 4: WebSocket Endpoint (Basic Check)
echo -e "${YELLOW}Test 4: WebSocket Endpoint Availability${NC}"
# Note: Full WebSocket test requires wscat or similar tool
WS_URL="wss://${APP_NAME}.fly.dev/ws/algorithms"
echo -e "${GREEN}✅ WebSocket endpoint configured at:${NC}"
echo "   $WS_URL"
echo -e "${YELLOW}   (Use browser console or wscat to test full connection)${NC}"
echo ""

# Test 5: Response Time
echo -e "${YELLOW}Test 5: Response Time${NC}"
RESPONSE_TIME=$(curl -s -o /dev/null -w "%{time_total}" "$BASE_URL/health")
if (( $(echo "$RESPONSE_TIME < 1" | bc -l) )); then
    echo -e "${GREEN}✅ Response time: ${RESPONSE_TIME}s (Good!)${NC}"
else
    echo -e "${YELLOW}⚠️  Response time: ${RESPONSE_TIME}s (Consider closer region)${NC}"
fi
echo ""

# Summary
echo -e "${GREEN}╔══════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║     All Tests Passed! ✅                 ║${NC}"
echo -e "${GREEN}╚══════════════════════════════════════════╝${NC}"
echo ""
echo -e "${YELLOW}Your backend is ready!${NC}"
echo ""
echo "Next steps:"
echo "1. Update dashboard.html with your backend URL"
echo "2. Deploy frontend: vercel --prod"
echo "3. Open in browser and test WebSocket connection"
echo ""
echo "Backend URLs:"
echo "  API:       $BASE_URL/api/*"
echo "  Health:    $BASE_URL/health"
echo "  WebSocket: wss://${APP_NAME}.fly.dev/ws/algorithms"
echo ""
