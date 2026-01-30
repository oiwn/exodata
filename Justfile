# Data download commands
download-stellarhosts:
    curl "https://exoplanetarchive.ipac.caltech.edu/TAP/sync?query=select+*+from+stellarhosts" -L --max-time 2000 > data/stellarhosts.vot

download-exoplanets:
    curl "https://exoplanetarchive.ipac.caltech.edu/TAP/sync?query=select+*+from+ps" -L --max-time 2000 > data/exoplanets.vot

stellarhosts-metadata:
    cargo run -p exo-cli -- view-metadata --path data/stellarhosts.vot

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

# Upload parquet data files
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
