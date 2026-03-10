# SuperNode Railway Deployment Guide

## Quick Start 🚀

Your SuperNode backend is configured and ready to deploy to Railway!

### One-Command Deploy

```bash
chmod +x deploy-railway.sh
./deploy-railway.sh
```

## What I've Set Up For You

### ✅ Configuration Files

1. **railway.toml** - Railway project configuration
2. **Dockerfile** - Multi-stage build optimized for production
3. **src/main.rs** - Updated to use Railway environment variables
4. **deploy-railway.sh** - Automated deployment script
5. **test-railway-backend.sh** - Backend verification tests

### ✅ Environment Variables

Your code now reads these Railway environment variables:
- `PORT` - Main server port (default: 9000)
- `API_PORT` - HTTP API port (default: 3000)
- `METRICS_PORT` - Prometheus metrics (default: 9090)

## Step-by-Step Deployment

### Option 1: Using the Script (Recommended)

```bash
# 1. Make scripts executable
chmod +x deploy-railway.sh test-railway-backend.sh

# 2. Deploy
./deploy-railway.sh

# 3. Wait for build (5-10 minutes first time)
# Watch at: https://railway.app/dashboard/super-nodes
```

### Option 2: Manual Deployment

```bash
# 1. Install Railway CLI
npm install -g @railway/cli

# 2. Login
railway login

# 3. Initialize project (if not done)
railway init

# 4. Deploy
railway up --detach
```

## After Deployment

### Get Your Railway URL

```bash
railway domain
```

This will show something like:
```
https://super-nodes-xyz.up.railway.app
```

### Test Your Backend

```bash
# Use the verification script
./test-railway-backend.sh https://super-nodes-xyz.up.railway.app

# Or manually test endpoints:

# 1. Health check
curl https://super-nodes-xyz.up.railway.app/health

# 2. List algorithms
curl https://super-nodes-xyz.up.railway.app/api/algorithms

# 3. Run an algorithm
curl -X POST \
  -H "Content-Type: application/json" \
  -d '{"algorithm":"dijkstra","params":{}}' \
  https://super-nodes-xyz.up.railway.app/api/algorithms/run
```

### Expected Responses

**Health Check:**
```json
{
  "status": "healthy",
  "uptime_seconds": 123,
  "node_count": 0,
  "active_connections": 0
}
```

**Algorithms List:**
```json
[
  {
    "id": "dijkstra",
    "name": "Dijkstra's Shortest Path",
    "description": "Find optimal path between two nodes",
    "complexity": "O((V+E) log V)"
  },
  ...
]
```

**Run Algorithm:**
```json
{
  "success": true,
  "event_id": "uuid-here",
  "message": "Algorithm dijkstra started"
}
```

## WebSocket Connection

For real-time algorithm visualization:

```javascript
// Frontend WebSocket connection
const ws = new WebSocket(
  'wss://super-nodes-xyz.up.railway.app/ws/algorithms'
);

ws.onmessage = (event) => {
  const data = JSON.parse(event.data);
  console.log('Algorithm event:', data);
};
```

## Monitoring & Debugging

### View Logs

```bash
# Real-time logs
railway logs

# Filter logs
railway logs | grep "API"
railway logs | grep "error"
```

### Check Status

```bash
railway status
```

### Open Dashboard

```bash
railway open
```

Or visit: https://railway.app/dashboard/super-nodes

## Updating Your Deployment

After making code changes:

```bash
# Simple update
git add .
git commit -m "Your changes"
git push

# Deploy to Railway
railway up --detach
```

## Frontend Integration

Update your frontend to use the live backend:

### Update dashboard.html or index.html

Replace the simulated backend URL with your Railway URL:

```javascript
// Before (simulated)
const BACKEND_URL = 'http://localhost:3000';

// After (live Railway backend)
const BACKEND_URL = 'https://super-nodes-xyz.up.railway.app';

// WebSocket
const WS_URL = 'wss://super-nodes-xyz.up.railway.app/ws/algorithms';
```

### Deploy Frontend to Vercel

```bash
cd /path/to/frontend
vercel --prod
```

## Troubleshooting

### Build Fails

**Issue:** Rust compilation timeout
```bash
# Railway automatically handles this, but you can check logs
railway logs
```

### Health Check Fails

**Issue:** Backend not starting properly
```bash
# Check if ports are correct
railway logs | grep "API server"

# Should see: "API server starting on http://0.0.0.0:3000"
```

### Port Binding Error

**Issue:** "Address already in use"
- Railway sets the `PORT` environment variable automatically
- Your code now reads from `env::var("PORT")`
- Make sure you deployed after the code update

### WebSocket Disconnects

**Issue:** Frequent disconnections
- Check browser console for errors
- Verify Railway URL is correct (https/wss)
- Check Railway logs for connection issues

## Cost Estimate

Railway free tier includes:
- $5/month credit
- Enough for development/testing
- No credit card required to start

Production estimate (~$5-15/month):
- Always-on service
- 512MB RAM
- Shared CPU

## Next Steps

1. ✅ Deploy to Railway: `./deploy-railway.sh`
2. ✅ Get your URL: `railway domain`
3. ✅ Test endpoints: `./test-railway-backend.sh <your-url>`
4. ⏭️ Update frontend with Railway URL
5. ⏭️ Deploy frontend to Vercel
6. ⏭️ Monitor logs: `railway logs --follow`

## Support

- Railway Docs: https://docs.railway.app
- Railway Discord: https://discord.gg/railway
- Your Railway Dashboard: https://railway.app/dashboard

---

**Happy Deploying! 🚀**

Your backend will be running at: `https://super-nodes.up.railway.app`
