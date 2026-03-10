#!/bin/bash
# ============================================================
# SUPERNODE - Quick Deploy Script
# Automates the deployment process to Fly.io
# ============================================================

set -e  # Exit on error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}╔══════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║   SuperNode Backend Deployment Script       ║${NC}"
echo -e "${BLUE}╚══════════════════════════════════════════════╝${NC}"
echo ""

# Check if Fly.io CLI is installed
if ! command-v fly &> /dev/null; then
    echo -e "${RED}❌ Fly.io CLI not found${NC}"
    echo "Please install it first:"
    echo "  curl -L https://fly.io/install.sh | sh"
    exit 1
fi

echo -e "${GREEN}✅ Fly.io CLI detected${NC}"

# Check if Docker is running
if ! docker info &> /dev/null; then
    echo -e "${YELLOW}⚠️  Docker is not running. Please start Docker Desktop.${NC}"
   read -p "Continue anyway? (y/n) " -n 1-r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

echo -e "${GREEN}✅ Docker is running${NC}"

# Step 1: Login to Fly.io
echo ""
echo -e "${YELLOW}Step 1: Authenticating with Fly.io...${NC}"
fly auth whoami &> /dev/null || {
    echo "Please login to Fly.io:"
    fly auth login
}
echo -e "${GREEN}✅ Authenticated${NC}"

# Step 2: Create or select app
echo ""
echo -e "${YELLOW}Step 2: Setting up Fly.io app...${NC}"
if [ -f "fly.toml" ]; then
    APP_NAME=$(grep "^app = " fly.toml | cut -d'"' -f2)
    echo "Found existing app config: ${BLUE}$APP_NAME${NC}"
   read -p "Use this app? (y/n) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        rm fly.toml
        fly launch --no-deploy
    fi
else
    fly launch --no-deploy
fi

# Get the app name from fly.toml
APP_NAME=$(grep "^app = " fly.toml | cut -d'"' -f2)
echo -e "${GREEN}✅ Using app: ${BLUE}$APP_NAME${NC}"

# Step 3: Set environment variables (optional)
echo ""
echo -e "${YELLOW}Step 3: Configuring environment...${NC}"
echo "Default configuration:"
echo "  - API Port: 3000"
echo "  - Server Port: 9000"
echo "  - Metrics Port: 9090"
echo "  - Log level: info"
read -p "Customize environment? (y/n) " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
   read -p "Enter RUST_LOG level (default: info): " RUST_LOG
    RUST_LOG=${RUST_LOG:-info}
    fly secrets set RUST_LOG="$RUST_LOG"
fi

# Step 4: Build and deploy
echo ""
echo -e "${YELLOW}Step 4: Building Docker image...${NC}"
echo "This may take 5-10 minutes on first build (Rust compilation)"
echo ""

# Show build progress
fly deploy --remote-only --build-only 2>&1 | while read line; do
    echo "$line"
done

echo -e "${GREEN}✅ Build complete${NC}"

# Step 5: Deploy
echo ""
echo -e "${YELLOW}Step 5: Deploying to Fly.io...${NC}"
fly deploy --remote-only

echo -e "${GREEN}✅ Deployment complete!${NC}"

# Step 6: Show status
echo ""
echo -e "${YELLOW}Deployment Summary:${NC}"
fly status

# Get the app URL
APP_URL="https://${APP_NAME}.fly.dev"

echo ""
echo -e "${GREEN}╔══════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║          🎉 Deployment Successful!          ║${NC}"
echo -e "${GREEN}╚══════════════════════════════════════════════╝${NC}"
echo ""
echo -e "${BLUE}Your backend is now available at:${NC}"
echo -e "  ${GREEN}API:${NC}         $APP_URL/api/*"
echo -e "  ${GREEN}Health:${NC}       $APP_URL/health"
echo -e "  ${GREEN}WebSocket:${NC}    wss://$APP_NAME.fly.dev/ws/algorithms"
echo -e "  ${GREEN}Metrics:${NC}      $APP_URL/metrics"
echo ""
echo -e "${YELLOW}Next steps:${NC}"
echo "1. Test the health endpoint:"
echo "   curl $APP_URL/health"
echo ""
echo "2. Update your frontend dashboard.html:"
echo "   - Copy dashboard-live.html to replace dashboard.html"
echo "   - Update BACKEND_CONFIG.wsUrl in dashboard.html"
echo "   - Deploy to Vercel: vercel --prod"
echo ""
echo "3. View logs:"
echo "   fly logs"
echo ""
echo "4. Open in browser:"
echo "   fly apps open $APP_NAME"
echo ""
echo -e "${YELLOW}For detailed instructions, see DEPLOYMENT.md${NC}"
echo ""
