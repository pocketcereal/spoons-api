variable "credentials_file" {
  description = "Path to GCP service account JSON key file"
  type        = string
}

variable "database_url" {
  description = "Supabase PostgreSQL connection string"
  type        = string
  sensitive   = true
}

variable "podcast_index_api_key" {
  description = "PodcastIndex API key"
  type        = string
  sensitive   = true
}

variable "podcast_index_api_secret" {
  description = "PodcastIndex API secret"
  type        = string
  sensitive   = true
}

variable "ghcr_user" {
  description = "GitHub Container Registry username"
  type        = string
}

variable "ghcr_token" {
  description = "GitHub Container Registry PAT for pulling images"
  type        = string
  sensitive   = true
}

variable "supabase_url" {
  description = "Supabase project URL (for JWKS auth)"
  type        = string
}

variable "jwt_secret" {
  description = "JWT fallback secret"
  type        = string
  sensitive   = true
}

variable "jamendo_client_id" {
  description = "Jamendo API client ID"
  type        = string
  sensitive   = true
}

variable "ssh_source_ranges" {
  description = "CIDR ranges allowed to SSH into the instance"
  type        = list(string)
  default     = ["0.0.0.0/0"]
}

variable "image_tag" {
  description = "Docker image tag for spoons-api (e.g. 'latest', 'v1.2.3', 'abc1234')"
  type        = string
  default     = "latest"
}
