# Backend Deployment Quick Start

## 🚀 Deploy Your SuperNode Backend in 5 Minutes

This guide will help you deploy your Rust-based SuperNode backend to Fly.io with global edge distribution.

## What You'll Get

✅ **Live Backend API** - HTTP REST endpoints for algorithm control  
✅ **Real-time WebSocket** - Stream algorithm events as they happen  
✅ **Global CDN** - Deployed to 30+ regions worldwide  
✅ **Auto-scaling** - Scales from 1 to 10 instances based on load  
✅ **HTTPS/WSS** - Secure connections out of the box  
✅ **Health Monitoring** - Built-in health checks and metrics  

## Prerequisites (2 minutes setup)

1. **Install Fly.io CLI**
   ```bash
   curl -L https://fly.io/install.sh | sh
   ```

2. **Create Free Fly.io Account**
   - Visit: https://fly.io/app/sign-up
   - Free tier includes 3 shared-cpu VMs (enough for testing)

3. **Docker Desktop** (already installed if you have it)
   - Download: https://www.docker.com/products/docker-desktop

## One-Command Deployment

```bash
cd /Users/macbookpri/Downloads/super-nodes
./deploy.sh
```

The script will:
- ✅ Authenticate with Fly.io
- ✅ Create your app configuration
- ✅ Build the Docker image (5-10 min first time)
- ✅ Deploy to global edge network
- ✅ Show you the deployment URL

## Manual Deployment (Alternative)

If you prefer step-by-step control:

### Step 1: Login
```bash
fly auth login
```

### Step 2: Launch App
```bash
fly launch --no-deploy
```

### Step 3: Deploy
```bash
fly deploy
```

That's it! Your backend is live.

## Testing Your Deployment

### 1. Health Check
```bash
curl https://your-app-name.fly.dev/health
```

Expected response:
```json
{
  "status": "healthy",
  "uptime_seconds": 42,
  "node_count": 0,
  "active_connections": 0
}
```

### 2. List Algorithms
```bash
curl https://your-app-name.fly.dev/api/algorithms
```

### 3. Connect via WebSocket
Open browser console and test:
```javascript
const ws = new WebSocket('wss://your-app-name.fly.dev/ws/algorithms');
ws.onmessage = (e) => console.log('Event:', JSON.parse(e.data));
ws.onopen = () => ws.send(JSON.stringify({type: 'run_algorithm', algorithm: 'dijkstra'}));
```

## Connecting Your Frontend

### Option A: Use the Live Dashboard

We've created `dashboard-live.html` with backend support built-in:

```bash
# Backup your current dashboard
cp dashboard.html dashboard-backup.html

# Replace with live version
cp dashboard-live.html dashboard.html

# Deploy to Vercel
vercel --prod
```

Then edit `dashboard.html` line ~275 to update the backend URL:
```javascript
const BACKEND_CONFIG = {
  enabled: true,
  wsUrl: 'wss://your-app-name.fly.dev/ws/algorithms',
  apiUrl: 'https://your-app-name.fly.dev/api',
};
```

### Option B: Hybrid Mode (Recommended)

The live dashboard automatically:
- ✅ Tries to connect to backend first
- ✅ Falls back to local simulation if backend unavailable
- ✅ Shows connection status indicator

No manual switching needed!

## Monitoring & Management

### View Real-time Logs
```bash
fly logs
```

### Check App Status
```bash
fly status
```

### Open in Browser
```bash
fly apps open your-app-name
```

### SSH into Running Instance
```bash
fly ssh console
```

## Scaling

### Increase Instances
```bash
fly scale count 3  # Run 3 instances
```

### Change VM Size
```bash
fly scale vm performance-2x
```

### Add More Regions
```bash
fly regions allow lax ord iad ams
fly deploy
```

## Cost Estimate

**Free Tier:**
- 3 shared-cpu-1x VMs (256MB RAM each)
- Enough for development/testing

**Production:**
- 1 performance-1x VM: ~$5/month
- Bandwidth: ~$1-3/month
- **Total: ~$6-8/month** for most use cases

See full pricing: https://fly.io/docs/about/pricing/

## Troubleshooting

### Build Timeout
**Problem:** Rust compilation times out  
**Solution:** 
```bash
fly deploy --build-timeout 1800  # 30 minutes
```

### Health Check Failing
**Problem:** App returns 503  
**Check logs:**
```bash
fly logs | grep -i error
```

**Verify port binding:**
```bash
fly ssh console-C "netstat -tlnp"
```

### WebSocket Disconnects
**Problem:** Frequent reconnections  
**Solution:** Check region latency and consider deploying closer to users:
```bash
fly regions list
fly regions allow <closer-region>
fly deploy
```

## Updating Your Deployment

After making code changes:

```bash
# Simple redeploy
fly deploy

# Or with no downtime (rolling update)
fly deploy --strategy rolling
```

## Rollback

If something goes wrong:

```bash
# List releases
fly releases list

# Rollback to previous version
fly releases rollback v123
```

## Next Steps

1. ✅ Deploy backend to Fly.io
2. ✅ Test API endpoints work
3. ✅ Update frontend to use live backend
4. ⏭️ Implement actual algorithm execution logic
5. ⏭️ Add authentication if needed
6. ⏭️ Set up custom domain

## Custom Domain (Optional)

```bash
# Add your domain
fly certs add yourdomain.com

# Get DNS records to configure
fly domains list
```

Then update your frontend to point to your custom domain instead of `.fly.dev`.

## Support & Resources

- 📚 [Fly.io Documentation](https://fly.io/docs/)
- 💬 [Community Forum](https://community.fly.io/)
- 📊 [Status Page](https://status.fly.io/)
- 🐛 [GitHub Issues](https://github.com/superproject/super-nodes/issues)

## What's Deployed

Your backend includes:

- **HTTP API Server** (Port 3000)
  - `/health` - Health check endpoint
  - `/api/algorithms` - List available algorithms
  - `/api/algorithms/run` - Execute algorithm
  
- **WebSocket Server** (Port 3000)
  - `/ws/algorithms` - Real-time event streaming
  
- **QUIC/UDP Server** (Port 9000)
  - Main SuperNode server for P2P connections
  
- **Metrics Endpoint** (Port 9090)
  - Prometheus-compatible metrics

---

**Need Help?** Check `DEPLOYMENT.md` for detailed instructions or run `./deploy.sh` for guided deployment.

Happy deploying! 🚀
