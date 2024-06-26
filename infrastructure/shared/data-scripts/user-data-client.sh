#!/bin/bash

set -e

exec > >(sudo tee /var/log/user-data.log|logger -t user-data -s 2>/dev/console) 2>&1
sudo bash /ops/shared/scripts/client.sh '${retry_join}' "${network_interface}" "${nomad_binary}"

NOMAD_HCL_PATH="/etc/nomad.d/nomad.hcl"
CONSULCONFIGDIR=/etc/consul.d

sed -i "s/CONSUL_TOKEN/${nomad_consul_token_secret}/g" $NOMAD_HCL_PATH

# Add auto-join token to consul agent for dns and start consul
sed -i "s/AGENT_TOKEN/${nomad_consul_token_secret}/g" $CONSULCONFIGDIR/consul.hcl
sudo systemctl start consul.service

sudo systemctl restart nomad

echo "Finished client setup"
