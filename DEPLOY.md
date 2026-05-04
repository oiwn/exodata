# Deployment Guide

## Overview

This project uses a hybrid deployment approach:
- **GitHub Actions** - Builds Docker image and pushes to GitHub Container Registry (ghcr.io)
- **Ansible (local)** - Deploys to DigitalOcean droplet

This avoids storing SSH keys in GitHub Secrets while keeping the build automated.

---

## Quick Reference

```bash
# After GitHub Actions builds a new image:
just ansible-deploy

# Upload data files to server:
just ansible-upload-data

# Full server setup:
just ansible-setup

# Check server status:
just ansible-status

# View logs:
just ansible-logs

# SSH into server:
just ansible-ssh
```

---

## Prerequisites

- DigitalOcean droplet provisioned (via OpenTofu)
- Domain with DNS pointing to droplet IP
- Ansible installed locally (`brew install ansible`)
- `just` task runner installed (`brew install just`)

---

## Part 1: Initial Setup (One-Time)

### 1.1 Provision Infrastructure

```bash
cd infrastructure/tofu
cp terraform.tfvars.example terraform.tfvars
# Edit terraform.tfvars with your values

tofu init
tofu apply
```

Copy the droplet IP from output.

### 1.2 Configure Ansible

```bash
cd infrastructure/ansible
cp .env.example .env
```

Edit `.env`:
```bash
DROPLET_IP=YOUR_DROPLET_IP_HERE
# Optional (only if GHCR image is private):
GHCR_USER=your-github-username
GHCR_TOKEN=your-github-pat
# Optional (Google Analytics):
LEPTOS_GA_ID=G-XXXXXXXXXX
```

### 1.3 Test Connection

```bash
just ansible-ping
```

Expected output:
```
exoplanets | SUCCESS => {
    "changed": false,
    "ping": "pong"
}
```

### 1.4 Setup Server

```bash
just ansible-setup
```

This installs Docker, Nginx, and configures the server.

**Note:** This will fail on the "Pull Docker image" step if no image exists yet. That's OK - proceed to build the image first.

### 1.5 Configure DNS

In your DNS provider (e.g., Cloudflare):
1. Add A record pointing to your droplet IP
2. Wait for propagation (1-5 minutes)

Verify:
```bash
dig exodata.space
```

### 1.6 Setup SSL

```bash
just ansible-ssl
```

---

## Part 2: Build & Deploy

### 2.1 Trigger Docker Build

The deploy workflow triggers when you bump the version in `Cargo.toml`:

```bash
# Edit Cargo.toml, increment version
git add Cargo.toml
git commit -m "deploy: version X.Y.Z"
git push origin main
```

Or manually trigger from GitHub Actions UI:
- Go to Actions → Deploy → Run workflow

Watch the build at: https://github.com/oiwn/exoplanets-catalog/actions

### 2.2 Upload Data Files

The app needs data files (parquet + metadata):

```bash
just ansible-upload-data
```

This uploads from `data/` directory:
- `*.parquet` - Data files
- `*.toml` - Metadata files

Uploaded files are served by Nginx at `/data/` for CLI downloads, for example
`https://exodata.space/data/stellarhosts.parquet`.

### 2.3 Deploy

After the GitHub Actions build completes:

```bash
just ansible-deploy
```

This pulls the latest image and restarts the container.

### 2.4 Verify

```bash
# Check container status
just ansible-status

# View logs
just ansible-logs

# Or visit the site
open https://exodata.space
```

---

## Part 3: Subsequent Deployments

For code changes:

```bash
# 1. Make your changes
# 2. Bump version in Cargo.toml
git add .
git commit -m "your changes"
git push origin main

# 3. Wait for GitHub Actions to build (~15-20 min)

# 4. Deploy
just ansible-deploy
```

For data updates only:

```bash
just ansible-upload-data
just ansible-deploy  # Restart to pick up new data
```

If only the Nginx `/data/` serving rule changed, run:

```bash
just ansible-setup
```

---

## Troubleshooting

### Container keeps restarting

```bash
# Check logs
just ansible-logs

# Common issues:
# - Missing data files → just ansible-upload-data
# - Missing LEPTOS_* env vars → redeploy with `just ansible-deploy`
```

### App not accessible (connection refused)

```bash
# Check what address app is listening on
just ansible-run "docker logs exoplanets-catalog"

# Should show: listening on http://0.0.0.0:3000
# If it shows 127.0.0.1:3000, env vars are not set

# Check env vars
just ansible-run "docker inspect exoplanets-catalog --format '{{json .Config.Env}}'"

# Should include LEPTOS_SITE_ADDR=0.0.0.0:3000
# If not, recreate container: just ansible-deploy
```

### Can't connect to server

```bash
# Test SSH
just ansible-ping

# Check .env has correct DROPLET_IP
cat infrastructure/ansible/.env
```

