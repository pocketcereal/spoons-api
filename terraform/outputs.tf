output "instance_ip" {
  description = "External IP of the spoons-api instance — set DNS A record for spoons.pocketcereal.com to this"
  value       = google_compute_instance.spoons_api.network_interface[0].access_config[0].nat_ip
}
