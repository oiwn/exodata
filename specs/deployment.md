# Deployment Specification: Exoplanets Catalog

Ultra-simple deployment strategy: `tofu apply` + push to main = deployed.

## Overview

**Goal**: Deploy in 1-2 hours with minimal complexity.

**Stack:**
- **Platform**: DigitalOcean Droplet (Ubuntu 24.04 LTS)
- **Infrastructure as Code**: OpenTofu (local state)
- **CI/CD**: GitHub Actions (build + deploy on version change)
- **Container**: Docker
- **Reverse Proxy**: Nginx + Let's Encrypt (auto-configured via cloud-init)
- **DNS**: Cloudflare (already configured)

**Architecture:**
```
[User] → [Cloudflare DNS] → [Droplet IP]
                                  ↓
                          [Nginx :80/:443]
                                  ↓
                          [Docker Container :3000]
                                  ↓
                          [/app/data/*.parquet]
```

## Directory Structure

```
infrastructure/
├── tofu/
│   ├── main.tf           # All-in-one OpenTofu config
│   ├── terraform.tfvars  # Your secrets (gitignored)
│   └── cloud-init.yaml   # Server setup script
├── docker/
│   └── Dockerfile        # Application container
└── .github/
    └── workflows/
        └── deploy.yml    # GitHub Actions workflow
```

## Step 1: OpenTofu Configuration

Create `infrastructure/tofu/main.tf`:

```hcl
terraform {
  required_providers {
    digitalocean = {
      source  = "digitalocean/digitalocean"
      version = "~> 2.0"
    }
  }
}

variable "do_token" {
  type      = string
  sensitive = true
}

variable "ssh_key_fingerprint" {
  type = string
}

variable "domain" {
  type    = string
  default = "exoplanets.yourdomain.com"
}

variable "cloudflare_email" {
  type = string
}

provider "digitalocean" {
  token = var.do_token
}

# Create droplet
resource "digitalocean_droplet" "app" {
  name   = "exoplanets-catalog"
  region = "nyc3"
  size   = "s-2vcpu-4gb"  # $24/month
  image  = "ubuntu-24-04-x64"

  ssh_keys = [var.ssh_key_fingerprint]

  user_data = templatefile("${path.module}/cloud-init.yaml", {
    domain            = var.domain
    cloudflare_email  = var.cloudflare_email
  })
}

# Firewall
resource "digitalocean_firewall" "web" {
  name = "exoplanets-web"
  droplet_ids = [digitalocean_droplet.app.id]

  inbound_rule {
    protocol         = "tcp"
    port_range       = "22"
    source_addresses = ["0.0.0.0/0", "::/0"]
  }

  inbound_rule {
    protocol         = "tcp"
    port_range       = "80"
    source_addresses = ["0.0.0.0/0", "::/0"]
  }

  inbound_rule {
    protocol         = "tcp"
    port_range       = "443"
    source_addresses = ["0.0.0.0/0", "::/0"]
  }

  outbound_rule {
    protocol              = "tcp"
    port_range            = "1-65535"
    destination_addresses = ["0.0.0.0/0", "::/0"]
  }

  outbound_rule {
    protocol              = "udp"
    port_range            = "1-65535"
    destination_addresses = ["0.0.0.0/0", "::/0"]
  }
}

# Outputs
output "droplet_ip" {
  value = digitalocean_droplet.app.ipv4_address
  description = "Add this IP as an A record in Cloudflare DNS"
}
```

Create `infrastructure/tofu/terraform.tfvars` (gitignored):

```hcl
do_token             = "your_digitalocean_token"
ssh_key_fingerprint  = "your_ssh_key_fingerprint"
domain               = "exoplanets.yourdomain.com"
cloudflare_email     = "your@email.com"
```

Create `infrastructure/tofu/cloud-init.yaml`:

```yaml
#cloud-config
package_update: true
package_upgrade: true

packages:
  - docker.io
  - nginx
  - certbot
  - python3-certbot-nginx
  - curl

write_files:
  - path: /etc/nginx/sites-available/exoplanets
    content: |
      server {
          listen 80;
          server_name ${domain};

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

runcmd:
  # Setup Nginx
  - ln -sf /etc/nginx/sites-available/exoplanets /etc/nginx/sites-enabled/
  - rm -f /etc/nginx/sites-enabled/default
  - systemctl restart nginx

  # Setup data directory
  - mkdir -p /app/data
  - chmod 755 /app/data

  # Enable Docker
  - systemctl enable docker
  - systemctl start docker

  # Wait for DNS propagation (manual step: add A record in Cloudflare first!)
  # Then run: certbot --nginx -d ${domain} --non-interactive --agree-tos --email ${cloudflare_email}
```

