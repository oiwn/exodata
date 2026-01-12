# Deployment Guide

## Current State

✅ **Completed:**
- GitHub Actions workflow (`build-test.yml`) - builds and creates artifacts
- Infrastructure code (OpenTofu) - ready to provision droplet
- Docker setup - working Dockerfile for containerized deployment

❌ **To Do:**
- Provision DigitalOcean droplet
- Manual deployment testing
- Automated deployment setup

---

## Deployment Approaches

### Approach 1: Docker Deployment (Automated - Existing)
Uses `.github/workflows/deploy.yml` to:
- Build Docker image in GitHub Actions
- Push to GitHub Container Registry (ghcr.io)
- SSH to droplet and deploy

**Status**: Workflow exists, needs droplet + GitHub secrets configured

### Approach 2: Manual Binary Deployment (Testing)
Uses `.github/workflows/build-test.yml` to:
- Build artifacts (server binary + site files)
- Download and manually deploy to droplet

**Status**: Workflow working, manual deployment process being documented

---

# Part 1: Infrastructure Setup

## Step 1: Install OpenTofu (2 min)

```bash
brew install opentofu
# or download from: https://opentofu.org/docs/intro/install/
```

## Step 2: DigitalOcean Setup (5 min)

1. **Create API Token:**

Follow: "https://cloud.digitalocean.com/account/api/tokens"

2. **Get SSH Key Fingerprint:**

Go to : "https://cloud.digitalocean.com/account/security", copy the fingerprint (format: `aa:bb:cc:...`)

## Step 3: Configure Infrastructure

```bash
cd infrastructure/tofu
cp terraform.tfvars.example terraform.tfvars
```

Edit `terraform.tfvars`:
```hcl
do_token            = "dop_v1_YOUR_TOKEN_HERE"
ssh_key_fingerprint = "YOUR_FINGERPRINT_HERE"
domain              = "exoplanets.yourdomain.com"
cloudflare_email    = "your@email.com"
```

## Step 4: Deploy Infrastructure

```bash
cd infrastructure/tofu
tofu init
tofu plan
tofu apply
```

**Copy the IP address from output!**

## Step 5: DNS Setup

In Cloudflare DNS:
1. Go to your domain → DNS → Records
2. Add A record:
   - Type: `A`
   - Name: `exoplanets` (or `@` for root domain)
   - IPv4 address: `DROPLET_IP_FROM_STEP_4`
   - Proxy status: DNS only (gray cloud)
   - TTL: Auto

Wait 1-2 minutes, then verify:
```bash
dig exoplanets.yourdomain.com
```

## Step 6: Setup SSL (5 min)

```bash
ssh root@DROPLET_IP

# Wait for cloud-init to finish (check: tail -f /var/log/cloud-init-output.log)

# Setup SSL
certbot --nginx -d exodata.space --non-interactive --agree-tos --email your@email.com

# Enable auto-renewal
systemctl enable certbot.timer
systemctl start certbot.timer

exit
```

## Step 7: Upload Data

If you don't have data yet, fetch it first:
```bash
# On your local machine
./scripts/update-data.sh  # Creates data/parquet/*.parquet files
```

Upload to droplet:
```bash
scp data/parquet/stellarhosts.parquet root@DROPLET_IP:/app/data/
scp data/parquet/exoplanets.parquet root@DROPLET_IP:/app/data/
```

^^^ need to add metadata files

## Step 8: Configure GitHub Secrets (5 min)

Go to: https://github.com/YOUR_USERNAME/exoplanets-catalog/settings/secrets/actions

Click "New repository secret" and add:

1. **SSH_KEY**
   ```bash
   cat ~/.ssh/id_rsa
   # Copy entire private key including BEGIN/END lines
   ```

2. **DROPLET_IP**
   ```
   YOUR_DROPLET_IP
   ```

3. **DOMAIN**
   ```
   exoplanets.yourdomain.com
   ```

## Step 9: Deploy! (10 min build time)

```bash
# Bump version
# Edit Cargo.toml, change version = "0.1.0" to "0.1.1"

git add .
git commit -m "deploy: initial deployment v0.1.1"
git push origin main
```

Watch the deployment:
- Go to: https://github.com/YOUR_USERNAME/exoplanets-catalog/actions
- Click on the running workflow
- Wait for build to complete (~10 minutes)

## Step 10: Verify!

Visit: **https://exoplanets.yourdomain.com**

You should see your exoplanets catalog! 🚀

## Troubleshooting

**Build fails?**
- Check GitHub Actions logs
- Ensure all dependencies are correct

**Can't access site?**
```bash
ssh root@DROPLET_IP
docker ps  # Container running?
docker logs exoplanets-catalog  # Check logs
systemctl status nginx  # Nginx running?
```

**SSL not working?**
- Ensure DNS points to correct IP: `dig exoplanets.yourdomain.com`
- Wait for DNS propagation (can take up to 1 hour)
- Re-run certbot: `certbot --nginx -d exoplanets.yourdomain.com`

## Next Deployment

To deploy updates:
1. Make your changes
2. Bump version in `Cargo.toml`
3. Push to `main`
4. GitHub Actions automatically builds and deploys!

