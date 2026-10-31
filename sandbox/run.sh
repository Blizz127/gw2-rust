#!/bin/bash
# Run the GW2 sandbox. ALWAYS --network none: the client never reaches ArenaNet.
#   sandbox/run.sh build                  build the image (needs network, no client)
#   sandbox/run.sh preflight              isolation + Wine + lavapipe + DXVK check
#   sandbox/run.sh selftest               build gw2-login-observe, run it inside the
#                                         isolated container, send a scripted STS
#                                         Connect over loopback, check the capture
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
  selftest)
    (cd "$HERE/.." && cargo build -q -p gw2-login --bin gw2-login-observe)
    exec docker run --rm --network none --name gw2-sandbox-selftest \
      -v "$HERE/../target/debug:/server:ro" --entrypoint bash "$IMAGE" -c '
        set -euo pipefail
        ifs=$(awk -F: "NR>2 {gsub(/ /,\"\",\$1); print \$1}" /proc/net/dev | grep -v "^lo$" || true)
        [[ -z "$ifs" ]] || { echo "FAIL: not isolated ($ifs)"; exit 1; }
        /server/gw2-login-observe --listen 127.0.0.1:6112 --log /tmp/cap >/tmp/obs.out 2>&1 &
        for _ in $(seq 50); do (exec 3<>/dev/tcp/127.0.0.1/6112) 2>/dev/null && break; sleep 0.1; done
        body="<Request><ConnType>400</ConnType><ProductName>gw2</ProductName></Request>"
        printf "P /Sts/Connect STS/1.0\r\ns:1\r\nl:%d\r\n\r\n%s" "${#body}" "$body" > /dev/tcp/127.0.0.1/6112
        for _ in $(seq 50); do grep -qs "closed after" /tmp/cap/*.log && break; sleep 0.1; done
        cat /tmp/cap/*.log
        grep -q "## +.*STS" /tmp/cap/*.log && grep -q "<ProductName>gw2</ProductName>" /tmp/cap/*.log \
          && echo "selftest: ok" || { echo "FAIL: capture missing"; exit 1; }' ;;
  client)
    build="${1:?client build id required}"; shift
    src="/home/blizz/gw2-client/$build"
    [[ -f "$src/Gw2-64.exe" ]] || { echo "no client at $src"; exit 2; }
    mkdir -p "$STATE/prefix" "$STATE/logs"
    exec docker run --rm --network none --name "gw2-sandbox-$build" \
      -v "$src:/client:ro" -v "$STATE/prefix:/sandbox/prefix" -v "$STATE/logs:/sandbox/logs" \
      -v "$HERE/../target/debug:/server:ro" -e SERVER="${SERVER:-}" \
      "$IMAGE" client "$@" ;;
  *) echo "usage: $0 build|preflight|selftest|client <build> [args]"; exit 2 ;;
esac
