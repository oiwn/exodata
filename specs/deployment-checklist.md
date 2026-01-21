# Deployment Checklist - Docker via GitHub Actions

This guide covers deploying the Exoplanets Catalog to DigitalOcean using Docker containers and automated GitHub Actions deployment.

## Prerequisites

- DigitalOcean droplet provisioned (via OpenTofu or manually)
- Domain name with DNS pointing to droplet IP
- GitHub repository with admin access
- SSH key for accessing droplet

---

## Part 1: Prepare Droplet (One-time Setup)

### 1.1 SSH into Droplet

```bash
ssh root@YOUR_DROPLET_IP
```

### 1.2 Install Docker

```bash
# Download and run Docker installation script
curl -fsSL https://get.docker.com -o get-docker.sh
sh get-docker.sh

# Verify installation
docker --version
```

### 1.3 Install Nginx

```bash
apt-get update && apt-get install -y nginx

# Verify nginx is running
systemctl status nginx
```

### 1.4 Create Application Directories

```bash
mkdir -p /app/data
```

### 1.5 Exit Droplet (for now)

```bash
exit
```

---

## Part 2: Upload Data Files

From your local machine:

```bash
# Upload parquet data files to droplet
scp data/parquet/*.parquet root@YOUR_DROPLET_IP:/app/data/

# Verify files were uploaded
ssh root@YOUR_DROPLET_IP 'ls -lh /app/data/'
```

---

## Part 3: Configure Nginx Reverse Proxy

### 3.1 SSH Back into Droplet

```bash
ssh root@YOUR_DROPLET_IP
```

### 3.2 Create Nginx Configuration

```bash
cat > /etc/nginx/sites-available/exoplanets <<'EOF'
server {
    listen 80;
    server_name YOUR_DOMAIN.com;

    location / {
        proxy_pass http://127.0.0.1:3000;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
EOF
```

**Note:** Replace `YOUR_DOMAIN.com` with your actual domain name.

### 3.3 Enable Site

```bash
# Create symlink to enable the site
ln -sf /etc/nginx/sites-available/exoplanets /etc/nginx/sites-enabled/

# Remove default site
rm -f /etc/nginx/sites-enabled/default

# Test nginx configuration
nginx -t

# Reload nginx
systemctl reload nginx
```

---

## Part 4: Setup SSL with Let's Encrypt

**Important:** Ensure DNS is already pointing to your droplet IP before running this step!

```bash
# Install certbot
apt-get install -y certbot python3-certbot-nginx

# Obtain SSL certificate
certbot --nginx -d YOUR_DOMAIN.com --non-interactive --agree-tos --email YOUR_EMAIL

# Enable automatic renewal
systemctl enable certbot.timer
systemctl start certbot.timer

# Verify certificate
certbot certificates

# Exit droplet
exit
```

**Note:** Replace `YOUR_DOMAIN.com` and `YOUR_EMAIL` with your actual values.

---

## Part 5: Configure GitHub Secrets

### 5.1 Navigate to GitHub Secrets

Go to: `https://github.com/oiwn/exoplanets-catalog/settings/secrets/actions`

### 5.2 Add Required Secrets

Click **"New repository secret"** for each of the following:

#### Secret 1: `SSH_KEY`

Get your SSH private key:

```bash
cat ~/.ssh/id_rsa
```

Copy the **entire output** including:
- `-----BEGIN OPENSSH PRIVATE KEY-----`
- All the key content
- `-----END OPENSSH PRIVATE KEY-----`

Paste into GitHub secret value.

#### Secret 2: `DROPLET_IP`

Value: Your droplet's IP address (e.g., `167.99.123.456`)

#### Secret 3: `DOMAIN`

Value: Your domain name (e.g., `exoplanets.yourdomain.com`)

---

## Part 6: Deploy Application

### 6.1 Prepare Code Changes

```bash
# Commit any pending changes (like the fixed Dockerfile)
git add infrastructure/docker/Dockerfile
git commit -m "Fix Dockerfile for multi-arch support"
```

### 6.2 Bump Version (Triggers Deploy Workflow)

The `deploy.yml` workflow only triggers when version changes in `Cargo.toml`.