Create `infrastructure/tofu/.gitignore`:

```
.terraform/
*.tfstate
*.tfstate.backup
terraform.tfvars
```

## Step 2: Dockerfile

Create `infrastructure/docker/Dockerfile`:

```dockerfile
FROM rust:1.82-slim as builder

RUN apt-get update && apt-get install -y \
    pkg-config libssl-dev curl \
    && rm -rf /var/lib/apt/lists/*

RUN cargo install --locked cargo-leptos
RUN rustup target add wasm32-unknown-unknown

RUN curl -sLO https://github.com/tailwindlabs/tailwindcss/releases/latest/download/tailwindcss-linux-x64 \
    && chmod +x tailwindcss-linux-x64 \
    && mv tailwindcss-linux-x64 /usr/local/bin/tailwindcss

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
COPY src ./src
COPY style ./style
COPY public ./public
COPY tailwind.config.js ./

RUN cargo leptos build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/server/release/exoplanets-catalog /app/
COPY --from=builder /app/target/site /app/site

EXPOSE 3000
CMD ["/app/exoplanets-catalog"]
```

Create `infrastructure/docker/.dockerignore`:

```
target/
.git/
data/
*.md
.env
```

## Step 3: GitHub Actions

Create `.github/workflows/deploy.yml`:

```yaml
name: Deploy

on:
  push:
    branches: [main]
    paths: ['Cargo.toml']
  workflow_dispatch:

env:
  REGISTRY: ghcr.io
  IMAGE_NAME: ${{ github.repository }}

jobs:
  check:
    runs-on: ubuntu-latest
    outputs:
      version: ${{ steps.version.outputs.version }}
      changed: ${{ steps.version.outputs.changed }}
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 2

      - id: version
        run: |
          VERSION=$(grep '^version = ' Cargo.toml | head -1 | cut -d'"' -f2)
          echo "version=$VERSION" >> $GITHUB_OUTPUT
          git diff HEAD~1 HEAD -- Cargo.toml | grep -q '^+version' && echo "changed=true" >> $GITHUB_OUTPUT || echo "changed=false" >> $GITHUB_OUTPUT

  build:
    runs-on: ubuntu-latest
    needs: check
    if: needs.check.outputs.changed == 'true' || github.event_name == 'workflow_dispatch'
    permissions:
      contents: read
      packages: write
    steps:
      - uses: actions/checkout@v4

      - uses: docker/setup-buildx-action@v3

      - uses: docker/login-action@v3
        with:
          registry: ${{ env.REGISTRY }}
          username: ${{ github.actor }}
          password: ${{ secrets.GITHUB_TOKEN }}

      - uses: docker/build-push-action@v5
        with:
          context: .
          file: infrastructure/docker/Dockerfile
          push: true
          tags: |
            ${{ env.REGISTRY }}/${{ env.IMAGE_NAME }}:${{ needs.check.outputs.version }}
            ${{ env.REGISTRY }}/${{ env.IMAGE_NAME }}:latest
          cache-from: type=gha
          cache-to: type=gha,mode=max

  deploy:
    runs-on: ubuntu-latest
    needs: [check, build]
    if: needs.check.outputs.changed == 'true' || github.event_name == 'workflow_dispatch'
    steps:
      - run: |
          mkdir -p ~/.ssh
          echo "${{ secrets.SSH_KEY }}" > ~/.ssh/id_rsa
          chmod 600 ~/.ssh/id_rsa
          ssh-keyscan -H ${{ secrets.DROPLET_IP }} >> ~/.ssh/known_hosts

      - run: |
          ssh -i ~/.ssh/id_rsa root@${{ secrets.DROPLET_IP }} << 'EOF'
            docker pull ${{ env.REGISTRY }}/${{ env.IMAGE_NAME }}:latest
            docker stop exoplanets-catalog || true
            docker rm exoplanets-catalog || true
            docker run -d \
              --name exoplanets-catalog \
              --restart unless-stopped \
              -p 3000:3000 \
              -v /app/data:/app/data \
              ${{ env.REGISTRY }}/${{ env.IMAGE_NAME }}:latest
            docker image prune -f
          EOF

      - run: sleep 10 && curl -f https://${{ secrets.DOMAIN }} || exit 1
```

