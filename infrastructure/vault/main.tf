# https://developer.hashicorp.com/nomad/docs/integrations/vault/acl#vault-acl

resource "vault_jwt_auth_backend" "nomad" {
  path               = "jwt-nomad"
  description        = "JWT auth backend for Nomad"
  jwks_url           = var.nomad_jwks_url
  jwt_supported_algs = ["RS256", "EdDSA"]
  default_role       = "nomad-workloads"
}

resource "vault_jwt_auth_backend_role" "nomad_workload" {
  backend   = vault_jwt_auth_backend.nomad.path
  role_name = "nomad-workloads"
  role_type = "jwt"

  bound_audiences = ["vault.io"]

  user_claim              = "/nomad_job_id"
  user_claim_json_pointer = true

  claim_mappings = {
    nomad_namespace = "nomad_namespace"
    nomad_job_id    = "nomad_job_id"
    nomad_group     = "nomad_group"
    nomad_task      = "nomad_task"
  }

  token_type             = "service"
  token_policies         = ["nomad-workloads"]
  token_period           = 3600
  token_explicit_max_ttl = 0
}

resource "vault_policy" "nomad_workload" {
  name   = "nomad-workloads"
  policy = <<EOT
path "secret/data/{{identity.entity.aliases.${vault_jwt_auth_backend.nomad.accessor}.metadata.nomad_namespace}}/{{identity.entity.aliases.${vault_jwt_auth_backend.nomad.accessor}.metadata.nomad_job_id}}/*" {
  capabilities = ["read"]
}

path "secret/data/{{identity.entity.aliases.${vault_jwt_auth_backend.nomad.accessor}.metadata.nomad_namespace}}/{{identity.entity.aliases.${vault_jwt_auth_backend.nomad.accessor}.metadata.nomad_job_id}}" {
  capabilities = ["read"]
}

path "secret/metadata/{{identity.entity.aliases.${vault_jwt_auth_backend.nomad.accessor}.metadata.nomad_namespace}}/*" {
  capabilities = ["list"]
}

path "secret/metadata/*" {
  capabilities = ["list"]
}
EOT
}
