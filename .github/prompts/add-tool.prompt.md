---
description: Add a new MCP tool to the server end to end.
agent: agent
tools: ["codebase", "editFiles", "runCommands"]
---

Add a new MCP tool named `${input:toolName}` that does: ${input:behavior}.

Steps:

1. Implement the logic in `src/rs.rs` if it is coding math, with unit tests (including a failure case).
2. Add the JSON schema to `tools()` and a match arm in `call_tool()` in `src/main.rs`.
3. Add a row to the Tools table in `README.md`.
4. Run `cargo test` and an end-to-end stdio `tools/call` for the new tool.
5. Summarize what changed and show the sample request/response.
