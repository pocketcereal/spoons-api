# Terraform Cleanup

**Date:** 2026-03-18
**Scope:** Harden and clean up the GCP single-VM Terraform deployment

## Changes

### 1. Restrict SSH access
Add `var.ssh_source_ranges` (list of CIDRs, default `["0.0.0.0/0"]`). The SSH firewall rule uses this instead of hardcoded `0.0.0.0/0`. Set your IP in tfvars to lock it down.

### 2. Pin container image tag
Add `var.image_tag` (default `"latest"`). The docker-compose template interpolates `ghcr.io/pocketcereal/spoons-api:${image_tag}` instead of hardcoded `latest`.

### 3. Consolidate templates
The startup script currently generates config files inline with heredocs, duplicating content that also exists in `templates/`. Refactor so:
- `templates/docker-compose.prod.yml` becomes a proper Terraform template with `${image_tag}` interpolation
- `templates/config.yaml` uses `${podcast_index_api_key}` / `${podcast_index_api_secret}` interpolation (matching what the startup script already does)
- `templates/Caddyfile` stays static (no variables needed)
- The startup script writes these rendered templates to disk instead of using heredocs
- `.env` stays inline in the startup script (simple key-value, not worth a template file)

### 4. Fix API healthcheck
Change docker-compose healthcheck from `spoons-api start --help` (validates CLI, not API) to `curl -sf http://localhost:4000/healthz` (confirms API is serving). Add `curl` to the API container or use wget.
