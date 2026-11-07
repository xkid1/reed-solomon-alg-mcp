#!/usr/bin/env bash
# Smoke-test the MCP server over stdio. Requires jq.
#   bash scripts/smoke-test.sh                       # uses target/release/reed-solomon-mcp
#   bash scripts/smoke-test.sh docker run -i --rm IMAGE   # any command that starts the server
set -euo pipefail

if [ "$#" -gt 0 ]; then CMD=("$@"); else CMD=(target/release/reed-solomon-mcp); fi
fail() { echo "FAIL: $*" >&2; exit 1; }
ok()   { echo "ok: $*"; }
call() { printf '%s\n' "$1" | "${CMD[@]}"; }
req()  { printf '{"jsonrpc":"2.0","id":%s,"method":"tools/call","params":{"name":"%s","arguments":%s}}' "$1" "$2" "$3"; }

INIT='{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"smoke","version":"1"}}}'
NOTE='{"jsonrpc":"2.0","method":"notifications/initialized"}'
LIST='{"jsonrpc":"2.0","id":2,"method":"tools/list"}'

# 1. Protocol: three requests with ids, one notification -> exactly three JSON lines.
out=$(printf '%s\n' "$INIT" "$NOTE" "$LIST" "$(req 3 rs_encode '{"data":"Hello","parity_symbols":6}')" | "${CMD[@]}")
[ "$(printf '%s\n' "$out" | wc -l)" -eq 3 ] || fail "expected 3 response lines (notification must get none)"
printf '%s\n' "$out" | jq -e . >/dev/null || fail "stdout contains non-JSON output"
ok "3 JSON-RPC responses, nothing else on stdout"

# 2. Tools are listed.
n=$(printf '%s\n' "$out" | jq 'select(.id==2) | .result.tools | length')
[ "$n" -eq 7 ] || fail "expected 7 tools, got $n"
ok "7 tools listed"

# 3. Known encoding.
cw=$(printf '%s\n' "$out" | jq -r 'select(.id==3) | .result.structuredContent.codeword_hex')
[ "$cw" = "48656c6c6fc0886617750e" ] || fail "unexpected codeword: $cw"
ok "rs_encode(Hello, 6) = $cw"

# 4. Corrupt 3 bytes, decode, expect the original back.
bad=$(call "$(req 11 rs_inject_errors "{\"codeword\":\"$cw\",\"encoding\":\"hex\",\"positions\":[0,4,9]}")" | jq -r '.result.structuredContent.corrupted_codeword_hex')
[ "$bad" != "$cw" ] || fail "inject_errors did not change the codeword"
dec=$(call "$(req 12 rs_decode "{\"codeword\":\"$bad\",\"encoding\":\"hex\",\"parity_symbols\":6}")")
[ "$(echo "$dec" | jq -r '.result.structuredContent.message_hex')" = "48656c6c6f" ] || fail "decode did not recover the message"
[ "$(echo "$dec" | jq -c '.result.structuredContent.error_positions')" = "[0,4,9]" ] || fail "wrong error positions"
ok "3 corrupted bytes located and corrected"

# 5. Over capacity (4 errors, 6 parity): must not crash. Either an error or a different codeword.
bad4=$(call "$(req 13 rs_inject_errors "{\"codeword\":\"$cw\",\"encoding\":\"hex\",\"positions\":[0,1,2,3]}")" | jq -r '.result.structuredContent.corrupted_codeword_hex')
over=$(call "$(req 14 rs_decode "{\"codeword\":\"$bad4\",\"encoding\":\"hex\",\"parity_symbols\":6}")")
echo "$over" | jq -e . >/dev/null || fail "no valid JSON for over-capacity decode"
if [ "$(echo "$over" | jq -r '.result.isError')" = "false" ]; then
  [ "$(echo "$over" | jq -r '.result.structuredContent.corrected_codeword_hex')" != "$cw" ] || fail "recovered beyond capacity?"
fi
ok "over-capacity input handled without crashing"

# 6. Invalid input returns isError, not a crash.
err=$(call "$(req 15 rs_encode '{"data":"x"}')")
[ "$(echo "$err" | jq -r '.result.isError')" = "true" ] || fail "missing argument should give isError:true"
err=$(call "$(req 16 rs_encode '{"data":"x","parity_symbols":300}')")
[ "$(echo "$err" | jq -r '.result.isError')" = "true" ] || fail "oversized codeword should give isError:true"
ok "invalid input returns isError:true"

echo "All smoke checks passed."
