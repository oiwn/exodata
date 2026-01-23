# Ansible Automation Plan

This document outlines the Ansible setup for automating deployment of the Exoplanets Catalog application.

## Goals

1. **Zero manual SSH** - All server operations via Ansible playbooks
2. **Idempotent** - Safe to run multiple times, only changes what's needed
3. **Single command deployment** - `ansible-playbook deploy.yml` does everything
4. **Reproducible** - Can rebuild server from scratch if needed

---

## Directory Structure

```
infrastructure/ansible/
├── ansible.cfg                 # Ansible configuration
├── Justfile                    # Task runner commands
├── .env.example                # Template for environment variables (committed)
├── .env                        # Actual environment variables (gitignored)
├── inventory/
│   ├── hosts.yml               # Server inventory (uses env vars)
│   └── group_vars/
│       └── all.yml             # Shared variables
├── playbooks/
│   ├── setup.yml               # Full server setup
│   ├── deploy.yml              # Pull & restart container
│   ├── ssl.yml                 # SSL certificate management
│   └── upload-data.yml         # Sync parquet files
├── roles/
│   ├── common/
│   │   └── tasks/main.yml      # Basic packages, updates
│   ├── docker/
│   │   └── tasks/main.yml      # Docker installation & config
│   ├── nginx/
│   │   ├── tasks/main.yml      # Nginx setup
│   │   ├── handlers/main.yml   # Reload/restart handlers
│   │   └── templates/
│   │       └── exoplanets.conf.j2
│   ├── certbot/
│   │   └── tasks/main.yml      # Let's Encrypt SSL
│   └── app/
│       └── tasks/main.yml      # Container management
└── files/
    └── .gitkeep                # For any static files needed
```

---

## Inventory Configuration

### inventory/hosts.yml

```yaml
all:
  hosts:
    exoplanets:
      ansible_host: "{{ lookup('env', 'DROPLET_IP') }}"
      ansible_user: root
      ansible_python_interpreter: /usr/bin/python3
```

The `DROPLET_IP` is loaded from `.env` file via `just` (see justfile section below).

### inventory/group_vars/all.yml

```yaml
# Domain configuration
domain: exoplanets.example.com
admin_email: admin@example.com

# Docker image
docker_image: ghcr.io/oiwn/exoplanets-catalog
docker_tag: latest

# Application settings
app_name: exoplanets-catalog
app_port: 3000
data_path: /app/data

# Local data path (for upload-data playbook)
local_data_path: "{{ playbook_dir }}/../../../data/parquet"
```

---

## Playbooks

### 1. setup.yml - Full Server Setup

Runs all roles to configure server from scratch. Safe to re-run.

```yaml
- name: Setup Exoplanets Catalog Server
  hosts: exoplanets
  become: yes

  roles:
    - common
    - docker
    - nginx
    - app
```

**Usage:**
```bash
ansible-playbook playbooks/setup.yml
```

### 2. deploy.yml - Deploy New Version

Pulls latest image and restarts container. Use after GitHub Actions builds new image.

```yaml
- name: Deploy Exoplanets Catalog
  hosts: exoplanets
  become: yes

  tasks:
    - name: Pull latest Docker image
      docker_image:
        name: "{{ docker_image }}"
        tag: "{{ docker_tag }}"
        source: pull
        force_source: yes

    - name: Stop existing container
      docker_container:
        name: "{{ app_name }}"
        state: absent

    - name: Start new container
      docker_container:
        name: "{{ app_name }}"
        image: "{{ docker_image }}:{{ docker_tag }}"
        state: started
        restart_policy: unless-stopped
        ports:
          - "{{ app_port }}:3000"
        volumes:
          - "{{ data_path }}:/app/data:ro"

    - name: Wait for application health check
      uri:
        url: "http://localhost:{{ app_port }}"
        status_code: 200
      register: health_check
      until: health_check.status == 200
      retries: 30
      delay: 2
```

**Usage:**
```bash
ansible-playbook playbooks/deploy.yml
```

### 3. ssl.yml - SSL Certificate Setup

Obtains and configures Let's Encrypt certificate.

```yaml
- name: Setup SSL Certificate
  hosts: exoplanets
  become: yes

  tasks:
    - name: Install certbot
      apt:
        name:
          - certbot
          - python3-certbot-nginx
        state: present
        update_cache: yes

    - name: Check if certificate exists
      stat:
        path: "/etc/letsencrypt/live/{{ domain }}/fullchain.pem"
      register: cert_file

    - name: Obtain SSL certificate
      command: >
        certbot --nginx
        -d {{ domain }}
        --non-interactive
        --agree-tos
        --email {{ admin_email }}
      when: not cert_file.stat.exists

    - name: Enable certbot auto-renewal timer
      systemd:
        name: certbot.timer
        enabled: yes
        state: started
```

