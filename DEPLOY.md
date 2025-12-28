# Quick Deployment Checklist

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

## Step 4: Deploy Infrastructure (10 min)

```bash
cd infrastructure/tofu
tofu init
tofu plan
tofu apply
```

**Copy the IP address from output!**

## Step 5: DNS Setup (2 min + wait for propagation)

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
certbot --nginx -d exoplanets.yourdomain.com --non-interactive --agree-tos --email your@email.com

# Enable auto-renewal
systemctl enable certbot.timer
systemctl start certbot.timer

exit
```

## Step 7: Upload Data (5 min)

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

## Step 10: Verify! (1 min)

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

**Total time: ~50 minutes + waiting for DNS propagation**

Good luck! 🌟
