# Current Context

## Task: Manual Deployment to DigitalOcean Droplet

**Goal**: Deploy the Exoplanets Catalog web application to a DigitalOcean droplet that's already been provisioned.

### Current State

✅ **Completed:**
- Droplet provisioned via OpenTofu
- Parquet data files copied to server (`/app/data/exoplanets.parquet`, `/app/data/stellarhosts.parquet`)

❌ **To Do:**
- Clone GitHub repository on server
- Build application (Docker or native)
- Configure nginx reverse proxy
- Set up systemd service or Docker Compose
- Configure SSL with Let's Encrypt/certbot
- Test deployment

---

## Application Architecture

**Tech Stack:**
- **Frontend**: Leptos (Rust WASM framework)
- **Backend**: Axum server with Leptos SSR
- **Data**: Polars DataFrames loading Parquet files
- **Build**: cargo-leptos + Tailwind CSS

**Port**: 3000 (app listens on this port)

**Data Location**: App expects data files at `/app/data/` (or configurable via env var)

---

## Deployment Options

### Option A: Docker Deployment (Recommended)
**Pros:**
- Isolated environment
- Easy to update/rollback
- Consistent across environments
- Already have Dockerfile

**Cons:**
- Slightly more resource overhead
- Need Docker installed on server

### Option B: Native Binary Deployment
**Pros:**
- Lower resource usage
- Faster startup
- Direct systemd integration

**Cons:**
- Need to compile on server (slow) or cross-compile locally
- Dependencies must be installed on server

---

## Implementation Plan

### Phase 1: Server Preparation (5-10 min)

**SSH into droplet:**
```bash
ssh root@YOUR_DROPLET_IP
```

**Install dependencies:**
```bash
# Update system
apt-get update && apt-get upgrade -y

# Install Docker (if using Option A)
curl -fsSL https://get.docker.com -o get-docker.sh
sh get-docker.sh

# Install nginx
apt-get install -y nginx

# Install git
apt-get install -y git

# Install certbot for SSL
apt-get install -y certbot python3-certbot-nginx
```

**Create app directory structure:**
```bash
mkdir -p /app/data
mkdir -p /app/repo
```

---

### Phase 2: Clone Repository (2 min)

```bash
cd /app/repo
git clone https://github.com/YOUR_USERNAME/exoplanets-catalog.git .
```

**Verify data files exist:**
```bash
ls -lh /app/data/
# Should show: exoplanets.parquet, stellarhosts.parquet
```

---

### Phase 3: Build Application

#### Option A: Docker Build (20-30 min)

```bash
cd /app/repo

# Build Docker image
docker build -f infrastructure/docker/Dockerfile -t exoplanets-catalog:latest .

# Verify image built
docker images | grep exoplanets-catalog
```

**Test run:**
```bash
docker run -d \
  --name exoplanets-test \
  -p 3000:3000 \
  -v /app/data:/app/data:ro \
  exoplanets-catalog:latest

# Check logs
docker logs -f exoplanets-test

# Test locally
curl http://localhost:3000

# Stop test container
docker stop exoplanets-test && docker rm exoplanets-test
```

#### Option B: Native Build (40-60 min)

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Install cargo-leptos
cargo install --locked cargo-leptos

# Add WASM target
rustup target add wasm32-unknown-unknown

# Install Tailwind CSS
curl -sLO https://github.com/tailwindlabs/tailwindcss/releases/latest/download/tailwindcss-linux-x64
chmod +x tailwindcss-linux-x64
mv tailwindcss-linux-x64 /usr/local/bin/tailwindcss

# Build application
cd /app/repo
cargo leptos build --release