**Usage:**
```bash
ansible-playbook playbooks/ssl.yml
```

### 4. upload-data.yml - Upload Data Files

Syncs parquet files from local machine to server.

```yaml
- name: Upload Data Files
  hosts: exoplanets
  become: yes

  tasks:
    - name: Ensure data directory exists
      file:
        path: "{{ data_path }}"
        state: directory
        mode: '0755'

    - name: Sync parquet files
      synchronize:
        src: "{{ local_data_path }}/"
        dest: "{{ data_path }}/"
        delete: no
        recursive: yes
      delegate_to: localhost

    - name: List uploaded files
      find:
        paths: "{{ data_path }}"
        patterns: "*.parquet"
      register: data_files

    - name: Display uploaded files
      debug:
        msg: "Uploaded {{ data_files.matched }} parquet files"
```

**Usage:**
```bash
ansible-playbook playbooks/upload-data.yml
```

---

## Roles Detail

### common/tasks/main.yml

```yaml
- name: Update apt cache
  apt:
    update_cache: yes
    cache_valid_time: 3600

- name: Install common packages
  apt:
    name:
      - curl
      - htop
      - vim
    state: present
```

### docker/tasks/main.yml

```yaml
- name: Check if Docker is installed
  command: docker --version
  register: docker_check
  ignore_errors: yes
  changed_when: false

- name: Install Docker
  when: docker_check.rc != 0
  block:
    - name: Download Docker install script
      get_url:
        url: https://get.docker.com
        dest: /tmp/get-docker.sh
        mode: '0755'

    - name: Run Docker install script
      command: /tmp/get-docker.sh
      args:
        creates: /usr/bin/docker

- name: Ensure Docker service is running
  systemd:
    name: docker
    state: started
    enabled: yes

- name: Log in to GitHub Container Registry
  docker_login:
    registry: ghcr.io
    username: "{{ lookup('env', 'GHCR_USER') }}"
    password: "{{ lookup('env', 'GHCR_TOKEN') }}"
  when: lookup('env', 'GHCR_TOKEN') | length > 0
```

### nginx/tasks/main.yml

```yaml
- name: Install Nginx
  apt:
    name: nginx
    state: present

- name: Deploy Nginx site configuration
  template:
    src: exoplanets.conf.j2
    dest: /etc/nginx/sites-available/exoplanets
    mode: '0644'
  notify: Reload Nginx

- name: Enable site
  file:
    src: /etc/nginx/sites-available/exoplanets
    dest: /etc/nginx/sites-enabled/exoplanets
    state: link
  notify: Reload Nginx

- name: Remove default site
  file:
    path: /etc/nginx/sites-enabled/default
    state: absent
  notify: Reload Nginx

- name: Ensure Nginx is running
  systemd:
    name: nginx
    state: started
    enabled: yes
```

### nginx/handlers/main.yml

```yaml
- name: Reload Nginx
  systemd:
    name: nginx
    state: reloaded

- name: Restart Nginx
  systemd:
    name: nginx
    state: restarted
```

### nginx/templates/exoplanets.conf.j2

```nginx
server {
    listen 80;
    server_name {{ domain }};

    location / {
        proxy_pass http://127.0.0.1:{{ app_port }};
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
```

### app/tasks/main.yml

```yaml
- name: Ensure data directory exists
  file:
    path: "{{ data_path }}"
    state: directory
    mode: '0755'

- name: Pull Docker image
  docker_image:
    name: "{{ docker_image }}"
    tag: "{{ docker_tag }}"
    source: pull

- name: Run application container
  docker_container:
    name: "{{ app_name }}"
    image: "{{ docker_image }}:{{ docker_tag }}"
    state: started
    restart_policy: unless-stopped
    ports:
      - "{{ app_port }}:3000"
    volumes:
      - "{{ data_path }}:/app/data:ro"
```

---

## Configuration File

### ansible.cfg

```ini
[defaults]
inventory = inventory/hosts.yml
remote_user = root
host_key_checking = False
retry_files_enabled = False

[ssh_connection]
pipelining = True
ssh_args = -o ControlMaster=auto -o ControlPersist=60s
```

---

## Environment Configuration

### .env.example (committed to git)

```bash
# Copy this file to .env and fill in your values
# cp .env.example .env

# Required: DigitalOcean droplet IP address
DROPLET_IP=YOUR_DROPLET_IP_HERE

# Optional: GitHub Container Registry credentials (for private images)
GHCR_USER=your-github-username
GHCR_TOKEN=your-github-pat
```

### .env (gitignored)

```bash
DROPLET_IP=167.99.xxx.xxx
GHCR_USER=oiwn
GHCR_TOKEN=ghp_xxxxxxxxxxxx
```

### .gitignore additions

