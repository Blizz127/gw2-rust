#!/bin/bash
# Run the GW2 sandbox. ALWAYS --network none: the client never reaches ArenaNet.
#   sandbox/run.sh build                  build the image (needs network, no client)
#   sandbox/run.sh preflight              isolation + Wine + lavapipe + DXVK check
#   sandbox/run.sh client <build> [args]  run the frozen client from
#                                         /home/blizz/gw2-client/<build> (read-only)
#   env SERVER="gw2-login-observe --log /sandbox/logs/capture" starts a workspace
#   binary from target/debug (mounted read-only at /server) before the client.
set -euo pipefail
IMAGE=gw2-rust-sandbox:latest
HERE="$(cd "$(dirname "$0")" && pwd)"
STATE=/home/blizz/gw2-sandbox
mode="${1:-preflight}"; shift || true
case "$mode" in
  build) exec docker build -t "$IMAGE" "$HERE" ;;
  preflight)
    exec docker run --rm --network none --name gw2-sandbox-preflight "$IMAGE" preflight ;;
  client)
    build="${1:?client build id required}"; shift
    src="/home/blizz/gw2-client/$build"
    [[ -f "$src/Gw2-64.exe" ]] || { echo "no client at $src"; exit 2; }
    mkdir -p "$STATE/prefix" "$STATE/logs"
    exec docker run --rm --network none --name "gw2-sandbox-$build" \
      -v "$src:/client:ro" -v "$STATE/prefix:/sandbox/prefix" -v "$STATE/logs:/sandbox/logs" \
      -v "$HERE/../target/debug:/server:ro" -e SERVER="${SERVER:-}" \
      "$IMAGE" client "$@" ;;
  *) echo "usage: $0 build|preflight|client <build> [args]"; exit 2 ;;
esac
