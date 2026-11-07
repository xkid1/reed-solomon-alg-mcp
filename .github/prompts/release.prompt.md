---
description: Prepare a new release and MCP Registry publication.
agent: mcp-registry-publisher
---

Prepare release `${input:version}`:

1. Bump the version in `Cargo.toml`, `server.json` (both `version` and the OCI tag).
2. Confirm no `YOUR_GITHUB_USERNAME` placeholders remain.
3. Run `cargo test` and `cargo build --release`.
4. Print the exact `docker build`, `docker push`, `mcp-publisher login github`, and `mcp-publisher publish` commands for me to run, and the git tag command to trigger the workflow.
