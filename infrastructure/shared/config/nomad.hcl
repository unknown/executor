data_dir  = "/opt/nomad/data"
bind_addr = "0.0.0.0"

# Specify network interface
advertise {
  http = "{{ GetInterfaceIP \"NETWORK_INTERFACE\" }}"
  rpc  = "{{ GetInterfaceIP \"NETWORK_INTERFACE\" }}"
  serf = "{{ GetInterfaceIP \"NETWORK_INTERFACE\" }}"
}

# Enable the server
server {
  enabled          = true
  bootstrap_expect = SERVER_COUNT
}

consul {
  address = "127.0.0.1:8500"
  token = "CONSUL_TOKEN"
}

acl {
  enabled = true
}

vault {
  enabled          = true

  default_identity {
    aud  = ["vault.io"]
    ttl  = "1h"
  }
}
