packer {
  required_plugins {
    hcloud = {
      source  = "github.com/hetznercloud/hcloud"
      version = "~> 1"
    }
  }
}

locals {
  timestamp = regex_replace(timestamp(), "[- TZ:]", "")
}

variable "region" {
  type = string
}

source "hcloud" "hashistack" {
  snapshot_name = "hashistack-${local.timestamp}"
  image         = "ubuntu-22.04"
  server_type   = "cpx11"
  ssh_username = "root"
  location      = var.region
}

build {
  sources = ["source.hcloud.hashistack"]

  provisioner "shell" {
    inline = ["sudo mkdir -p /ops/shared", "sudo chmod 777 -R /ops"]
  }

  provisioner "file" {
    destination = "/ops"
    source      = "./shared"
  }

  provisioner "shell" {
    environment_vars = ["INSTALL_NVIDIA_DOCKER=FALSE"]
    script           = "./shared/scripts/setup.sh"
  }
}
