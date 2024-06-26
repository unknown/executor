output "lb_address_consul_nomad" {
  value = "http://${hcloud_server.server[0].ipv4_address}"
}

output "consul_token_secret" {
  value = random_uuid.nomad_token.result
}

output "IP_Addresses" {
  value = <<CONFIGURATION

Client public IPs: ${join(", ", hcloud_server.client[*].ipv4_address)}

Server public IPs: ${join(", ", hcloud_server.server[*].ipv4_address)}

The Consul UI can be accessed at http://${hcloud_server.server[0].ipv4_address}:8500/ui
with the token: ${random_uuid.nomad_token.result}
CONFIGURATION
}
