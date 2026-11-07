---
name: mcp-registry-publish
description: Publish this Rust MCP server to the official MCP Registry as an OCI image on GHCR using mcp-publisher, including server.json, name verification, versioning, and troubleshooting. Use when preparing a release or fixing a failed publish.
---

# Publishing to the MCP Registry

The registry stores metadata only. It supports npm, PyPI, NuGet, OCI and MCPB packages, not crates.io, so ship the Rust binary as an OCI image.

## Release steps

1. Replace `xkid1` everywhere (lowercase for image paths).
2. Align versions: `Cargo.toml`, `server.json` `version`, OCI tag.
3. `cargo test && docker build -t ghcr.io/<user>/reed-solomon-alg-mcp:<version> .`
4. `docker push ...` and set the GHCR package to public.
5. `mcp-publisher login github` (device flow), then `mcp-publisher publish`.
6. Verify: `curl "https://registry.modelcontextprotocol.io/v0/servers?search=io.github.xkid1/reed-solomon-alg-mcp"`

CI alternative: push tag `v<version>`; `.github/workflows/publish.yml` builds, pushes and publishes via GitHub OIDC.

## Name verification

- Namespace `io.github.xkid1/...` requires GitHub login as that user.
- For OCI the image must carry `LABEL io.modelcontextprotocol.server.name="<same name as server.json>"`.

## Troubleshooting

| Error                       | Fix                                                           |
| --------------------------- | ------------------------------------------------------------- |
| Namespace permission denied | Log in as the GitHub user matching `io.github.xkid1`          |
| Label/name mismatch         | Rebuild the image with the correct LABEL and push again       |
| Image not found             | Package is private or tag differs from `server.json`          |
| Schema validation failed    | Run `mcp-publisher init` to get the current schema and fields |
| Version already exists      | Versions are immutable; bump and republish                    |
