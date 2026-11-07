# Security Policy

## Supported versions
| Version | Supported |
|---|---|
| 0.1.x | Yes |
| older | No |

Only the latest published release receives security fixes.

## Reporting a vulnerability
Please **do not open a public issue** for security problems.

Report privately with [GitHub private vulnerability reporting](https://github.com/xkid1/reed-solomon-alg-mcp/security/advisories/new). Include:
- What you found and why it matters
- Steps or input that reproduce it (for example a JSON-RPC request)
- The version or image tag you tested
- Any suggested fix

You can expect an acknowledgement within about 7 days and an update on the fix within about 30 days. Reporters are credited in the advisory unless they prefer otherwise.

## Scope
In scope:
- Crashes, panics, hangs or unbounded memory use triggered by crafted input to any tool
- Incorrect output that could cause silent data corruption
- Anything written to stdout that breaks the MCP stream
- Problems in the Docker image, build files or workflows

Out of scope:
- Reed-Solomon decoders cannot detect every uncorrectable error: beyond `parity/2` errors a decoder may return an error or a different valid codeword. This is a property of the code, not a bug. Use an outer checksum if you need detection.
- This project is not a cryptographic tool and makes no confidentiality or integrity claims.

## Supply chain
- Releases are published as an image on GHCR and listed in the MCP Registry under `io.github.xkid1/reed-solomon-alg-mcp`.
- The only Rust dependency is `serde_json`. Dependabot keeps dependencies and GitHub Actions up to date.
