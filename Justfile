# Service commands
clippy:
  cargo lx --locked -- -D warnings

# Download both NASA VOTable sources.
download-data:
  mkdir -p data
  curl --fail --location --remove-on-error --max-time 3000 --output data/stellarhosts.vot "https://exoplanetarchive.ipac.caltech.edu/TAP/sync?query=select+*+from+stellarhosts&format=votable"
  curl --fail --location --remove-on-error --max-time 3000 --output data/exoplanets.vot "https://exoplanetarchive.ipac.caltech.edu/TAP/sync?query=select+*+from+ps&format=votable"

# Convert VOTables to Parquet and generate matching metadata TOML files.
convert-raw-files:
  cargo run -p exodata -- dev convert-raw-files --data-dir data

# Confirm all runtime files exist and are non-empty.
verify-data:
  test -s data/stellarhosts.parquet
  test -s data/exoplanets.parquet
  test -s data/stellarhosts-metadata.toml
  test -s data/exoplanets-metadata.toml

# Replace served descriptions and metadata from a variant, e.g. description_pass2.
copy-descriptions $variant:
  #!/usr/bin/env bash
  set -euo pipefail
  if [[ ! "$variant" =~ ^description_[A-Za-z0-9-]+$ ]]; then
    printf 'Expected description_<label> without .md, e.g. description_pass2\n' >&2
    exit 1
  fi
  label="${variant#description_}"
  shopt -s nullglob
  articles=(content/systems/*/"$variant.md")
  if (( ${#articles[@]} == 0 )); then
    printf 'No %s.md files found under content/systems\n' "$variant" >&2
    exit 1
  fi
  for article in "${articles[@]}"; do
    system_dir="${article%/*}"
    if [[ ! -s "$article" || ! -s "$system_dir/metadata_$label.toml" ]]; then
      printf 'Missing or empty article/metadata for %s in %s\n' "$variant" "$system_dir" >&2
      exit 1
    fi
    if [[ -e "$system_dir/fail_$label.toml" ]]; then
      printf 'Latest %s generation failed in %s; resolve it before copying\n' "$label" "$system_dir" >&2
      exit 1
    fi
  done
  printf 'Copying %s descriptions and matching metadata from %s...\n' "${#articles[@]}" "$variant"
  for article in "${articles[@]}"; do
    system_dir="${article%/*}"
    cp "$article" "$system_dir/description.md"
    cp "$system_dir/metadata_$label.toml" "$system_dir/metadata.toml"
  done
  printf 'Copied %s descriptions and matching metadata. Restart the website to load them.\n' "${#articles[@]}"

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

# Upload generated stellar-host descriptions (description.md files only)
ansible-upload-descriptions:
  cd {{ansible_dir}} && ansible-playbook {{ansible_args}} playbooks/upload-descriptions.yml

# Check server status (docker + nginx)
ansible-status:
  cd {{ansible_dir}} && ansible all {{ansible_args}} -m shell -a "docker ps && echo '---' && systemctl status nginx --no-pager"

# View application logs
ansible-logs:
  cd {{ansible_dir}} && ansible all {{ansible_args}} -m shell -a "docker logs --tail 100 exodata"

# SSH into server
ansible-ssh:
  ssh root@$DROPLET_IP

# Run arbitrary ansible command
ansible-run cmd:
  cd {{ansible_dir}} && ansible all {{ansible_args}} -m shell -a "{{cmd}}"
