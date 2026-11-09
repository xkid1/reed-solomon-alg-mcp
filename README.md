<div align="center">

# reed-solomon-mcp

**An MCP server that encodes, decodes, and explains Reed-Solomon error-correcting codes.**

[![CI](https://img.shields.io/github/actions/workflow/status/xkid1/reed-solomon-alg-mcp/ci.yml?branch=main&label=CI&logo=githubactions&logoColor=white)](https://github.com/xkid1/reed-solomon-alg-mcp/actions/workflows/ci.yml)
[![Build](https://img.shields.io/github/actions/workflow/status/xkid1/reed-solomon-alg-mcp/build.yml?branch=main&label=Build&logo=githubactions&logoColor=white)](https://github.com/xkid1/reed-solomon-alg-mcp/actions/workflows/build.yml)
[![Test](https://img.shields.io/github/actions/workflow/status/xkid1/reed-solomon-alg-mcp/test.yml?branch=main&label=Test&logo=githubactions&logoColor=white)](https://github.com/xkid1/reed-solomon-alg-mcp/actions/workflows/test.yml)
[![Publish](https://img.shields.io/github/actions/workflow/status/xkid1/reed-solomon-alg-mcp/publish.yml?branch=main&label=Publish&logo=githubactions&logoColor=white)](https://github.com/xkid1/reed-solomon-alg-mcp/actions/workflows/publish.yml)
[![Action](https://img.shields.io/github/actions/workflow/status/xkid1/reed-solomon-alg-mcp/action.yml?branch=main&label=Action&logo=githubactions&logoColor=white)](https://github.com/xkid1/reed-solomon-alg-mcp/actions/workflows/action.yml)
[![Code Conventions](https://img.shields.io/github/actions/workflow/status/xkid1/reed-solomon-alg-mcp/code-conventions.yml?branch=main&label=Conventions&logo=githubactions&logoColor=white)](https://github.com/xkid1/reed-solomon-alg-mcp/actions/workflows/code-conventions.yml)
[![PR Labeler](https://img.shields.io/github/actions/workflow/status/xkid1/reed-solomon-alg-mcp/pr-labeler.yml?branch=main&label=Labeler&logo=githubactions&logoColor=white)](https://github.com/xkid1/reed-solomon-alg-mcp/actions/workflows/pr-labeler.yml)
[![PR Welcome](https://img.shields.io/github/actions/workflow/status/xkid1/reed-solomon-alg-mcp/pr-welcome.yml?branch=main&label=Welcome&logo=githubactions&logoColor=white)](https://github.com/xkid1/reed-solomon-alg-mcp/actions/workflows/pr-welcome.yml)
[![Made with Rust](https://img.shields.io/badge/Made%20with-Rust-orange?logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/github/license/xkid1/reed-solomon-alg-mcp)](LICENSE)
[![MCP Registry](https://img.shields.io/badge/MCP%20Registry-io.github.xkid1%2Freed--solomon--alg--mcp-blue)](https://registry.modelcontextprotocol.io/v0/servers?search=io.github.xkid1/reed-solomon-alg-mcp)
[![Docker image](https://img.shields.io/badge/ghcr.io-reed--solomon--alg--mcp-2496ED?logo=docker&logoColor=white)](https://github.com/users/xkid1/packages/container/package/reed-solomon-alg-mcp)
[![Listed on mcpservers.org](https://mcpservers.org/badge.svg)](https://mcpservers.org/servers/xkid1/reed-solomon-alg-mcp)
[![Dependabot Updates](https://github.com/xkid1/reed-solomon-alg-mcp/actions/workflows/dependabot/dependabot-updates/badge.svg)](https://github.com/xkid1/reed-solomon-alg-mcp/actions/workflows/dependabot/dependabot-updates)
[![M8ven Score](https://m8ven.ai/badge/mcp/xkid1-reed-solomon-alg-mcp-1lbk09)](https://m8ven.ai/mcp/xkid1-reed-solomon-alg-mcp-1lbk09?s=readme)

</div>

A small [Model Context Protocol](https://modelcontextprotocol.io) server written in Rust. It speaks JSON-RPC over stdio, has a single dependency (`serde_json`), and implements Reed-Solomon over GF(2^8) with primitive polynomial `0x11d`, alpha = 2, and generator roots alpha^0 to alpha^(nsym-1).

## Features
- Systematic encoding with any number of parity bytes (codeword length up to 255)
- Error location and correction with Berlekamp-Massey, Chien search and Forney
- Syndrome checks, generator polynomials and GF(256) arithmetic
- Test helpers that corrupt chosen bytes so you can watch decoding work

## Tools
| Tool | Purpose |
|---|---|
| `rs_encode` | Systematic encode; returns codeword and parity |
| `rs_decode` | Corrects up to `nsym/2` byte errors; reports positions and magnitudes |
| `rs_syndromes` | Computes syndromes and tells you if a word is a valid codeword |
| `rs_generator_polynomial` | Generator polynomial coefficients for a parity count |
| `rs_code_parameters` | RS(n,k) capacity, rate, overhead and minimum distance |
| `rs_inject_errors` | Corrupts bytes at given positions for testing |
| `gf256_math` | add, mul, div, pow, inverse, log, exp in GF(256) |

## Quick start: use the published Docker image
The server is published in the MCP Registry as `io.github.xkid1/reed-solomon-alg-mcp` and ships as a public image on GHCR. You only need Docker.

```bash
docker pull ghcr.io/xkid1/reed-solomon-alg-mcp:0.1.0
```

### VS Code (agent mode)
Add to `.vscode/mcp.json` (or your user MCP config):
```json
{
  "servers": {
    "reed-solomon": {
      "type": "stdio",
      "command": "docker",
      "args": ["run", "-i", "--rm", "ghcr.io/xkid1/reed-solomon-alg-mcp:0.1.0"]
    }
  }
}
```

### Claude Desktop
Add to `claude_desktop_config.json`:
```json
{
  "mcpServers": {
    "reed-solomon": {
      "command": "docker",
      "args": ["run", "-i", "--rm", "ghcr.io/xkid1/reed-solomon-alg-mcp:0.1.0"]
    }
  }
}
```
Use `-i`, not `-t`: the transport needs a clean stdin/stdout pipe.

### MCP Inspector
```bash
npx @modelcontextprotocol/inspector docker run -i --rm ghcr.io/xkid1/reed-solomon-alg-mcp:0.1.0
```

### Try it
Ask your assistant: *"Encode 'hello' with 8 parity bytes, corrupt 3 bytes, then decode it."*

Or talk to the server by hand:
```bash
printf '%s\n' \
  '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"t","version":"1"}}}' \
  '{"jsonrpc":"2.0","method":"notifications/initialized"}' \
  '{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"rs_encode","arguments":{"data":"Hello","parity_symbols":6}}}' \
  | docker run -i --rm ghcr.io/xkid1/reed-solomon-alg-mcp:0.1.0
```
The encoded codeword is `48656c6c6fc0886617750e`.

## Build from source
Requires a recent stable Rust toolchain.
```bash
git clone https://github.com/xkid1/reed-solomon-alg-mcp.git
cd reed-solomon-alg-mcp
cargo build --release
./target/release/reed-solomon-mcp
```
Point a client at the binary instead of Docker:
```json
{ "mcpServers": { "reed-solomon": { "command": "/absolute/path/to/target/release/reed-solomon-mcp" } } }
```

## Development
```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo build --release && bash scripts/smoke-test.sh
```
Every pull request to `main` runs tests and builds first, then the code-convention checks. See [CONTRIBUTING.md](CONTRIBUTING.md).

## Limitations
- Codeword length is at most 255 bytes (GF(256)).
- Errors-only decoding: erasure decoding is not implemented.
- Beyond `parity/2` errors the decoder may return an error or silently produce a different valid codeword. This is inherent to Reed-Solomon, so add an outer checksum if you need to detect that.
- Not a cryptographic tool: error correction gives no confidentiality or authenticity.

## Project
- [Contributing](CONTRIBUTING.md)
- [Code of Conduct](CODE_OF_CONDUCT.md)
- [Security Policy](SECURITY.md)
- [License](LICENSE) (MIT)
