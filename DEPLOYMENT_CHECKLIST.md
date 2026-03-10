# 🚀 Backend Deployment Checklist

Use this step-by-step checklist to deploy your SuperNode backend.

---

## ✅ Pre-Deployment Checklist

### 1. Install Required Tools
```bash
# Fly.io CLI
curl -L https://fly.io/install.sh | sh

# Verify installation
fly version
# Should show: fly v0.x.x

# Docker (optional, for local testing)
docker --version
```

**Status:** [ ] Done

---

### 2. Create Fly.io Account
- Visit: https://fly.io/app/sign-up
- Complete registration
- Verify email

**Status:** [ ] Done

---

### 3. Review Configuration Files

Check these files exist and are correct:
- [ ] `fly.toml` - Fly.io deployment config
- [ ] `Dockerfile` - Container build instructions
- [ ] `.dockerignore` - Files to exclude from image
- [ ] `src/api.rs` - API server implementation
- [ ] `Cargo.toml` - Rust dependencies (includes axum)

**Status:** [ ] All files reviewed

---

## 🎯 Deployment Steps

### Step 1: Authenticate with Fly.io
```bash
fly auth login
```

This will open a browser window for authentication.

**Status:** [ ] Authenticated

---

### Step 2: Initialize App
```bash
cd /Users/macbookpri/Downloads/super-nodes
fly launch --no-deploy
```

When prompted:
- **App name**: Keep default or customize (e.g., "super-nodes")
- **Organization**: Select your org
- **Region**: Choose closest to you (e.g., "sjc" for San Jose)
- **Add PostgreSQL?**: No
- **Add Redis?**: No
- **Deploy now?**: No (we'll deploy manually)

**Status:** [ ] App initialized

---

### Step 3: Verify Configuration

Check `fly.toml` was created correctly:
```bash
cat fly.toml
```

Should include:
- `app = "super-nodes"` (or your chosen name)
- `[build]` section with dockerfile
- `[http_service]` on port 3000
- `[[services]]` on port 9000

**Status:** [ ] Configuration verified

---

### Step 4: Deploy Backend

#### Option A: Automated (Recommended)
```bash
./deploy.sh
```

Follow the prompts. This handles everything automatically.

**Status:** [ ] Deployed via script

#### Option B: Manual
```bash
# Build and deploy
fly deploy

# Watch the build progress...
# Takes 5-10 minutes first time (Rust compilation)
```

**Status:** [ ] Deployed manually

---

### Step 5: Wait for Deployment

Watch for these messages:
```
✅ Build complete
✅ Deployment complete!
```

Then verify status:
```bash
fly status
```

Should show:
```
App: super-nodes
Status: running
Version: v123
```

**Status:** [ ] Deployment successful

---

## 🧪 Post-Deployment Verification

### Test 1: Health Check
```bash
# Replace with your app name
curl https://super-nodes.fly.dev/health
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

**Status:** [ ] Health check works

---

### Test 2: Algorithms API
```bash
curl https://super-nodes.fly.dev/api/algorithms
```

Expected: List of 5 algorithms (Dijkstra, A*, Gossip, MST, BFS)

**Status:** [ ] Algorithms listed

---

### Test 3: Run Algorithm
```bash
curl -X POST https://super-nodes.fly.dev/api/algorithms/run \
  -H "Content-Type: application/json" \
  -d '{"algorithm": "dijkstra"}'
```

Expected: Success response with event_id

**Status:** [ ] Algorithm execution works

---

### Test 4: Automated Test Script
```bash
./test-backend.sh
```

Should show all tests passing ✅

**Status:** [ ] All tests passed

---

## 🌐 Frontend Integration

### Update Dashboard (Option A - Live Backend)

1. Copy live dashboard:
```bash
cp dashboard-live.html dashboard.html
```

2. Edit `dashboard.html` line ~275:
```javascript
const BACKEND_CONFIG = {
  enabled: true,
  wsUrl: 'wss://super-nodes.fly.dev/ws/algorithms', // Update with your app name
  apiUrl: 'https://super-nodes.fly.dev/api',        // Update with your app name
};
```

3. Deploy to Vercel:
```bash
vercel --prod
```

**Status:** [ ] Frontend updated

---

### Keep Simulation Mode (Option B - No Backend)

If you don't want to connect to backend yet:
- Keep current `dashboard.html` as-is
- It will work in simulation mode
- Shows "Backend Offline" indicator
- Still fully functional with local JS simulations

**Status:** [ ] Keeping simulation mode for now

---

## 🔍 Monitoring Setup

### View Logs
```bash
fly logs
```

See real-time logs from your backend.

**Status:** [ ] Can view logs

---

### Check Metrics
```bash
fly ssh console-C "curl http://localhost:9090/metrics"
```

Shows Prometheus-format metrics.

**Status:** [ ] Metrics accessible

---

### SSH Access
```bash
fly ssh console
```

Get shell access to running instance.

**Status:** [ ] SSH works

---

## 📊 Final Verification

Open your Vercel app in browser:
```
https://super-nodes.vercel.app/
```

Check for:
- [ ] Backend status indicator shows "🟢 Backend Connected"
- [ ] Click "Run Dijkstra" button
- [ ] See algorithm execute (either via backend or simulation)
- [ ] Check activity log shows events
- [ ] Auto-rotation cycles through algorithms

---

## 💰 Cost Check

Verify you're within budget:
```bash
fly apps open super-nodes
```

Check:
- Number of VMs running
- VM size (shared-cpu-1x vs performance-1x)
- Bandwidth usage

Free tier should be sufficient for testing!

**Status:** [ ] Costs verified

---

## 🎉 Success Criteria

Your deployment is complete when ALL boxes are checked:

- ✅ Backend deployed to Fly.io
- ✅ Health endpoint responds
- ✅ API endpoints work
- ✅ WebSocket accepts connections
- ✅ Frontend can connect (or uses simulation)
- ✅ Logs are accessible
- ✅ You can monitor the app
- ✅ Costs are acceptable

---

## 🆘 If Something Goes Wrong

### Build Fails
```bash
# Increase timeout
fly deploy --build-timeout 1800
```

### Health Check Fails
```bash
# Check logs for errors
fly logs | grep -i error
```

### Can't Connect
```bash
# Verify app is running
fly status

# Restart if needed
fly restart
```

### Need to Rollback
```bash
# List releases
fly releases list

# Rollback
fly releases rollback v123
```

---

## 📞 Getting Help

If stuck:
1. Check logs: `fly logs`
2. Review docs: `BACKEND_README.md`, `DEPLOYMENT.md`
3. Fly.io community: https://community.fly.io/
4. Check status: https://status.fly.io/

---

## ✅ What's Next?

After successful deployment:

1. **Implement Real Algorithms**
   - Update `src/api.rs` with actual algorithm logic
   - Use existing `graph.rs`, `node.rs` modules

2. **Add Persistence** (if needed)
   - Enable volume mounts in `fly.toml`
   - Store Raft state on disk

3. **Custom Domain** (optional)
   ```bash
   fly certs add yourdomain.com
   ```

4. **CI/CD** (recommended)
   - Set up GitHub Actions
   - Auto-deploy on push to main

5. **Monitoring Dashboard**
   - Set up Grafana
   - Create alerting rules

---

**Congratulations!** Your SuperNode backend is live and ready! 🚀

Total deployment time: ~10-15 minutes (first time)
Subsequent deployments: ~2-3 minutes
