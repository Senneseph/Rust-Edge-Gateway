terraform {
  required_providers {
    digitalocean = {
      source  = "digitalocean/digitalocean"
      version = "~> 2.0"
    }
  }
}

variable "do_token" {
  description = "DigitalOcean API token"
  type        = string
  sensitive   = true
}

variable "ssh_key_fingerprint" {
  description = "SSH key fingerprint for droplet access"
  type        = string
}

variable "domain" {
  description = "Domain for Rust Edge Gateway"
  type        = string
  default     = "rust-edge-gateway.iffuso.com"
}

variable "droplet_ip" {
  description = "Existing droplet IP (if using existing droplet)"
  type        = string
  default     = ""
}

provider "digitalocean" {
  token = var.do_token
}

# DNS Records for Rust Edge Gateway
resource "digitalocean_record" "rust_edge_gateway" {
  count  = var.droplet_ip != "" ? 1 : 0
  domain = "iffuso.com"
  type   = "A"
  name   = "rust-edge-gateway"
  value  = var.droplet_ip
  ttl    = 3600
}

resource "digitalocean_record" "rust_edge_gateway_wildcard" {
  count  = var.droplet_ip != "" ? 1 : 0
  domain = "iffuso.com"
  type   = "A"
  name   = "*.rust-edge-gateway"
  value  = var.droplet_ip
  ttl    = 3600
}

# Firewall rules for Rust Edge Gateway
resource "digitalocean_firewall" "rust_edge_gateway" {
  name = "rust-edge-gateway-fw"

  # Allow SSH
  inbound_rule {
    protocol         = "tcp"
    port_range       = "22"
    source_addresses = ["0.0.0.0/0", "::/0"]
  }

  # Allow HTTP
  inbound_rule {
    protocol         = "tcp"
    port_range       = "80"
    source_addresses = ["0.0.0.0/0", "::/0"]
  }

  # Allow HTTPS
  inbound_rule {
    protocol         = "tcp"
    port_range       = "443"
    source_addresses = ["0.0.0.0/0", "::/0"]
  }

  # Allow all outbound
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

output "rust_edge_gateway_domain" {
  value = var.domain
}

output "droplet_ip" {
  value = var.droplet_ip
}

