# AGENTS.md

Guidance for AI coding agents working in this repository.

## Project overview
`reed-solomon-mcp` is a Rust MCP (Model Context Protocol) server that encodes, decodes, and explains Reed-Solomon codes over GF(2^8). It talks JSON-RPC 2.0 over stdio and has one dependency, `serde_json`. It is published to the MCP Registry as an OCI image on GHCR.

## Repository layout
| Path | Purpose |
|---|---|
| `src/rs.rs` | Pure Reed-Solomon logic: GF(256), encoder, syndromes, Berlekamp-Massey, Chien search, Forney. Unit-tested. |
| `src/main.rs` | MCP stdio server: JSON-RPC framing, `tools()` schemas, `call_tool()` handlers. |
| `server.json` | MCP Registry metadata. |
| `Dockerfile` | Builds the OCI image; carries the registry ownership label. |
| `.github/agents/` | Custom agents (developer, reviewer, unit-test writer, registry publisher). |
| `.github/instructions/` | Path-scoped instructions (Rust, testing, MCP protocol, registry). |
| `.github/prompts/` | Reusable prompts (add-tool, erasure-decoding, release, explain-rs, write-unit-tests, test-mcp-tool, review-test-gaps). |
| `.github/skills/` | Skills (reed-solomon, rust-mcp-server, rust-unit-testing, mcp-registry-publish). |
| `.github/workflows/publish.yml` | Tag-triggered build and registry publish. |

## Commands
```bash
cargo build --release          # build
cargo test                     # unit tests (must pass before any commit)
cargo test every_single        # run tests matching a name filter
cargo clippy --all-targets -- -D warnings   # lint tests too
cargo fmt --check              # formatting
cargo clippy -- -D warnings    # lints
./target/release/reed-solomon-mcp   # run server (reads stdin, writes stdout)
```

End-to-end smoke test:
```bash
printf '%s\n' \
 '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"t","version":"1"}}}' \
 '{"jsonrpc":"2.0","method":"notifications/initialized"}' \
 '{"jsonrpc":"2.0","id":2,"method":"tools/list"}' \
 | ./target/release/reed-solomon-mcp
```
Expect two responses; the notification gets none.

## Conventions
- Rust 2021, builds on 1.75+. No `unsafe`. No `unwrap()`/`expect()` on anything derived from user input; return `Result<_, String>`.
- Do not add dependencies without asking. The single-dependency design is intentional.
- **stdout is the protocol channel.** Write only JSON-RPC lines to stdout; log to stderr with `eprintln!`.
- Tool execution failures return a normal result with `"isError": true`. Only protocol problems (unknown method, unknown tool) use JSON-RPC `error`.
- Reed-Solomon conventions, do not change silently:
  - Primitive polynomial `0x11d`, alpha = 2, generator roots alpha^0..alpha^(nsym-1).
  - Polynomials are highest-degree first; syndrome vector is `[0, S0..S(nsym-1)]`.
  - Systematic encoding: message bytes followed by parity bytes. Codeword length is at most 255.
- Capacity rule: `2*errors + erasures <= parity_symbols`. Errors-only decoding is currently implemented; erasures are not.

## Testing
- Unit tests live in `#[cfg(test)] mod tests` at the bottom of the file under test (`src/rs.rs` currently has 7).
- Every public routine needs a happy path, a boundary, an invalid-input case, and an over-capacity case.
- Assert exact values (positions, magnitudes, bytes), not just `is_ok()`.
- Tests are deterministic: use the seeded `Lcg` helper in `src/rs.rs`; no system time, unseeded randomness, network, or new dev-dependencies.
- Beyond `parity_symbols/2` errors the decoder may return `Err` or miscorrect to another valid codeword. Assert `Err(_)` or `codeword != original`, never that it always errors.
- Never weaken an assertion to make a test pass. If a test exposes a real bug, report the failing input and fix the code.
- For new MCP tools also run the stdio smoke test and confirm failures return `isError: true` and nothing but JSON lines reaches stdout.

## Adding or changing a tool
1. Put coding math in `src/rs.rs` with a unit test, including a failure case (too many errors).
2. Add the schema to `tools()` and a match arm in `call_tool()` in `src/main.rs`.
3. Add a row to the Tools table in `README.md`.
4. Run `cargo test` and the stdio smoke test with a `tools/call` for the new tool.

## Releasing
- Keep three versions identical: `Cargo.toml` `version`, `server.json` `version`, and the OCI tag in `server.json` `packages[0].identifier`.
- The Dockerfile label `io.modelcontextprotocol.server.name` must equal `server.json` `name` (`io.github.<lowercase-username>/reed-solomon-mcp`).
- Replace every `YOUR_GITHUB_USERNAME` placeholder before publishing.
- The MCP Registry does not accept crates.io packages; distribute as OCI or MCPB.
- Agents cannot authenticate for the user: print the `docker push`, `mcp-publisher login github`, and `mcp-publisher publish` commands and let the user run them.
- Never commit tokens. CI authenticates with GitHub OIDC.

## Specialized guidance
Use the matching custom agent or skill instead of improvising:
- Extending the server: agent `rs-rust-developer`, skill `rust-mcp-server`.
- Reviewing changes: agent `rs-code-reviewer`.
- Writing or fixing tests: agent `rs-unit-test-writer`, skill `rust-unit-testing`, prompts `write-unit-tests`, `test-mcp-tool`, `review-test-gaps`.
- Reed-Solomon theory or decoder bugs: skill `reed-solomon`.
- Publishing: agent `mcp-registry-publisher`, skill `mcp-registry-publish`.

## Definition of done
- `cargo test`, `cargo fmt --check`, and `cargo clippy --all-targets -- -D warnings` all pass.
- New or changed behavior has tests that follow the Testing section.
- Smoke test output is valid JSON-RPC with nothing extra on stdout.
- README and `server.json` are updated if tools, versions, or names changed.

## Known limitations
- Errors-only decoding. More than `parity_symbols/2` errors may be rejected or may miscorrect to a different valid codeword; use an outer CRC if detection matters.
- Codewords are limited to 255 bytes (GF(256)).
