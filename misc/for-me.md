# reed-solomon-mcp

An MCP server (Rust, stdio, one dependency: `serde_json`) that helps with Reed-Solomon error-correcting codes over GF(2^8)
(primitive polynomial 0x11d, alpha = 2, roots alpha^0..alpha^(nsym-1)).

## Tools
| Tool | Purpose |
|---|---|
| `rs_encode` | Systematic encode; returns codeword + parity |
| `rs_decode` | Berlekamp-Massey + Chien + Forney; corrects up to nsym/2 byte errors, reports positions/magnitudes |
| `rs_syndromes` | Syndromes / validity check |
| `rs_generator_polynomial` | g(x) coefficients for a given parity count |
| `rs_code_parameters` | RS(n,k) capacity, rate, overhead, min distance |
| `rs_inject_errors` | Corrupt bytes to test decoding |
| `gf256_math` | add, mul, div, pow, inverse, log, exp |

Limits: n <= 255 bytes. Errors-only decoding (erasure decoding is not implemented).

## Build / run
```
cargo test
cargo build --release
./target/release/reed-solomon-mcp
```
Claude Desktop config:
```json
{ "mcpServers": { "reed-solomon": { "command": "/path/to/reed-solomon-mcp" } } }
```

## Publish to the MCP Registry
The registry hosts metadata only and does not support crates.io packages, so this project ships as an OCI image on GHCR.

1. Replace `xkid1` in `Cargo.toml`, `server.json`, `Dockerfile` (lowercase it for the image path).
2. Push the repo to GitHub: `github.com/xkid1/reed-solomon-alg-mcp`.
3. Build and push the image (the `LABEL io.modelcontextprotocol.server.name` in the Dockerfile must match the `name` in `server.json`):
   ```
   docker build -t ghcr.io/xkid1/reed-solomon-alg-mcp:0.1.0 .
   echo $GITHUB_PAT | docker login ghcr.io -u xkid1 --password-stdin
   docker push ghcr.io/xkid1/reed-solomon-alg-mcp:0.1.0
   ```
   Make the GHCR package public.
4. Install the publisher: `brew install mcp-publisher` (or download from github.com/modelcontextprotocol/registry/releases).
5. `mcp-publisher login github` (follow the device code flow), then `mcp-publisher publish`.
6. Verify: `curl "https://registry.modelcontextprotocol.io/v0/servers?search=io.github.xkid1/reed-solomon-alg-mcp"`

Or just push a `v0.1.0` tag and let `.github/workflows/publish.yml` do steps 3-5 via GitHub OIDC.
If the schema URL in `server.json` is out of date, run `mcp-publisher init` to see the current one.
