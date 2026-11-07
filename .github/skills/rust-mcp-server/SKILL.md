---
name: rust-mcp-server
description: Build, test, and debug a dependency-light stdio MCP server in Rust (JSON-RPC 2.0 framing, tool schemas, isError semantics, end-to-end smoke tests). Use when adding or changing tools in this repo or diagnosing client connection problems.
---

# Rust stdio MCP server

## Layout

- `src/rs.rs`: pure Reed-Solomon logic, unit-tested.
- `src/main.rs`: read line, parse JSON, dispatch, write one JSON line.

## Smoke test

```bash
cargo build --release
printf '%s\n' \
 '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"t","version":"1"}}}' \
 '{"jsonrpc":"2.0","method":"notifications/initialized"}' \
 '{"jsonrpc":"2.0","id":2,"method":"tools/list"}' \
 '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"rs_encode","arguments":{"data":"Hello","parity_symbols":6}}}' \
 | ./target/release/reed-solomon-mcp
```

Expect three responses (the notification gets none).

## Common problems

| Symptom                                  | Cause                                                           |
| ---------------------------------------- | --------------------------------------------------------------- |
| Client hangs on connect                  | Something printed non-JSON to stdout; move logs to stderr       |
| Tool shows as failed but message is fine | Return `isError: true` in a normal result, not a JSON-RPC error |
| Tool missing in client                   | Not listed in `tools()`, or invalid `inputSchema`               |
| Blank response                           | Request had no `id` (treated as notification)                   |

## Adding a tool

Schema in `tools()`, arm in `call_tool()`, test request, README row. Validate all arguments and return `Err(String)`.
