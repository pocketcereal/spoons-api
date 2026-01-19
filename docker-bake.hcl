variable "REGISTRY" {
  default = "ghcr.io"
}

variable "REPOSITORY" {
  default = "pocketcereal/spoons-api"
}

variable "TAG" {
  default = "latest"
}

variable "PLATFORMS" {
  default = ["linux/amd64", "linux/arm64"]
}

group "default" {
  targets = ["spoons-api"]
}

target "spoons-api" {
  context    = "."
  dockerfile = "Dockerfile"
  tags = [
    "${REGISTRY}/${REPOSITORY}:${TAG}",
    "${REGISTRY}/${REPOSITORY}:latest"
  ]
  platforms = PLATFORMS
  cache-from = ["type=gha"]
  cache-to   = ["type=gha,mode=max"]
}

target "spoons-api-local" {
  inherits = ["spoons-api"]
  tags     = ["spoons-api:local"]
  platforms = ["linux/amd64"]
}
