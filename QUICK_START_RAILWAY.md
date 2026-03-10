# SuperNode Railway - Quick Reference Card

## 🚀 Deploy in 3 Commands

```bash
chmod +x deploy-railway.sh
./deploy-railway.sh
railway domain
```

## ✅ Your Backend URL

Once deployed, your backend will be at:
```
https://super-nodes.up.railway.app
```

Replace with actual URL from `railway domain`

## 🧪 Test Endpoints

```bash
# Health check
curl https://YOUR-URL.railway.app/health

# List algorithms
curl https://YOUR-URL.railway.app/api/algorithms

# Run algorithm
curl -X POST \
  -H "Content-Type: application/json" \
  -d '{"algorithm":"dijkstra"}' \
  https://YOUR-URL.railway.app/api/algorithms/run

# WebSocket (in browser console)
const ws = new WebSocket('wss://YOUR-URL.railway.app/ws/algorithms');
ws.onmessage = (e) => console.log(JSON.parse(e.data));
```

## 📊 Monitoring

```bash
# View logs
railway logs

# Real-time logs
railway logs --follow

# Check status
railway status

# Open dashboard
railway open
```

## 🔧 Environment Variables

Railway automatically sets these:
- `PORT` - Main server (default: 9000)
- `API_PORT` - HTTP API (default: 3000)
- `METRICS_PORT` - Prometheus (default: 9090)

No manual configuration needed!

## 📝 Files Created

1. `deploy-railway.sh` - Deployment script
2. `test-railway-backend.sh` - Verification tests
3. `RAILWAY_DEPLOYMENT.md` - Full guide
4. `src/main.rs` - Updated for Railway env vars
5. `Dockerfile` - Updated health check

## 🎯 Next Steps

1. Run: `./deploy-railway.sh`
2. Wait for build (5-10 min first time)
3. Get URL: `railway domain`
4. Test: `./test-railway-backend.sh <URL>`
5. Update frontend with Railway URL
6. Deploy frontend to Vercel

## 💰 Cost

- Free tier: $5 credit (no CC required)
- Production: ~$5-15/month
- Pay-as-you-go

## 🆘 Help

- Dashboard: https://railway.app/dashboard/super-nodes
- Docs: https://docs.railway.app
- Discord: https://discord.gg/railway

---

**Your backend is ready! 🎉**