## Deployment Steps (1-2 hours)

### Prerequisites (5 minutes)
1. DigitalOcean account + API token
2. SSH key added to DigitalOcean
3. GitHub repository created
4. Cloudflare account with domain ready

### Step 1: Infrastructure (10 minutes)

```bash
# Install OpenTofu
brew install opentofu  # or: https://opentofu.org/docs/intro/install/

# Get your DO token and SSH fingerprint
# DO Dashboard → API → Generate New Token
# DO Dashboard → Settings → Security → SSH Keys → (copy fingerprint)

# Create infrastructure
cd infrastructure/tofu
cp terraform.tfvars.example terraform.tfvars
# Edit terraform.tfvars with your values

tofu init
tofu plan
tofu apply
```

**Note the output IP address!**

### Step 2: DNS Setup (5 minutes + propagation time)

In Cloudflare DNS:
1. Add A record: `exoplanets.yourdomain.com` → `DROPLET_IP`
2. Wait 1-2 minutes for propagation
3. Verify: `dig exoplanets.yourdomain.com`

### Step 3: SSL Setup (5 minutes)

```bash
# SSH to droplet
ssh root@DROPLET_IP

# Setup SSL (after DNS propagates)
certbot --nginx -d exoplanets.yourdomain.com --non-interactive --agree-tos --email your@email.com

# Enable auto-renewal
systemctl enable certbot.timer
systemctl start certbot.timer

# Exit
exit
```

### Step 4: Upload Data Files (5 minutes)

```bash
# From your local machine
scp data/stellarhosts.parquet root@DROPLET_IP:/app/data/
scp data/exoplanets.parquet root@DROPLET_IP:/app/data/
```

### Step 5: GitHub Secrets (5 minutes)

In GitHub repository → Settings → Secrets and variables → Actions:

Add these secrets:
- `SSH_KEY`: Your private SSH key (the one that matches the DO fingerprint)
- `DROPLET_IP`: The droplet IP from tofu output
- `DOMAIN`: Your domain (e.g., `exoplanets.yourdomain.com`)

### Step 6: Deploy! (5-10 minutes)

```bash
# Bump version in Cargo.toml
version = "0.1.0"  → "0.1.1"

# Commit and push
git add Cargo.toml
git commit -m "deploy: version 0.1.1"
git push origin main

# Watch GitHub Actions run
# Visit: https://github.com/yourusername/exoplanets-catalog/actions
```

After 5-10 minutes (build time), visit:
**https://exoplanets.yourdomain.com**

## Useful Commands

```bash
# Check if app is running
ssh root@DROPLET_IP
docker ps
docker logs exoplanets-catalog

# Manual deploy
docker pull ghcr.io/yourusername/exoplanets-catalog:latest
docker stop exoplanets-catalog && docker rm exoplanets-catalog
docker run -d --name exoplanets-catalog --restart unless-stopped \
  -p 3000:3000 -v /app/data:/app/data \
  ghcr.io/yourusername/exoplanets-catalog:latest

# Check Nginx
systemctl status nginx
tail -f /var/log/nginx/error.log

# Check SSL
certbot certificates
```

## Troubleshooting

**Build fails in GitHub Actions:**
- Check logs in Actions tab
- Ensure all dependencies in Dockerfile

**Container won't start:**
```bash
ssh root@DROPLET_IP
docker logs exoplanets-catalog
# Check if data files exist
ls -la /app/data/
```

**SSL certificate fails:**
- Ensure DNS A record points to droplet
- Wait for DNS propagation (can take up to 1 hour)
- Check: `dig exoplanets.yourdomain.com`

**App not accessible:**
```bash
# Check if Nginx is running
systemctl status nginx

# Check if Docker container is running
docker ps

# Check if port 3000 is listening
netstat -tlnp | grep 3000
```

## Cost

- Droplet (s-2vcpu-4gb): **$24/month**
- Bandwidth: 2TB included
- Total: **$24/month**

## Future Improvements

When you need them (not now):
- Separate volume for data
- Automated backups
- Staging environment
- Monitoring (Prometheus/Grafana)
- Multiple regions
