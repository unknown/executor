variable "nomad_jwks_url" {
  description = "The URL pointing to Nomad's JWKS information. It must be reachable by all Vault servers and should resolve to multiple Nomad agents for high availability."
  type        = string
}
