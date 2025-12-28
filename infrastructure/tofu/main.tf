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
  size   = "s-1vcpu-2gb" # $12/month
  image  = "ubuntu-24-04-x64"

  ssh_keys = [var.ssh_key_fingerprint]

  user_data = templatefile("${path.module}/cloud-init.yaml", {
    domain           = var.domain
    cloudflare_email = var.cloudflare_email
  })
}

# Firewall
resource "digitalocean_firewall" "web" {
  name        = "exoplanets-web"
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
  value       = digitalocean_droplet.app.ipv4_address
  description = "Add this IP as an A record in Cloudflare DNS"
}