### Build fails in GitHub Actions

- Check the workflow logs
- Ensure `Cargo.lock` is committed (not gitignored)
- Check Dockerfile syntax

### Image pull fails

```bash
# On the server, check if you can pull manually:
docker pull ghcr.io/oiwn/exoplanets-catalog:latest

# If private, you need GHCR_TOKEN in .env
```

### SSL issues

```bash
# Re-run SSL setup
just ansible-ssl

# Or manually on server:
ssh root@DROPLET_IP
certbot --nginx -d exodata.space
```

---

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        DEPLOYMENT FLOW                          │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│   Developer Machine                                             │
│      │                                                          │
│      │ git push (version bump)                                  │
│      ▼                                                          │
│   GitHub Actions                                                │
│      │                                                          │
│      ├──→ Build Docker image                                    │
│      │                                                          │
│      └──→ Push to ghcr.io/oiwn/exoplanets-catalog              │
│                                                                 │
│   Developer Machine                                             │
│      │                                                          │
│      │ just ansible-deploy                                      │
│      ▼                                                          │
│   Ansible (via SSH)                                             │
│      │                                                          │
│      └──→ DigitalOcean Droplet                                 │
│              │                                                  │
│              ├──→ docker pull                                   │
│              ├──→ docker stop/rm                                │
│              └──→ docker run                                    │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

## File Structure

```
infrastructure/
├── ansible/
│   ├── .env.example          # Template (committed)
│   ├── .env                  # Your config (gitignored)
│   ├── ansible.cfg
│   ├── inventory/
│   │   ├── hosts.yml
│   │   └── group_vars/all.yml
│   ├── playbooks/
│   │   ├── setup.yml         # Full server setup
│   │   ├── deploy.yml        # Pull & run container
│   │   ├── ssl.yml           # SSL certificate
│   │   └── upload-data.yml   # Upload data files
│   └── roles/
│       ├── common/
│       ├── docker/
│       ├── nginx/
│       └── app/
├── docker/
│   └── Dockerfile
└── tofu/
    ├── main.tf
    └── terraform.tfvars      # Your config (gitignored)
```

---

## Available Just Commands

| Command | Description |
|---------|-------------|
| `just ansible-ping` | Test SSH connection |
| `just ansible-setup` | Full server setup (idempotent) |
| `just ansible-deploy` | Pull latest image & restart container |
| `just ansible-ssl` | Setup/renew SSL certificate |
| `just ansible-upload-data` | Upload parquet & metadata files |
| `just ansible-status` | Check Docker & Nginx status |
| `just ansible-logs` | View container logs |
| `just ansible-ssh` | SSH into server |
| `just ansible-run "cmd"` | Run arbitrary command on server |

---

## Leptos Configuration

The app uses environment variables for production configuration (set automatically by Ansible):

| Variable | Value | Description |
|----------|-------|-------------|
| `LEPTOS_OUTPUT_NAME` | `exoplanets-catalog` | JS/WASM bundle name |
| `LEPTOS_SITE_ROOT` | `site` | Static files directory |
| `LEPTOS_SITE_PKG_DIR` | `pkg` | JS/WASM subdirectory |
| `LEPTOS_SITE_ADDR` | `0.0.0.0:3000` | Listen address (must be 0.0.0.0 for Docker) |
| `LEPTOS_ENV` | `PROD` | Environment mode |

In development, the app reads from `Cargo.toml` instead. See [leptos-rs/start-axum](https://github.com/leptos-rs/start-axum#executing-a-server-on-a-remote-machine-without-the-toolchain) for details.

### Code Splitting (`--split`)

The Dockerfile builds with `cargo leptos build --release --split`, which produces multiple WASM chunks in `target/site/pkg/`. Each lazy route gets its own `.wasm` file that loads on demand:

| File | Description |
|------|-------------|
| `exoplanets-catalog.wasm` | Main bundle (535 KB) |
| `split_*.wasm` | Per-route lazy chunks |
| `chunk_*.wasm` | Shared dependency chunks |
| `__wasm_split.______________________.js` | Chunk loader |

The runtime stage copies the entire `target/site/` directory, so all chunks are served. Nginx serves all files from the `pkg/` directory — no whitelist, no config changes needed. Content-Type headers are set correctly for all `.wasm` files by the Axum static file handler.

---

## GitHub Actions

The workflow only builds and pushes the Docker image. No GitHub Secrets are required for deployment since it's handled manually via Ansible.

---

## Costs

- **DigitalOcean Droplet**: ~$6-12/month
- **GitHub Actions**: Free (2000 min/month for private repos)
- **GitHub Container Registry**: Free (500MB)
- **SSL (Let's Encrypt)**: Free

---

## TODO

- [ ] Setup monitoring/alerting
- [ ] Automate data refresh pipeline
