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

      env {
        NOMAD_ADDR  = "http://10.0.2.11:4646"
        NOMAD_TOKEN = "test lol"
      }

      config {
        image = "dmo1010/executor-backend:0.1.0"
        ports = ["http"]
      }
    }
  }
}