## Useful Commands

```bash
# SSH to server
ssh root@DROPLET_IP

# Check app logs
docker logs -f exoplanets-catalog

# Restart app
docker restart exoplanets-catalog

# Check Nginx
tail -f /var/log/nginx/error.log

# Destroy infrastructure (WARNING: deletes everything!)
cd infrastructure/tofu
tofu destroy
```

---

# Part 2: Manual Deployment (Alternative Approach)

> **Note**: This documents manual deployment using pre-built artifacts from GitHub Actions.
> Use this to understand the deployment process before automating it.

## Prerequisites

- DigitalOcean droplet provisioned (Steps 1-4 above)
- DNS configured (Step 5 above)
- Data files uploaded (Step 7 above)
- Build artifacts from GitHub Actions workflow

## Step 1: Download Build Artifacts

After `build-test.yml` workflow runs successfully:

```bash
# Install GitHub CLI if needed
brew install gh

# Authenticate
gh auth login

# List recent workflow runs
gh run list --repo oiwn/exoplanets-catalog

# Download artifacts from latest run
gh run download <RUN_ID> --repo oiwn/exoplanets-catalog

# You'll get two directories:
# - server-binary/exoplanets-catalog (backend)
# - site-files/ (frontend assets)
```

## Step 2: Prepare Droplet

```bash
# SSH into droplet
ssh root@DROPLET_IP

# Create application directory structure
mkdir -p /app/bin
mkdir -p /app/site
mkdir -p /app/data

# Install nginx (if not already installed)
apt-get update && apt-get install -y nginx

# Exit for now
exit
```

## Step 3: Upload Application Files

```bash
# Upload server binary
scp server-binary/exoplanets-catalog root@DROPLET_IP:/app/bin/
ssh root@DROPLET_IP 'chmod +x /app/bin/exoplanets-catalog'

# Upload frontend files
scp -r site-files/* root@DROPLET_IP:/app/site/

# Verify data files exist
ssh root@DROPLET_IP 'ls -lh /app/data/'
```

## Step 4: Configure Systemd Service

```bash
ssh root@DROPLET_IP

# Create systemd service
cat > /etc/systemd/system/exoplanets.service <<'EOF'
[Unit]
Description=Exoplanets Catalog Web Application
After=network.target

[Service]
Type=simple
User=root
WorkingDirectory=/app
ExecStart=/app/bin/exoplanets-catalog
Restart=always
RestartSec=5
Environment=RUST_LOG=info
Environment=LEPTOS_SITE_ROOT=/app/site
Environment=LEPTOS_SITE_ADDR=127.0.0.1:3000

[Install]
WantedBy=multi-user.target
EOF

# Reload systemd and start service
systemctl daemon-reload
systemctl enable exoplanets.service
systemctl start exoplanets.service

# Check status
systemctl status exoplanets.service

# View logs
journalctl -u exoplanets.service -f
```

## Step 5: Configure Nginx

```bash
# Still on the droplet
cat > /etc/nginx/sites-available/exoplanets <<'EOF'
server {
    listen 80;
    server_name YOUR_DOMAIN.com;

    location / {
        proxy_pass http://127.0.0.1:3000;
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
ln -sf /etc/nginx/sites-available/exoplanets /etc/nginx/sites-enabled/
rm -f /etc/nginx/sites-enabled/default

# Test and reload
nginx -t
systemctl reload nginx
```

## Step 6: Setup SSL

```bash
# Install certbot
apt-get install -y certbot python3-certbot-nginx

# Get certificate
certbot --nginx -d YOUR_DOMAIN.com --non-interactive --agree-tos --email YOUR_EMAIL

# Enable auto-renewal
systemctl enable certbot.timer
systemctl start certbot.timer

# Verify
certbot certificates

# Exit droplet
exit
```

## Step 7: Verify Deployment

```bash
# Test from your machine
curl https://YOUR_DOMAIN.com

# Or open in browser
open https://YOUR_DOMAIN.com
```

## Updating the Application

When you need to deploy updates:

```bash
# 1. Push changes to GitHub - triggers build-test.yml workflow
git push origin main

# 2. Download new artifacts
gh run download <NEW_RUN_ID> --repo oiwn/exoplanets-catalog

# 3. Upload to droplet
scp server-binary/exoplanets-catalog root@DROPLET_IP:/app/bin/
scp -r site-files/* root@DROPLET_IP:/app/site/

# 4. Restart service
ssh root@DROPLET_IP 'systemctl restart exoplanets.service'
```

## Troubleshooting Manual Deployment

```bash
# Check if app is running
ssh root@DROPLET_IP 'systemctl status exoplanets.service'

# View application logs
ssh root@DROPLET_IP 'journalctl -u exoplanets.service -n 100'

# Check if port 3000 is listening
ssh root@DROPLET_IP 'netstat -tlnp | grep 3000'

# Test app directly (bypass nginx)
ssh root@DROPLET_IP 'curl http://localhost:3000'

# Check nginx logs
ssh root@DROPLET_IP 'tail -f /var/log/nginx/error.log'
```

---