```bash
# Edit Cargo.toml
# Change: version = "0.1.0"
# To:     version = "0.1.1"

git add Cargo.toml
git commit -m "deploy: version 0.1.1"
```

### 6.3 Push to GitHub

```bash
git push origin main
```

This will automatically trigger the deployment workflow!

---

## Part 7: Monitor Deployment

### 7.1 Watch GitHub Actions

1. Go to: `https://github.com/oiwn/exoplanets-catalog/actions`
2. Click on the latest "Deploy" workflow run
3. Watch the progress

**Workflow Steps:**
1. **Check** - Validates version change (~30 seconds)
2. **Build** - Builds x86_64 Docker image (~15-20 minutes)
   - Installs cargo-leptos via cargo-binstall
   - Compiles Rust + Leptos + WASM
   - Creates optimized production image
   - Pushes to GitHub Container Registry (ghcr.io)
3. **Deploy** - Deploys to droplet (~1-2 minutes)
   - SSHs into droplet
   - Pulls latest image from ghcr.io
   - Stops old container
   - Starts new container with data mounted
   - Runs health check

### 7.2 Check Deployment Logs

If deployment fails, check the GitHub Actions logs for errors.

---

## Part 8: Verify Deployment

### 8.1 Test via HTTP/HTTPS

```bash
# Test HTTPS endpoint
curl https://YOUR_DOMAIN.com

# Or open in browser
open https://YOUR_DOMAIN.com
```

### 8.2 Check Container on Droplet

```bash
# SSH into droplet
ssh root@YOUR_DROPLET_IP

# Check running containers
docker ps | grep exoplanets-catalog

# Check container logs
docker logs exoplanets-catalog

# Check container health
docker inspect exoplanets-catalog | grep -A 10 State
```

### 8.3 Expected Result

You should see:
- Container running with status "Up"
- Application accessible at `https://YOUR_DOMAIN.com`
- SSL certificate valid (green padlock in browser)
- Exoplanets data loading correctly

---

## Part 9: Future Deployments

For subsequent deployments:

### 9.1 Make Code Changes

```bash
# Make your changes
git add .
git commit -m "Your changes"
```

### 9.2 Bump Version

```bash
# Edit Cargo.toml
# Increment version: 0.1.1 → 0.1.2

git add Cargo.toml
git commit -m "deploy: version 0.1.2"
git push origin main
```

The workflow automatically:
- Builds new Docker image
- Pushes to registry
- Deploys to droplet
- Zero downtime (container swap)

---

## Troubleshooting

### Build Fails in GitHub Actions

**Check:**
```bash
# View workflow logs on GitHub
# Look for compilation errors or dependency issues
```

**Common Issues:**
- Cargo.toml syntax error
- Missing dependencies
- Tailwind CSS errors

### Deployment Fails

**SSH Issues:**
```bash
# Verify SSH_KEY secret is correct
# Test SSH manually: ssh root@DROPLET_IP
```

**Container Issues:**
```bash
# SSH into droplet
ssh root@YOUR_DROPLET_IP

# Check if old container is still running
docker ps -a | grep exoplanets-catalog

# Manually stop and remove
docker stop exoplanets-catalog
docker rm exoplanets-catalog

# Try pulling image manually
docker pull ghcr.io/oiwn/exoplanets-catalog:latest

# Run container manually
docker run -d \
  --name exoplanets-catalog \
  --restart unless-stopped \
  -p 3000:3000 \
  -v /app/data:/app/data \
  ghcr.io/oiwn/exoplanets-catalog:latest
```

### Application Not Accessible

**Check Nginx:**
```bash
ssh root@YOUR_DROPLET_IP

# Test nginx config
nginx -t

# Check nginx status
systemctl status nginx

# Check nginx logs
tail -f /var/log/nginx/error.log
```

**Check Firewall:**
```bash
# Ensure ports 80 and 443 are open
ufw status
```

**Check Container:**
```bash
# Is app listening on port 3000?
curl http://localhost:3000

# Check container logs
docker logs -f exoplanets-catalog
```

### SSL Certificate Issues

**Renew Certificate:**
```bash
ssh root@YOUR_DROPLET_IP

# Test renewal
certbot renew --dry-run

# Force renewal if needed
certbot renew --force-renewal

# Check status
certbot certificates
```

