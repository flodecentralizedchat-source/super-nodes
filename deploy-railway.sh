#!/bin/bash
# ============================================================
# Railway Deployment Script for SuperNode
# ============================================================

set -e

echo "🚀 Deploying SuperNode to Railway..."
echo ""

# Check if Railway CLI is installed
if ! command -v railway &> /dev/null; then
    echo "❌ Railway CLI not found. Installing..."
    npm install -g @railway/cli
fi

# Check if logged in
echo "🔐 Checking Railway authentication..."
if ! railway whoami &> /dev/null; then
    echo "📡 Logging into Railway..."
   railway login
fi

# Get project info
PROJECT_NAME="super-nodes"
echo ""
echo "📦 Project: $PROJECT_NAME"
echo ""

# Build locally (optional, Railway will build too)
echo "🔨 Building locally (optional check)..."
cargo check --release
echo "✅ Local build successful!"
echo ""

# Deploy to Railway
echo "🚀 Deploying to Railway..."
railway up --detach

echo ""
echo "⏳ Waiting for deployment to complete..."
sleep 10

# Get deployment status
echo ""
echo "📊 Deployment Status:"
railway status

echo ""
echo "✅ Deployment initiated!"
echo ""
echo "📝 Next steps:"
echo "   1. Visit https://railway.app/dashboard/super-nodes"
echo "   2. Check logs: railway logs"
echo "   3. Get your URL: railway domain"
echo "   4. Test health: curl https://your-url.railway.app/health"
echo ""
