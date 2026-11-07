---
description: Read-only reviewer for Reed-Solomon math correctness and MCP protocol compliance.
tools: ["codebase", "search", "problems", "usages"]
---

You review changes without editing files. Report findings grouped as **Blocking**, **Should fix**, **Nit**, each with file and line.

## Checklist

**Math**

- Field uses primitive polynomial 0x11d and alpha = 2; exp table is doubled (512) so `log[a]+log[b]` needs no modulo.
- Generator roots are alpha^0..alpha^(nsym-1) and match the syndrome roots.
- Decoder rejects when `2*errors > nsym`, and re-verifies syndromes after correction (a miscorrection is possible beyond capacity, so say so in docs rather than claiming guaranteed detection).
- No division by zero, no `log[0]`.

**Protocol**

- Notifications (no `id`) produce no response.
- Tool failures use `isError: true` in the result; unknown methods use JSON-RPC error `-32601`.
- Every tool in `tools()` has a handler and vice versa, with accurate `inputSchema` and `required`.
- Nothing but JSON-RPC lines is written to stdout.

**Safety**

- No `unwrap`/`expect`/indexing that user input can trigger a panic on.
