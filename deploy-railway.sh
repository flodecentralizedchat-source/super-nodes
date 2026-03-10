#!/bin/bash
# ============================================================
# SUPERNODE - Railway Deployment Script
# No credit card required!
# ============================================================

set -e

GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${BLUE}╔══════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║  SuperNode Railway Deployment              ║${NC}"
echo -e "${BLUE}╚══════════════════════════════════════════════╝${NC}"
echo ""

# Check Railway CLI
if ! command -v railway &> /dev/null; then
    echo -e "${RED}❌ Railway CLI not installed${NC}"
    echo "Run: npm install -g @railway/cli"
   exit 1
fi

echo -e "${GREEN}✅ Railway CLI detected${NC}"

# Login
echo ""
echo -e "${YELLOW}Step 1: Authenticating with Railway...${NC}"
railway whoami || railway login

# Link or create project
echo ""
echo -e "${YELLOW}Step 2: Setting up Railway project...${NC}"
if [ -f "railway.toml" ]; then
    echo "Found railway.toml configuration"
fi

# Link to project (creates new if doesn't exist)
railway link || railway init

# Add Docker buildpack
echo ""
echo -e "${YELLOW}Step 3: Configuring Docker build...${NC}"
railway up --detach

echo ""
echo -e "${GREEN}✅ Deployment started!${NC}"
echo ""
echo -e "${YELLOW}Building your Rust backend (this takes 5-10 minutes first time)...${NC}"
echo "Watch progress at: https://cloud.railway.com"
echo ""
echo "Once complete, your backend will be available at:"
echo "  https://your-project.up.railway.app"
echo ""
echo -e "${YELLOW}Next steps:${NC}"
echo "1. Wait for build to complete (check Railway dashboard)"
echo "2. Get your Railway URL: railway domain"
echo "3. Update dashboard.html with the URL"
echo "4. Deploy frontend: vercel --prod"
echo ""
