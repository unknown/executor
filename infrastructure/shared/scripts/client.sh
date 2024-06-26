#!/bin/bash

set -e

CONFIGDIR=/ops/shared/config

CONSULCONFIGDIR=/etc/consul.d
NOMADCONFIGDIR=/etc/nomad.d
CONSULTEMPLATECONFIGDIR=/etc/consul-template.d

# Wait for network
sleep 15

DOCKER_BRIDGE_IP_ADDRESS=(`ip -brief addr show docker0 | awk '{print $3}' | awk -F/ '{print $1}'`)
RETRY_JOIN=$1
NETWORK_INTERFACE=$2
NOMAD_BINARY=$3

# Get IP from metadata service
IP_ADDRESS=$(curl http://169.254.169.254/hetzner/v1/metadata/public-ipv4)

# Consul
sed -i "s/IP_ADDRESS/$IP_ADDRESS/g" $CONFIGDIR/consul_client.hcl
sed -i "s/RETRY_JOIN/$RETRY_JOIN/g" $CONFIGDIR/consul_client.hcl
sudo cp $CONFIGDIR/consul_client.hcl $CONSULCONFIGDIR/consul.hcl

sudo systemctl enable consul.service

# Nomad

## Replace existing Nomad binary if remote file exists
if [[ `wget -S --spider $NOMAD_BINARY  2>&1 | grep 'HTTP/1.1 200 OK'` ]]; then
  curl -L $NOMAD_BINARY > nomad.zip
  sudo unzip -o nomad.zip -d /usr/local/bin
  sudo chmod 0755 /usr/local/bin/nomad
  sudo chown root:root /usr/local/bin/nomad
fi

sed -i "s/NETWORK_INTERFACE/$NETWORK_INTERFACE/g" $CONFIGDIR/nomad_client.hcl
sudo cp $CONFIGDIR/nomad_client.hcl $NOMADCONFIGDIR/nomad.hcl

# Install and link CNI Plugins to support Consul Connect-Enabled jobs
sudo apt install -y containernetworking-plugins
sudo mkdir /opt/cni && sudo  ln -s /usr/lib/cni /opt/cni/bin

sudo systemctl enable nomad.service
sudo systemctl start nomad.service
sleep 10
export NOMAD_ADDR=http://$IP_ADDRESS:4646

# Consul Template

sudo cp $CONFIGDIR/consul-template.hcl $CONSULTEMPLATECONFIGDIR/consul-template.hcl
sudo cp $CONFIGDIR/consul-template.service /etc/systemd/system/consul-template.service

# Add hostname to /etc/hosts
echo "127.0.0.1 $(hostname)" | sudo tee --append /etc/hosts

# Add systemd-resolved configuration for Consul DNS
# ref: https://developer.hashicorp.com/consul/tutorials/networking/dns-forwarding#systemd-resolved-setup
sed -i "s/DOCKER_BRIDGE_IP_ADDRESS/$DOCKER_BRIDGE_IP_ADDRESS/g" $CONFIGDIR/consul-systemd-resolved.conf
sudo mkdir -p /etc/systemd/resolved.conf.d/
sudo cp $CONFIGDIR/consul-systemd-resolved.conf /etc/systemd/resolved.conf.d/consul.conf
sudo systemctl restart systemd-resolved

# Set env vars for tool CLIs
echo "export VAULT_ADDR=http://$IP_ADDRESS:8200" | sudo tee --append /$HOME/.bashrc
echo "export NOMAD_ADDR=http://$IP_ADDRESS:4646" | sudo tee --append /$HOME/.bashrc
echo "export JAVA_HOME=/usr/lib/jvm/java-8-openjdk-amd64/jre"  | sudo tee --append /$HOME/.bashrc
