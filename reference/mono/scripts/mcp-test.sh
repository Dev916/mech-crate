#!/usr/bin/env bash
set -euo pipefail

# Simple MCP stdio client helpers for mono mcp
# Usage examples:
#   ./scripts/mcp-test.sh init
#   ./scripts/mcp-test.sh tools
#   ./scripts/mcp-test.sh ps | jq
#   ./scripts/mcp-test.sh describe connect | jq
#   ./scripts/mcp-test.sh call exec '{"service":"connect","command":"ls -la"}' | jq

_send() {
  local req="$1"
  printf 'Content-Length: %d\r\n\r\n%s' "$(printf '%s' "$req" | wc -c | awk '{print $1}')" "$req" \
    | mono mcp \
    | awk 'NR>2'
}

init() {
  _send '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}'
}

tools() {
  _send '{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}'
}

ps() {
  _send '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"ps","arguments":{}}}'
}

describe() {
  local svc="${1:-}"
  if [[ -z "$svc" ]]; then echo "usage: $0 describe <service>" >&2; exit 1; fi
  _send "{\"jsonrpc\":\"2.0\",\"id\":4,\"method\":\"tools/call\",\"params\":{\"name\":\"services/describe\",\"arguments\":{\"service\":\"$svc\"}}}"
}

call() {
  local name="${1:-}"
  local args_json="${2:-{}}"
  if [[ -z "$name" ]]; then echo "usage: $0 call <name> [args_json]" >&2; exit 1; fi
  _send "{\"jsonrpc\":\"2.0\",\"id\":5,\"method\":\"tools/call\",\"params\":{\"name\":\"$name\",\"arguments\":$args_json}}"
}

case "${1:-}" in
  init) shift; init "$@" ;;
  tools) shift; tools "$@" ;;
  ps) shift; ps "$@" ;;
  describe) shift; describe "$@" ;;
  call) shift; call "$@" ;;
  *)
    cat >&2 <<USAGE
Usage: $0 <command> [args]
Commands:
  init                     Initialize the MCP session
  tools                    List available MCP tools
  ps                       Show running services (parsed by server)
  describe <service>       Describe a service (urls, structure, paths)
  call <name> [args_json]  Call an MCP tool with optional JSON args
USAGE
    exit 1
  ;;
esac


