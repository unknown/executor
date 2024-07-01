terraform {
  required_version = ">= 0.12"
  required_providers {
    hcloud = {
      source  = "hetznercloud/hcloud"
      version = "~> 1.45"
    }
    random = {
      source  = "hashicorp/random"
      version = ">= 2"
    }
  }
}