```gitignore
# Ansible environment
infrastructure/ansible/.env
```

---

## Justfile (Task Runner)

The `justfile` provides convenient shortcuts and automatically loads `.env` variables.

### justfile

```just
# Load environment variables from .env file
set dotenv-load

# Default recipe: show available commands
default:
    @just --list

# Test SSH connection to server
ping:
    ansible all -m ping

# Full server setup (idempotent)
setup:
    ansible-playbook playbooks/setup.yml

# Deploy latest Docker image
deploy:
    ansible-playbook playbooks/deploy.yml

# Setup SSL certificate
ssl:
    ansible-playbook playbooks/ssl.yml

# Upload parquet data files
upload-data:
    ansible-playbook playbooks/upload-data.yml

# Check server status
status:
    ansible all -m shell -a "docker ps && echo '---' && systemctl status nginx --no-pager"

# View application logs
logs:
    ansible all -m shell -a "docker logs --tail 100 exoplanets-catalog"

# SSH into server
ssh:
    ssh root@${DROPLET_IP}

# Run arbitrary ansible command
run cmd:
    ansible all -m shell -a "{{cmd}}"
```

---

## Environment Variables Reference

| Variable | Purpose | Local (.env) | GitHub Actions |
|----------|---------|--------------|----------------|
| `DROPLET_IP` | Server IP address | Required | `secrets.DROPLET_IP` |
| `GHCR_USER` | GitHub username | Optional | `github.actor` |
| `GHCR_TOKEN` | GitHub PAT | Optional | `secrets.GITHUB_TOKEN` |

---

## Usage Workflows

All commands are run from `infrastructure/ansible/` directory using `just`.

### Initial Setup (One-Time)

```bash
cd infrastructure/ansible

# 1. Create .env from template
cp .env.example .env

# 2. Edit .env with your droplet IP
vim .env  # or use your preferred editor

# 3. Test connection
just ping
```

### First-Time Setup (New Droplet)

```bash
cd infrastructure/ansible

# 1. Run full setup (Docker, Nginx, app container)
just setup

# 2. Upload data files
just upload-data

# 3. Setup SSL (after DNS is configured)
just ssl
```

### Regular Deployment

```bash
cd infrastructure/ansible

# After GitHub Actions builds new image:
just deploy
```

### Update Data Files

```bash
just upload-data
```

### Fix Server Drift / Ensure Configuration

```bash
just setup  # Idempotent, safe to re-run
```

### Debugging Commands

```bash
# Check server status (docker + nginx)
just status

# View application logs
just logs

# SSH into server
just ssh

# Run arbitrary command on server
just run "df -h"
just run "docker images"
```

---

## Integration with GitHub Actions

Two options for automated deployment:

### Option A: GitHub Actions calls Ansible (Recommended)

GitHub Actions uses the same Ansible playbooks but gets environment variables from GitHub Secrets instead of `.env` file.

```yaml
# In .github/workflows/deploy.yml
deploy:
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4

    - name: Install Ansible
      run: pip install ansible

    - name: Setup SSH key
      run: |
        mkdir -p ~/.ssh
        echo "${{ secrets.SSH_KEY }}" > ~/.ssh/id_rsa
        chmod 600 ~/.ssh/id_rsa
        ssh-keyscan -H ${{ secrets.DROPLET_IP }} >> ~/.ssh/known_hosts

    - name: Deploy via Ansible
      env:
        DROPLET_IP: ${{ secrets.DROPLET_IP }}
        GHCR_USER: ${{ github.actor }}
        GHCR_TOKEN: ${{ secrets.GITHUB_TOKEN }}
      run: |
        cd infrastructure/ansible
        ansible-playbook playbooks/deploy.yml
```

**Note:** In GitHub Actions, environment variables are set directly via `env:` block. The `justfile` with `set dotenv-load` is for local development convenience only.

### Option B: Keep Current SSH Deploy

Keep existing GitHub Actions SSH commands for deployment, use Ansible only for local operations (setup, data upload, debugging).

This is simpler if the current deploy workflow is working well.

---

## Security Notes

1. **SSH Key**: Never commit private keys. Use environment variables or GitHub Secrets.
2. **Ansible Vault**: For sensitive variables, use `ansible-vault encrypt` on group_vars files.
3. **GHCR Token**: Use a Personal Access Token with minimal `read:packages` scope.

---

## Next Steps

I created Justfile, it's alrady existing.

1. [ ] Create the directory structure
2. [ ] Write the actual Ansible files
3. [ ] Create `.env.example`
4. [ ] Add `.env` to `.gitignore`
5. [ ] Create local `.env` with droplet IP
6. [ ] Test with `just ping`
7. [ ] Run `just setup` on existing droplet
8. [ ] Integrate with GitHub Actions (Option A or B)
9. [ ] Document in main README
