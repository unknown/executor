terraform {
  required_version = ">= 0.12"
  required_providers {
    vault = {
      source  = "hashicorp/vault"
      version = "~> 3.21"
    }
  }
}
