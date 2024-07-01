job "executor-backend" {
  datacenters = ["dc1"]
  type = "service"

  group "executor-backend" {
    count = 3

    network {
      port "http" {
        to = "3000"
      }
    }

    service {
      name = "backend"
      port = "http"

      check {
        type     = "http"
        path     = "/"
        interval = "2s"
        timeout  = "2s"
      }
    }

    task "backend" {
      driver = "docker"

      config {
        image = "dmo1010/executor-backend:0.1.0"
        ports = ["http"]
      }

      vault {}

      template {
        data        = <<EOF
NOMAD_ADDR  = "http://10.0.2.11:4646"
{{with secret "secret/data/default/executor-backend/config"}}
NOMAD_TOKEN={{.Data.data.nomad_token}}
{{end}}
EOF
        destination = "${NOMAD_SECRETS_DIR}/env"
        env         = true
      }
    }
  }
}