# Binary will be at: target/server/release/exoplanets-catalog
# Static files at: target/site/
```

---

### Phase 4: Configure Nginx Reverse Proxy (5 min)

**Create nginx config:**
```bash
cat > /etc/nginx/sites-available/exoplanets <<'EOF'
server {
    listen 80;
    server_name YOUR_DOMAIN.com;  # e.g., exoplanets.yourdomain.com

    location / {
        proxy_pass http://localhost:3000;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_cache_bypass $http_upgrade;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
EOF

# Enable site
ln -s /etc/nginx/sites-available/exoplanets /etc/nginx/sites-enabled/

# Test config
nginx -t

# Reload nginx
systemctl reload nginx
```

---

### Phase 5: Set Up Service

#### Option A: Docker Compose (Recommended)

**Create docker-compose.yml:**
```bash
cat > /app/docker-compose.yml <<'EOF'
version: '3.8'

services:
  exoplanets-catalog:
    image: exoplanets-catalog:latest
    container_name: exoplanets-catalog
    restart: unless-stopped
    ports:
      - "3000:3000"
    volumes:
      - /app/data:/app/data:ro
    environment:
      - RUST_LOG=info
EOF

# Start service
cd /app
docker compose up -d

# Check logs
docker compose logs -f
```

#### Option B: Systemd Service

```bash
cat > /etc/systemd/system/exoplanets.service <<'EOF'
[Unit]
Description=Exoplanets Catalog Web Application
After=network.target

[Service]
Type=simple
User=root
WorkingDirectory=/app/repo
ExecStart=/app/repo/target/server/release/exoplanets-catalog
Restart=always
RestartSec=5
Environment=RUST_LOG=info
Environment=LEPTOS_SITE_ROOT=/app/repo/target/site

[Install]
WantedBy=multi-user.target
EOF

# Reload systemd
systemctl daemon-reload

# Enable and start service
systemctl enable exoplanets.service
systemctl start exoplanets.service

# Check status
systemctl status exoplanets.service

# View logs
journalctl -u exoplanets.service -f
```

---

### Phase 6: Configure SSL (5 min)

**Set up Let's Encrypt SSL:**
```bash
certbot --nginx -d YOUR_DOMAIN.com --non-interactive --agree-tos --email YOUR_EMAIL

# Enable auto-renewal
systemctl enable certbot.timer
systemctl start certbot.timer
```

**Verify SSL:**
```bash
curl https://YOUR_DOMAIN.com
```

---

### Phase 7: Verify Deployment (2 min)

**Check all services:**
```bash
# App running (Docker)
docker ps | grep exoplanets-catalog

# OR App running (systemd)
systemctl status exoplanets.service

# Nginx running
systemctl status nginx

# SSL cert valid
certbot certificates
```

**Test application:**
```bash
# Local test
curl http://localhost:3000

# Public test
curl https://YOUR_DOMAIN.com
```

**Open in browser:**
- Visit: `https://YOUR_DOMAIN.com`
- Test tables load
- Test column selector
- Test pagination/sorting

---

## Environment Variables (Optional)

If app needs configuration:

```bash
# For Docker Compose
# Edit /app/docker-compose.yml, add under environment:
environment:
  - DATA_PATH=/app/data
  - RUST_LOG=info
  - LEPTOS_OUTPUT_NAME=exoplanets-catalog

# For systemd
# Edit /etc/systemd/system/exoplanets.service, add under [Service]:
Environment=DATA_PATH=/app/data
```

---

## Updating the Application

**Docker deployment:**
```bash
cd /app/repo
git pull origin main
docker build -f infrastructure/docker/Dockerfile -t exoplanets-catalog:latest .
cd /app
docker compose down
docker compose up -d
```

**Native deployment:**
```bash
cd /app/repo
git pull origin main
cargo leptos build --release
systemctl restart exoplanets.service
```

---

## Troubleshooting

**App won't start:**
```bash
# Check logs (Docker)
docker logs exoplanets-catalog

# Check logs (systemd)
journalctl -u exoplanets.service -n 100

# Common issues:
# - Data files not found: Check /app/data/
# - Port 3000 in use: lsof -i :3000
# - Permission issues: chown -R root:root /app
```

**Nginx errors:**
```bash
tail -f /var/log/nginx/error.log
nginx -t  # Test config
```

**SSL issues:**
```bash
certbot renew --dry-run  # Test renewal
certbot certificates  # Check cert status
```

---

## Next Steps

After manual deployment works:
1. Set up CI/CD with GitHub Actions (use DEPLOY.md as reference)
2. Configure monitoring (uptime checks, logs)
3. Set up automated backups for data
4. Configure firewall rules (UFW)

---

## Required Information

Before starting, have ready:
- [ ] Droplet IP address
- [ ] Domain name (DNS already pointing to droplet)
- [ ] GitHub repository URL
- [ ] Email for SSL certificate
- [ ] Data files confirmed at `/app/data/`
