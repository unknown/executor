ui           = true
api_addr     = "http://IP_ADDRESS:8200"
cluster_addr = "https://IP_ADDRESS:8201"

storage "consul" {
  address = "127.0.0.1:8500"
  token   = "CONSUL_VAULT_TOKEN"
  path    = "vault/"
}

listener "tcp" {
  address         = "0.0.0.0:8200"
  cluster_address = "0.0.0.0:8201"
  tls_disable     = 1
}
