# Icarus MCP Server - Claude Code Web Setup

Complete guide for deploying your Rust MCP server to work with **Claude Code Web** (https://claude.ai/code/session).

## Understanding the Difference

| Feature | Desktop Claude | Claude Code Web |
|---------|---------------|-----------------|
| **Transport** | Stdio (local processes) | **HTTP (network)** |
| **Configuration** | Local JSON file | `claude mcp add` command |
| **Server Location** | Local machine | Network-accessible URL |
| **Best For** | Personal development | Cloud deployment, team sharing |

**Key Insight:** Claude Code Web requires your MCP server to be accessible via HTTP, not stdio.

---

## Quick Start (3 Methods)

### Method 1: Local + Ngrok (Easiest for Testing)
```bash
# 1. Run the script
run-with-ngrok.bat

# 2. Copy the ngrok URL (e.g., https://abcd-1234.ngrok-free.app)

# 3. Add to Claude Code
claude mcp add --transport http icarus https://your-ngrok-url.ngrok-free.app
```

### Method 2: Docker + Cloud (Production)
```bash
# 1. Build and deploy Docker container
docker-compose up -d

# 2. Deploy to cloud (Railway, Fly.io, etc.)

# 3. Add to Claude Code
claude mcp add --transport http icarus https://your-server.com
```

### Method 3: Direct Binary (Advanced)
```bash
# 1. Build HTTP server
cargo build --release --bin icarus-mcp-http

# 2. Run on cloud server
./target/release/icarus-mcp-http 3000

# 3. Add to Claude Code
claude mcp add --transport http icarus https://your-server-ip:3000
```

---

## Detailed Setup Instructions

### Prerequisites

1. **Rust Toolchain**
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **Claude CLI**
   ```bash
   # Install Claude CLI (check Claude Code docs for latest)
   npm install -g @anthropic-ai/claude-cli
   # or
   brew install claude-cli
   ```

3. **For Ngrok Method:** [Ngrok Account](https://ngrok.com/download)
4. **For Docker Method:** [Docker Desktop](https://www.docker.com/products/docker-desktop/)

---

## Method 1: Local Development with Ngrok (Recommended for Testing)

### What is Ngrok?
Ngrok creates a secure tunnel from a public URL to your local server. Perfect for testing without deploying to cloud.

### Setup Steps

#### Step 1: Install and Configure Ngrok

```bash
# Download from https://ngrok.com/download

# Authenticate (sign up for free)
ngrok authtoken YOUR_AUTH_TOKEN
```

#### Step 2: Build the HTTP Server

```bash
cd icarus-core
cargo build --release --bin icarus-mcp-http
```

#### Step 3: Run with Ngrok (Automated)

**Windows:**
```cmd
run-with-ngrok.bat
```

**Manual (All Platforms):**
```bash
# Terminal 1: Start MCP HTTP server
./target/release/icarus-mcp-http 3000

# Terminal 2: Start ngrok tunnel
ngrok http 3000
```

#### Step 4: Get Ngrok URL

Look for the line in ngrok output:
```
Forwarding    https://abcd-1234-xyz.ngrok-free.app -> http://localhost:3000
```

Copy the `https://abcd-1234-xyz.ngrok-free.app` URL.

#### Step 5: Add to Claude Code

```bash
claude mcp add --transport http icarus https://abcd-1234-xyz.ngrok-free.app
```

#### Step 6: Verify in Claude Code

Open https://claude.ai/code/session and check:
```bash
/mcp
```

You should see "icarus" listed as connected.

### Ngrok Tips

- **Free Plan:** URL changes each restart (use paid plan for static URLs)
- **Dashboard:** http://localhost:4040 to see requests
- **Static URL:** `ngrok http 3000 --domain=your-static-domain.ngrok.app`

---

## Method 2: Docker Deployment (Production)

### Why Docker?
- Consistent environment
- Easy deployment to cloud platforms
- Portable across systems

### Local Docker Setup

#### Step 1: Build Docker Image

```bash
cd icarus-core

# Build image
docker build -t icarus-mcp-http:latest .

# Or use docker-compose
docker-compose build
```

#### Step 2: Run Locally

```bash
# Using docker-compose (recommended)
docker-compose up -d

# Or using docker directly
docker run -d \
  --name icarus-mcp \
  -p 3000:3000 \
  --restart unless-stopped \
  icarus-mcp-http:latest
```

#### Step 3: Test Locally

```bash
# Health check
curl http://localhost:3000/health

# Should return: {"status":"ok","service":"icarus-mcp"}
```

### Deploy to Cloud Platforms

#### Option A: Railway.app (Easiest)

1. **Install Railway CLI:**
   ```bash
   npm install -g @railway/cli
   ```

2. **Login:**
   ```bash
   railway login
   ```

3. **Deploy:**
   ```bash
   railway up
   ```

4. **Get URL:**
   ```bash
   railway domain
   # Returns: https://your-app.railway.app
   ```

5. **Add to Claude Code:**
   ```bash
   claude mcp add --transport http icarus https://your-app.railway.app
   ```

#### Option B: Fly.io

1. **Install Fly CLI:**
   ```bash
   curl -L https://fly.io/install.sh | sh
   ```

2. **Create `fly.toml`:**
   ```toml
   app = "icarus-mcp"
   primary_region = "sjc"

   [build]
     dockerfile = "Dockerfile"

   [[services]]
     internal_port = 3000
     protocol = "tcp"

     [[services.ports]]
       port = 80
       handlers = ["http"]

     [[services.ports]]
       port = 443
       handlers = ["tls", "http"]
   ```

3. **Deploy:**
   ```bash
   fly launch
   fly deploy
   ```

4. **Get URL:**
   ```bash
   fly status
   # Returns: https://icarus-mcp.fly.dev
   ```

5. **Add to Claude Code:**
   ```bash
   claude mcp add --transport http icarus https://icarus-mcp.fly.dev
   ```

#### Option C: DigitalOcean App Platform

1. **Login to DigitalOcean**
2. **Create New App**
3. **Select "Docker Hub" or "GitHub"**
4. **Configure:**
   - Port: 3000
   - Health Check: `/health`
5. **Deploy**
6. **Copy URL:** `https://your-app.ondigitalocean.app`
7. **Add to Claude Code:**
   ```bash
   claude mcp add --transport http icarus https://your-app.ondigitalocean.app
   ```

#### Option D: AWS/Azure/GCP

See detailed cloud deployment guides in `CLOUD_DEPLOYMENT.md`.

---

## Method 3: Direct Binary on Cloud Server

### Setup on Cloud VM (Ubuntu/Debian)

#### Step 1: Provision Server

- Minimum: 1 vCPU, 1GB RAM
- Open port: 3000 (or your chosen port)

#### Step 2: Install Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

#### Step 3: Clone and Build

```bash
git clone https://github.com/yourusername/icarus-core.git
cd icarus-core
cargo build --release --bin icarus-mcp-http
```

#### Step 4: Run as Service (Systemd)

Create `/etc/systemd/system/icarus-mcp.service`:

```ini
[Unit]
Description=Icarus MCP HTTP Server
After=network.target

[Service]
Type=simple
User=ubuntu
WorkingDirectory=/home/ubuntu/icarus-core
ExecStart=/home/ubuntu/icarus-core/target/release/icarus-mcp-http 3000
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
```

Enable and start:

```bash
sudo systemctl enable icarus-mcp
sudo systemctl start icarus-mcp
sudo systemctl status icarus-mcp
```

#### Step 5: Setup Nginx Reverse Proxy (Optional)

```nginx
server {
    listen 80;
    server_name your-domain.com;

    location / {
        proxy_pass http://localhost:3000;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_cache_bypass $http_upgrade;
    }
}
```

#### Step 6: Add to Claude Code

```bash
claude mcp add --transport http icarus https://your-server-ip:3000
# or with domain
claude mcp add --transport http icarus https://your-domain.com
```

---

## Adding to Claude Code

### Basic Command

```bash
claude mcp add --transport http <name> <url>
```

### With Authentication (if needed)

```bash
claude mcp add --transport http icarus https://your-server.com \
  --header "Authorization: Bearer your-token-here"
```

### Configuration Scopes

```bash
# Local (just for this project)
claude mcp add --transport http icarus https://your-server.com

# Project (shared with team via .mcp.json)
claude mcp add --transport http icarus --scope project https://your-server.com

# User (available across all projects)
claude mcp add --transport http icarus --scope user https://your-server.com
```

### Verify Installation

```bash
# List all MCP servers
claude mcp list

# Check specific server
claude mcp get icarus

# In Claude Code session
/mcp
```

---

## Testing Your Deployment

### 1. Health Check

```bash
curl https://your-server-url/health
# Expected: {"status":"ok","service":"icarus-mcp"}
```

### 2. MCP Request

```bash
curl -X POST https://your-server-url/mcp \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}'
```

### 3. Claude Code Test

In Claude Code:
```
/mcp
```

Should show:
```
Connected MCP Servers:
- icarus (http://your-server-url)
```

Try using an Icarus tool:
```
Use the icarus_query_status tool
```

---

## Troubleshooting

### Server Not Starting

```bash
# Check logs
docker logs icarus-mcp-server

# Or for systemd
sudo journalctl -u icarus-mcp -f
```

### Connection Refused

- Verify server is running: `curl http://localhost:3000/health`
- Check firewall rules
- Ensure port is open

### Claude Code Can't Connect

```bash
# Remove and re-add server
claude mcp remove icarus
claude mcp add --transport http icarus https://your-server-url

# Check server status
claude mcp get icarus
```

### Ngrok URL Changes

- Use paid ngrok plan for static URLs
- Or redeploy to cloud for permanent URLs

### Build Errors

```bash
# Clean build
cargo clean
cargo build --release --bin icarus-mcp-http

# Check dependencies
cargo update
```

---

## Security Considerations

### For Production Deployments

1. **Use HTTPS:** Always use HTTPS in production
2. **Authentication:** Add API key authentication
3. **Rate Limiting:** Implement rate limiting
4. **CORS:** Configure CORS headers properly
5. **Firewall:** Restrict access to known IPs
6. **Monitoring:** Set up logging and monitoring

### Add Authentication

Modify `icarus-mcp-http.rs`:

```rust
// Check for API key
if let Some(auth) = req.headers().get("Authorization") {
    if auth != "Bearer YOUR_SECRET_KEY" {
        return unauthorized_response();
    }
}
```

Then add to Claude Code:

```bash
claude mcp add --transport http icarus https://your-server.com \
  --header "Authorization: Bearer YOUR_SECRET_KEY"
```

---

## Cost Comparison

| Method | Cost | Pros | Cons |
|--------|------|------|------|
| **Ngrok Free** | $0 | Easy testing | URL changes, rate limits |
| **Ngrok Pro** | $8/mo | Static URL | Still tunneling overhead |
| **Railway** | ~$5-20/mo | Easy deploy | Limited free tier |
| **Fly.io** | $0-5/mo | Generous free tier | Learning curve |
| **DigitalOcean** | $6-12/mo | Predictable cost | Manual setup |
| **Cloud VM** | $3-10/mo | Full control | More maintenance |

---

## Recommended Setup

### For Development/Testing
→ **Ngrok** (run-with-ngrok.bat)

### For Personal Use
→ **Fly.io** or **Railway** (free tier sufficient)

### For Team/Production
→ **Cloud VM** with Nginx + SSL

### For Enterprise
→ **Kubernetes** deployment (see enterprise docs)

---

## Example Workflow

### Day 1: Local Testing
```bash
# Start with ngrok
run-with-ngrok.bat

# Add to Claude Code
claude mcp add --transport http icarus https://abc-123.ngrok-free.app

# Test in Claude Code
/mcp
```

### Day 2: Deploy to Cloud
```bash
# Build Docker image
docker-compose build

# Deploy to Railway
railway up

# Update Claude Code
claude mcp remove icarus
claude mcp add --transport http icarus https://icarus-mcp.railway.app
```

### Day 3: Share with Team
```bash
# Add to project scope
claude mcp add --transport http icarus --scope project https://icarus-mcp.railway.app

# Team members can now use it automatically
```

---

## Next Steps

1. ✅ Choose deployment method
2. ✅ Deploy your server
3. ✅ Add to Claude Code
4. ✅ Test connection
5. ✅ Start using Icarus tools in Claude Code web!

---

## Quick Commands Reference

```bash
# Build HTTP server
cargo build --release --bin icarus-mcp-http

# Run locally
./target/release/icarus-mcp-http 3000

# Docker build
docker-compose up -d

# Add to Claude Code
claude mcp add --transport http icarus URL

# List servers
claude mcp list

# Check status in Claude Code
/mcp

# Remove server
claude mcp remove icarus
```

---

## Support & Resources

- **Claude Code Docs:** https://code.claude.com/docs
- **MCP Protocol:** https://modelcontextprotocol.io/
- **Ngrok Docs:** https://ngrok.com/docs
- **Railway Docs:** https://docs.railway.app/
- **Fly.io Docs:** https://fly.io/docs/

---

**You're now ready to use your Rust MCP server with Claude Code web! 🚀**
