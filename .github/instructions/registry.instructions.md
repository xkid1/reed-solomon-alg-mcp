---
description: Rules for MCP Registry metadata and container packaging.
applyTo: "{server.json,Dockerfile,.github/workflows/publish.yml}"
---

- `server.json` `name` is `io.github.<xkid1>/reed-solomon-alg-mcp` and must match the Dockerfile `LABEL io.modelcontextprotocol.server.name`.
- `version` must equal the `Cargo.toml` version; the OCI tag must equal it too.
- The registry does not accept crates.io packages: distribute as OCI (`ghcr.io/...`) or MCPB.
- The container image and GHCR package must be public.
- Never commit tokens. The workflow authenticates with GitHub OIDC (`id-token: write`).
