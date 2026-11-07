---
description: Add and run a stdio end-to-end test for an MCP tool.
agent: agent
tools: ["codebase", "editFiles", "runCommands"]
---

For the tool `${input:toolName}`:

1. Add unit tests for its handler path (valid args, missing arg, wrong type, out-of-range value).
2. Build with `cargo build --release` and send `initialize`, `notifications/initialized`, then a `tools/call` over stdio (see the `rust-mcp-server` skill).
3. Confirm: success returns `isError: false` with `structuredContent`; bad input returns `isError: true` with a readable message; nothing but JSON lines appears on stdout.
4. Paste the requests and trimmed responses in your summary.
