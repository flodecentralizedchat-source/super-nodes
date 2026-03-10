# SuperNode Backend Deployment Guide

## Overview

This guide walks you through deploying the SuperNode backend to Fly.io with global edge distribution.

## Prerequisites

- Docker installed and running
- Fly.io account (free tier available)
- Fly.io CLI installed: `curl -L https://fly.io/install.sh | sh`
- Rust toolchain (for local testing)

## Quick Start

### 1. Login to Fly.io

```bash
fly auth login
```

### 2. Initialize Your App

```bash
cd /path/to/super-nodes
fly launch --no-deploy
```

This will:
- Create a new Fly.io app named "super-nodes" (or your chosen name)
- Generate an organization and app name
- NOT deploy yet (we'll do that manually)

### 3. Configure Environment Variables (Optional)

The default configuration in `fly.toml` includes:
- `PORT=9000` - Main QUIC/WebSocket server
- `API_PORT=3000` - HTTP API server
- `METRICS_PORT=9090` - Prometheus metrics
- `RUST_LOG=info` - Log level

To customize:
```bash
fly secrets set RUST_LOG=debug
```

### 4. Deploy to Fly.io

```bash
fly deploy
```

This will:
- Build the Docker image (multi-stage build, optimized for size)
- Push to Fly.io registry
- Deploy to the primary region (sjc - San Jose)

**First deployment takes ~5-10 minutes** (compiling Rust in release mode).

### 5. Access Your Deployment

Once deployed, your backend will be available at:
- **API**: `https://super-nodes.fly.dev/api/*`
- **WebSocket**: `wss://super-nodes.fly.dev/ws/algorithms`
- **Health Check**: `https://super-nodes.fly.dev/health`
- **Metrics**: `http://super-nodes.internal:9090/metrics` (internal network)

Check your app URL:
```bash
fly status
```

## Testing the Backend

### Health Check

```bash
curl https://your-app-name.fly.dev/health
```

Expected response:
```json
{
  "status": "healthy",
  "uptime_seconds": 123,
  "node_count": 0,
  "active_connections": 0
}
```

### List Available Algorithms

```bash
curl https://your-app-name.fly.dev/api/algorithms
```

### Run an Algorithm via API

```bash
curl -X POST https://your-app-name.fly.dev/api/algorithms/run \
  -H "Content-Type: application/json" \
  -d '{
    "algorithm": "dijkstra",
    "params": {}
  }'
```

### Connect via WebSocket

Use a WebSocket client to connect:
```javascript
const ws = new WebSocket('wss://your-app-name.fly.dev/ws/algorithms');

ws.onmessage = (event) => {
  const data = JSON.parse(event.data);
  console.log('Algorithm event:', data);
};
```

## Frontend Integration

Update your Vercel frontend to connect to the live backend:

### 1. Update dashboard.html

Replace simulated data with real WebSocket connection:

```javascript
// In dashboard.html, update the useEffect hook
useEffect(() => {
  // Connect to live backend
  const BACKEND_URL = 'wss://super-nodes.fly.dev/ws/algorithms';
  const ws = new WebSocket(BACKEND_URL);
  
  ws.onopen = () => {
    addLog('Connected to SuperNode backend');
  };
  
  ws.onmessage = (event) => {
    const eventData = JSON.parse(event.data);
    handleAlgorithmEvent(eventData);
  };
  
  ws.onerror = (error) => {
    addLog('WebSocket error: ' + error.message);
  };
  
  return () => {
    if (ws) ws.close();
  };
}, []);
```

### 2. Deploy Updated Frontend to Vercel

```bash
cd /path/to/super-nodes
vercel --prod
```

## Monitoring & Debugging

### View Logs

```bash
# Real-time logs
fly logs

# Logs from specific region
fly logs --region sjc

# Search logs
fly logs | grep "error"
```

### Access Metrics

Prometheus metrics are available at port 9090:

```bash
# From local machine (if you have SSH access)
fly ssh console -C "curl http://localhost:9090/metrics"
```

### SSH into Running Instance

```bash
# Get shell access
fly ssh console

# Check running processes
fly ssh console -C "ps aux"

# Check environment variables
fly ssh console -C "env"
```

## Scaling

### Manual Scaling

```bash
# Increase to 3 instances
fly scale count 3

# Change VM size
fly scale vm performance-2x
```

### Auto-scaling

Already configured in `fly.toml`:
- Min: 1 machine
- Max: 10 machines
- Based on connection load

## Cost Estimation

**Free Tier:**
- Up to 3 shared-cpu-1x VMs
- 256MB memory each
- Enough for development/testing

**Production (~$5-20/month):**
- 1-3 performance-1x VMs: ~$5-15/month
- Bandwidth: ~$1-5/month (depends on usage)

Check pricing: https://fly.io/docs/about/pricing/

## Troubleshooting

### Build Fails

**Issue:** Rust compilation timeout
**Solution:** Increase build timeout or use cached images

```bash
fly deploy --build-timeout 1800  # 30 minutes
```

### Health Checks Failing

**Issue:** App starts but health check returns 503
**Solution:** Check logs and verify API server is binding to correct port

```bash
fly logs | grep "API"
```

### WebSocket Connection Drops

**Issue:** Frequent disconnections
**Solution:** Increase idle timeout in fly.toml

```toml
[http_service]
  [http_service.concurrency]
    type = "requests"
    hard_limit = 1000
```

## Advanced Configuration

### Custom Domain

```bash
# Add custom domain
fly certs add yourdomain.com

# Update DNS records
fly domains list
```

### Persistent Storage

If you need disk persistence (for Raft storage):

1. Create volume:
```bash
fly volumes create supernode_data --region sjc --size 10
```

2. Uncomment mounts section in `fly.toml`

3. Redeploy:
```bash
fly deploy
```

### Multi-Region Deployment

Deploy to multiple regions for lower latency:

```bash
fly regions allow lax ord iad ams nrt
fly deploy
```

## CI/CD Integration

### GitHub Actions Example

Create `.github/workflows/deploy.yml`:

```yaml
name: Deploy to Fly.io

on:
  push:
    branches: [ main ]

jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Install Fly.io CLI
        run: curl -L https://fly.io/install.sh | sh
      
      - name: Login to Fly.io
        run: fly auth login
        env:
          FLY_API_TOKEN: ${{ secrets.FLY_API_TOKEN }}
      
      - name: Deploy
        run: fly deploy --remote-only
```

Store your Fly.io API token in GitHub Secrets as `FLY_API_TOKEN`.

## Rollback

If something goes wrong:

```bash
# List releases
fly releases list

# Rollback to previous version
fly releases rollback v123
```

## Security Best Practices

1. **Use secrets for sensitive config:**
   ```bash
   fly secrets set DATABASE_URL="postgres://..."
   ```

2. **Enable automatic HTTPS** (already configured)

3. **Restrict regions** if you only need specific geographic areas

4. **Monitor resource usage:**
   ```bash
   fly apps open super-nodes
   ```

## Next Steps

1. ✅ Deploy backend to Fly.io
2. ✅ Test API endpoints
3. ✅ Update frontend to use live WebSocket
4. ⏭️ Implement actual algorithm execution
5. ⏭️ Add authentication/authorization
6. ⏭️ Set up monitoring dashboards

## Support

- Documentation: https://fly.io/docs/
- Community Forum: https://community.fly.io/
- Status Page: https://status.fly.io/

---

**Happy Deploying! 🚀**
