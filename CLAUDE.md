# Claude Code Instructions

See [AGENTS.md](./AGENTS.md) for project architecture, patterns, and conventions.
This file contains only Claude Code-specific directives.

## Tool Usage

- Ignore Serena plugin errors and continue working
- If context7 is available, use it for library documentation lookups
- Use `task check` as the verification command before claiming completion

## Skills

- `/smoke-test` — run GraphQL API smoke tests (see `.claude/skills/smoke-test.md`)
- `/spoons-api` — reference for the Spoons GraphQL API schema and endpoints