**DNS Issues:**
```bash
# Verify DNS points to correct IP
dig YOUR_DOMAIN.com

# Wait for DNS propagation (up to 1 hour)
```

---

## Manual Container Management

### Pull Latest Image

```bash
ssh root@YOUR_DROPLET_IP
docker pull ghcr.io/oiwn/exoplanets-catalog:latest
```

### Restart Container

```bash
ssh root@YOUR_DROPLET_IP
docker restart exoplanets-catalog
```

### View Logs

```bash
ssh root@YOUR_DROPLET_IP
docker logs -f exoplanets-catalog
```

### Stop/Remove Container

```bash
ssh root@YOUR_DROPLET_IP
docker stop exoplanets-catalog
docker rm exoplanets-catalog
```

### Cleanup Old Images

```bash
ssh root@YOUR_DROPLET_IP
docker image prune -f
```

---

## Architecture Summary

```
Developer Machine
  ↓ git push
GitHub Actions (x86_64 runner)
  ↓ builds Docker image
GitHub Container Registry (ghcr.io)
  ↓ docker pull
DigitalOcean Droplet
  ↓ port 3000
Nginx Reverse Proxy
  ↓ port 80/443
Internet (HTTPS)
```

**Key Points:**
- Build happens in GitHub Actions (fast x86_64 native build)
- Image stored in GitHub Container Registry (private by default)
- Droplet only pulls and runs pre-built image (no compilation on server)
- Nginx handles SSL termination and proxying
- Data files mounted as read-only volume

---

## Security Notes

- SSH private key stored securely in GitHub Secrets
- Container runs with minimal privileges
- Data mounted read-only (`:ro` flag)
- SSL/TLS encryption via Let's Encrypt
- GitHub Container Registry images are private by default
- No exposed ports except 80/443 (nginx)

---

## Performance Optimization

**GitHub Actions:**
- Uses cargo-binstall for faster cargo-leptos installation (~15s vs 5-10min)
- Multi-stage Docker build (small runtime image)
- GitHub Actions cache for dependencies

**Runtime:**
- Debian bookworm-slim base (minimal footprint)
- Only necessary libraries included
- Leptos SSR for fast initial page load
- WASM for interactive client-side features

---

## Monitoring & Maintenance

**Regular Tasks:**
- Monitor GitHub Actions for failed deployments
- Check container logs for errors: `docker logs exoplanets-catalog`
- Verify SSL certificate auto-renewal: `certbot certificates`
- Update data files as needed

**Automated:**
- SSL certificate renewal (certbot.timer)
- Container restart on failure (--restart unless-stopped)
- Automatic deployment on version bump

---

## Cost Considerations

**DigitalOcean:**
- Basic Droplet: $4-6/month (sufficient for low-traffic app)
- Bandwidth: Usually included
- Backups: Optional ($1-2/month)

**GitHub:**
- Private repo: Free (2000 minutes/month)
- Container Registry: Free (500MB storage)
- Actions: ~20 min per deployment

**Domain & DNS:**
- Domain registration: Variable
- Cloudflare DNS: Free tier available

**SSL:**
- Let's Encrypt: Free

---

## Next Steps After Deployment

1. **Monitor application performance**
   - Set up uptime monitoring (UptimeRobot, etc.)
   - Check resource usage: `docker stats exoplanets-catalog`

2. **Set up backups**
   - DigitalOcean droplet snapshots
   - Data file backups (parquet files)

3. **Configure monitoring**
   - Application logs aggregation
   - Error tracking (Sentry, etc.)

4. **Optimize caching**
   - Add nginx caching for static assets
   - Configure browser caching headers

5. **Scale if needed**
   - Upgrade droplet size
   - Add CDN for static assets
   - Database for dynamic data (if applicable)

---

## Summary

✅ **One-time Setup:**
- Provision droplet
- Install Docker + Nginx
- Configure SSL
- Set GitHub secrets

✅ **Every Deployment:**
- Bump version in Cargo.toml
- Push to main
- GitHub Actions handles everything else

✅ **Zero Manual Work:**
- Build, push, deploy all automated
- Container auto-restarts on failure
- SSL auto-renews

🚀 **Deploy with confidence!**
