---
description: MCP stdio protocol rules for the server entrypoint.
applyTo: "src/main.rs"
---

- Transport is newline-delimited JSON-RPC 2.0 on stdin/stdout. One message per line, no pretty-printing of responses.
- stdout is reserved for protocol output. Log only to stderr (`eprintln!`).
- Handle: `initialize` (echo client `protocolVersion`), `notifications/initialized` (no reply), `ping`, `tools/list`, `tools/call`.
- Tool execution errors return a normal result with `"isError": true`; protocol errors (unknown method/tool, bad params) return a JSON-RPC `error`.
- Return both `content` (text) and `structuredContent` for successful tool calls.
- When adding a tool: add its schema in `tools()`, its arm in `call_tool()`, a row in `README.md`, and a test request in the stdio check.
