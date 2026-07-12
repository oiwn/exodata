# Service commands
clippy:
  cargo clippy --all-targets --all-features -- -D warnings

# Download both NASA VOTable sources.
download-data:
  mkdir -p data
  curl --fail --location --remove-on-error --max-time 3000 --output data/stellarhosts.vot "https://exoplanetarchive.ipac.caltech.edu/TAP/sync?query=select+*+from+stellarhosts&format=votable"
  curl --fail --location --remove-on-error --max-time 3000 --output data/exoplanets.vot "https://exoplanetarchive.ipac.caltech.edu/TAP/sync?query=select+*+from+pscomppars&format=votable"

# Convert VOTables to Parquet and generate matching metadata TOML files.
convert-raw-files:
  cargo run -p exodata -- dev convert-raw-files --data-dir data

# Confirm all runtime files exist and are non-empty.
verify-data:
  test -s data/stellarhosts.parquet
  test -s data/exoplanets.parquet
  test -s data/stellarhosts-metadata.toml
  test -s data/exoplanets-metadata.toml

# =============================================================================
# Ansible Deployment Commands
# =============================================================================
# Load environment variables from .env file in ansible directory
set dotenv-load
set dotenv-path := "infrastructure/ansible/.env"

# Ansible working directory
ansible_dir := "infrastructure/ansible"

# Common ansible args (pass droplet_ip from environment)
ansible_args := "-e droplet_ip=$DROPLET_IP"

# Test SSH connection to server
ansible-ping:
  cd {{ansible_dir}} && ansible all {{ansible_args}} -m ping

# Full server setup (idempotent)
ansible-setup:
  cd {{ansible_dir}} && ansible-playbook {{ansible_args}} playbooks/setup.yml

# Deploy latest Docker image
ansible-deploy:
  cd {{ansible_dir}} && ansible-playbook {{ansible_args}} playbooks/deploy.yml

# Setup SSL certificate
ansible-ssl:
  cd {{ansible_dir}} && ansible-playbook {{ansible_args}} playbooks/ssl.yml

# Upload Parquet data and metadata TOML files
ansible-upload-data:
  cd {{ansible_dir}} && ansible-playbook {{ansible_args}} playbooks/upload-data.yml

# Check server status (docker + nginx)
ansible-status:
  cd {{ansible_dir}} && ansible all {{ansible_args}} -m shell -a "docker ps && echo '---' && systemctl status nginx --no-pager"

# View application logs
ansible-logs:
  cd {{ansible_dir}} && ansible all {{ansible_args}} -m shell -a "docker logs --tail 100 exoplanets-catalog"

# SSH into server
ansible-ssh:
  ssh root@$DROPLET_IP

# Run arbitrary ansible command
ansible-run cmd:
  cd {{ansible_dir}} && ansible all {{ansible_args}} -m shell -a "{{cmd}}"
