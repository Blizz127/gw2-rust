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
#                                         SANDBOX_LOG_DIR selects a separate run's
#                                         log/fixture directory (default STATE/logs).
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
      -v "${SERVER_BIN:-$HERE/../target/debug}:/server:ro" --entrypoint bash "$IMAGE" -c '
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
  restore)
    # Replace the working copy's Gw2.dat with the frozen baseline. Needed after the client
    # trims or rewrites its archive (e.g. MultiArchiveTrim drops assets no manifest lists).
    build="${1:?build id required}"
    src="$STATE/client-work-$build"; base="/home/blizz/gw2-client/build-$build"
    docker ps --format '{{.Names}}' | grep -qx "gw2-sandbox-$build" && { echo "client container still running"; exit 2; }
    cp "$base/Gw2.dat" "$src/Gw2.dat.restore" && chmod 644 "$src/Gw2.dat.restore" && mv -f "$src/Gw2.dat.restore" "$src/Gw2.dat"
    cmp -n 1048576 "$src/Gw2.dat" "$base/Gw2.dat" && echo "restored $src/Gw2.dat from $base"
    ;;
  client)
    build="${1:?client build id required}"; shift
    src="$STATE/client-work-$build"
    [[ -f "$src/Gw2-64.exe" ]] || { echo "no client at $src (working copy missing or not prepared)"; exit 2; }
    [[ -w "$src" ]] || { echo "$src is not writable; refusing to mount a frozen baseline"; exit 2; }
    [[ -f "$src/.prepared" ]] || { echo "run: $0 prepare $build (Gw2-64.exe is still the installer bootstrap)"; exit 2; }
    mkdir -p "$STATE/home"
    # WINE_RUNTIME=proton uses the devbox's GE-Proton (read-only) with its own prefix.
    runtime_args=(); prefix="$STATE/prefix"
    if [[ "${WINE_RUNTIME:-system}" == proton ]]; then
      proton_dir="${PROTON_DIR:-/home/blizz/.local/share/Steam/compatibilitytools.d/GE-Proton11-6-x86_64}"
      [[ -x "$proton_dir/files/bin/wine" ]] || { echo "no Proton wine at $proton_dir"; exit 2; }
      runtime_args=(-v "$proton_dir:/opt/proton:ro" -e WINE=/opt/proton/files/bin/wine)
      prefix="$STATE/prefix-proton"
    fi
    mkdir -p "$prefix"
    logs="${SANDBOX_LOG_DIR:-$STATE/logs}"
    [[ "$logs" == /* ]] || { echo "SANDBOX_LOG_DIR must be absolute"; exit 2; }
    mkdir -p "$STATE/prefix" "$logs"
    # ADD_HOSTS="name[=ip] ..." maps extra names via /etc/hosts (default ip 127.0.0.1; use with DNS_SINK=0).
    host_args=(); for h in ${ADD_HOSTS:-}; do
      ip=127.0.0.1; [[ $h == *=* ]] && { ip=${h#*=}; h=${h%%=*}; }
      [[ $ip == 127.* ]] || { echo "ADD_HOSTS: $h -> $ip is not loopback"; exit 2; }
      host_args+=(--add-host "$h:$ip"); done
    exec docker run --rm --network none --name "gw2-sandbox-$build" \
      --user "$(id -u):$(id -g)" -e HOME=/sandbox/home -v "$STATE/home:/sandbox/home" \
      --dns 127.0.0.1 --dns-search . -e DNS_SINK="${DNS_SINK:-1}" -e DNS_ANSWER="${DNS_ANSWER:-127.1.0.1}" -e DNS_MAP="${DNS_MAP:-}" -e WINEDEBUG="${WINEDEBUG:--all}" -e GDB_STACKDUMP="${GDB_STACKDUMP:-0}" -e GDB_SCRIPT="${GDB_SCRIPT:-}" -e UI_CMD="${UI_CMD:-}" -e UI_DELAY="${UI_DELAY:-60}" \
      --hostname gw2-sandbox --add-host gw2-sandbox:127.0.0.1 "${host_args[@]}" \
      --cpuset-cpus "${CPUS:-0-15}" -v "$src:/client" -v "$prefix:/sandbox/prefix" "${runtime_args[@]}" -v "$logs:/sandbox/logs" \
      -v "${SERVER_BIN:-$HERE/../target/debug}:/server:ro" -e SERVER="${SERVER:-}" \
      "$IMAGE" client "$@" ;;
  *) echo "usage: $0 build|preflight|selftest|prepare <build>|restore <build>|client <build> [args]"; exit 2 ;;
esac
