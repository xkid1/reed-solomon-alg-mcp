# Contributing

Thanks for helping improve `reed-solomon-mcp`. Please read the [Code of Conduct](CODE_OF_CONDUCT.md) first. For security problems, see [SECURITY.md](SECURITY.md) instead of opening an issue.

## Getting set up
1. Install Rust: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
2. On Linux you also need a C linker: `sudo apt install build-essential`
3. Fork and clone the repo, then:
   ```bash
   cargo test
   ```
4. Optional: Docker (for the image checks) and Node.js (for the MCP Inspector).

## Before you open a pull request
All of these must pass. CI runs the same checks.
```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test --locked
cargo build --release --locked
bash scripts/smoke-test.sh
```
Run `cargo fmt` (without `--check`) to fix formatting.

## How CI works
Pull requests to `main` run in this order:
1. **Test** and **Build** run first, in parallel.
2. **Code conventions** run after both succeed.
3. A final **CI passed** job summarizes the result. Make that the required check for merging.

## Project rules
- Keep the dependency list minimal. Please ask before adding a crate.
- No `unsafe`, and no `unwrap()` or `expect()` on values derived from user input. Return `Result<_, String>`.
- **stdout is the protocol channel.** Write only JSON-RPC lines to stdout. Log to stderr with `eprintln!`. CI fails on `println!`/`print!` in `src/`.
- Reed-Solomon conventions must not change silently: polynomial `0x11d`, alpha = 2, generator roots alpha^0..alpha^(nsym-1), highest-degree-first polynomials, codeword length at most 255.
- Tool failures return a normal result with `"isError": true`. Only protocol errors use a JSON-RPC `error`.

## Tests
- Put unit tests in `#[cfg(test)] mod tests` next to the code.
- Cover the happy path, a boundary, invalid input, and an over-capacity case.
- Assert exact values (positions, magnitudes, bytes), not just `is_ok()`.
- Keep tests deterministic: no network, system time or unseeded randomness. Use the seeded `Lcg` helper in `src/rs.rs`.
- Beyond `parity/2` errors the decoder may return `Err` or miscorrect. Assert "error or a different codeword", never "always an error".

## Adding a tool
1. Put coding math in `src/rs.rs` with tests.
2. Add its schema to `tools()` and a match arm in `call_tool()` in `src/main.rs`.
3. Add a row to the Tools table in `README.md`.
4. Extend `scripts/smoke-test.sh` if the tool changes the protocol surface.

## Commits and pull requests
- Branch from `main`: `feat/...`, `fix/...`, `docs/...`.
- Write clear commit messages. [Conventional Commits](https://www.conventionalcommits.org) are welcome (`feat:`, `fix:`, `docs:`, `test:`, `chore:`).
- Keep pull requests focused, link the issue, and describe how you tested.
- Update the README if behavior changes.

## Releases (maintainers)
Keep the version identical in `Cargo.toml`, `server.json` (`version` and the image tag). The Docker label `io.modelcontextprotocol.server.name` must equal the `name` in `server.json`, and `server.json` `description` must stay at or below 100 characters. CI checks all of this. Registry versions are permanent, so never reuse one.
