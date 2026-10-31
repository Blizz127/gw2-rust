#!/bin/bash
# Run the GW2 sandbox. ALWAYS --network none: the client never reaches ArenaNet.
#   sandbox/run.sh build                  build the image (needs network, no client)
#   sandbox/run.sh preflight              isolation + Wine + lavapipe + DXVK check
#   sandbox/run.sh selftest               build gw2-login-observe, run it inside the
#                                         isolated container, send a scripted STS
#                                         Connect over loopback, check the capture
#   sandbox/run.sh client <build> [args]  run the client from the WRITABLE working
#                                         copy /home/blizz/gw2-sandbox/client-work-<build>
#                                         (a copy of the frozen /home/blizz/gw2-client/
#                                         build-<build>; the frozen baseline is never
#                                         mounted). CPUS=0-15 limits cores (default).
#                                         Runs as the host user. Requires `prepare`.
#   sandbox/run.sh prepare <build>        after the working copy exists: verify it
#                                         against the frozen baseline, then do the
#                                         launcher's own swap (Gw2-64.new.exe ->
#                                         Gw2-64.exe; the old Gw2-64.exe is the
#                                         installer bootstrap, kept as Gw2Setup-64.exe)
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
  prepare)
    build="${1:?build id required}"
    src="$STATE/client-work-$build"; base="/home/blizz/gw2-client/build-$build"
    [[ -f "$src/Gw2-64.new.exe" && -f "$src/Gw2.dat" ]] || { echo "working copy incomplete"; exit 2; }
    [[ -f "$src/.prepared" ]] && { echo "already prepared"; exit 0; }
    [[ "$(stat -c %s "$src/Gw2.dat")" == "$(stat -c %s "$base/Gw2.dat")" ]] || { echo "Gw2.dat size differs from baseline (copy still running?)"; exit 2; }
    cmp -s "$src/Gw2-64.new.exe" "$base/Gw2-64.new.exe" || { echo "Gw2-64.new.exe differs from baseline"; exit 2; }
    cmp -s "$src/Gw2-64.exe" "$src/Gw2Setup-64.exe" || { echo "Gw2-64.exe is not the installer bootstrap; refusing to replace it"; exit 2; }
    rm "$src/Gw2-64.exe"
    mv "$src/Gw2-64.new.exe" "$src/Gw2-64.exe"
    sha256sum "$src/Gw2-64.exe" | tee "$src/.prepared"
    ;;
  client)
    build="${1:?client build id required}"; shift
    src="$STATE/client-work-$build"
    [[ -f "$src/Gw2-64.exe" ]] || { echo "no client at $src (working copy missing or not prepared)"; exit 2; }
    [[ -w "$src" ]] || { echo "$src is not writable; refusing to mount a frozen baseline"; exit 2; }
    [[ -f "$src/.prepared" ]] || { echo "run: $0 prepare $build (Gw2-64.exe is still the installer bootstrap)"; exit 2; }
    mkdir -p "$STATE/home"
    mkdir -p "$STATE/prefix" "$STATE/logs"
    exec docker run --rm --network none --name "gw2-sandbox-$build" \
      --user "$(id -u):$(id -g)" -e HOME=/sandbox/home -v "$STATE/home:/sandbox/home" \
      --cpuset-cpus "${CPUS:-0-15}" -v "$src:/client" -v "$STATE/prefix:/sandbox/prefix" -v "$STATE/logs:/sandbox/logs" \
      -v "$HERE/../target/debug:/server:ro" -e SERVER="${SERVER:-}" \
      "$IMAGE" client "$@" ;;
  *) echo "usage: $0 build|preflight|selftest|prepare <build>|client <build> [args]"; exit 2 ;;
esac
