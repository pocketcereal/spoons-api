resource "google_compute_instance" "spoons_api" {
  name         = "spoons-api"
  machine_type = "e2-medium"
  zone         = "us-central1-a"
  tags         = ["http-server", "ssh-server"]

  boot_disk {
    initialize_params {
      image = "ubuntu-os-cloud/ubuntu-2404-lts-amd64"
      size  = 20
      type  = "pd-standard"
    }
  }

  network_interface {
    subnetwork = google_compute_subnetwork.subnet.id
    access_config {}
  }

  metadata_startup_script = templatefile("startup.sh.tpl", {
    database_url             = var.database_url
    podcast_index_api_key    = var.podcast_index_api_key
    podcast_index_api_secret = var.podcast_index_api_secret
    ghcr_user                = var.ghcr_user
    ghcr_token               = var.ghcr_token
    supabase_url             = var.supabase_url
    jwt_secret               = var.jwt_secret
    docker_compose = templatefile("templates/docker-compose.prod.yml", {
      image_tag = var.image_tag
    })
    caddyfile = file("templates/Caddyfile")
    config_yaml = templatefile("templates/config.yaml", {
      podcast_index_api_key    = var.podcast_index_api_key
      podcast_index_api_secret = var.podcast_index_api_secret
    })
  })
}
