#!/bin/bash
# Sandbox entrypoint. Modes:
#   preflight          check isolation, Wine, Vulkan (lavapipe), DXVK; no client needed
#   client [args...]   start Xvfb, prepare the prefix, run /client/Gw2-64.exe [args]
#                      SERVER="<cmd under /server>" starts our server first, inside
#                      the same isolated network (e.g. SERVER="gw2-login-observe
#                      --log /sandbox/logs/capture").
set -euo pipefail
WINE="${WINE:-wine}"

isolated() {
  # Pass only if no interface other than lo exists and an outbound connect fails.
  local ifs
  ifs=$(awk -F: 'NR>2 {gsub(/ /,"",$1); print $1}' /proc/net/dev | grep -v '^lo$' || true)
  [[ -z "$ifs" ]] || { echo "NOT ISOLATED: interfaces: $ifs"; return 1; }
  if timeout 3 bash -c 'exec 3<>/dev/tcp/1.1.1.1/443' 2>/dev/null; then
    echo "NOT ISOLATED: outbound TCP succeeded"; return 1
  fi
  echo "isolation: ok (lo only, outbound unreachable)"
}

start_x() {
  Xvfb :99 -screen 0 1280x720x24 -nolisten tcp >/sandbox/logs/xvfb.log 2>&1 &
  for _ in $(seq 50); do xdpyinfo >/dev/null 2>&1 && return 0; sleep 0.1; done
  echo "Xvfb did not start"; return 1
}

prepare_prefix() {
  if [[ ! -f "$WINEPREFIX/.dxvk-installed" ]]; then
    "$WINE" wineboot -i >/sandbox/logs/wineboot.log 2>&1 || true
    local sys="$WINEPREFIX/drive_c/windows/system32"
    for dll in d3d11 dxgi d3d10core d3d9; do
      cp "/opt/dxvk/x64/$dll.dll" "$sys/$dll.dll"
      "$WINE" reg add 'HKCU\Software\Wine\DllOverrides' /v "$dll" /d native /f >/dev/null 2>&1
    done
    touch "$WINEPREFIX/.dxvk-installed"
  fi
}

mkdir -p /sandbox/logs
mode="${1:-preflight}"; shift || true
isolated
case "$mode" in
  preflight)
    "$WINE" --version
    start_x
    vk=$(vulkaninfo --summary 2>&1 || true)
    grep -E 'deviceName|driverName' <<<"$vk" | head -4
    grep -q 'llvmpipe' <<<"$vk" || { echo "lavapipe Vulkan device not found"; exit 1; }
    echo "dxvk: $(ls /opt/dxvk/x64/ | tr '\n' ' ')"
    echo "preflight: ok"
    ;;
  client)
    [[ -f /client/Gw2-64.exe ]] || { echo "no /client/Gw2-64.exe mounted"; exit 2; }
    start_x
    prepare_prefix
    if [[ "${DNS_SINK:-0}" == 1 && -x /server/gw2-dns-observe ]]; then
      /server/gw2-dns-observe --log /sandbox/logs/dns.log >/sandbox/logs/dns.out 2>&1 &
      echo "dns sinkhole started (all names -> 127.0.0.1, log /sandbox/logs/dns.log)"
    fi
    if [[ -n "${SERVER:-}" ]]; then
      read -ra cmd <<<"$SERVER"
      "/server/${cmd[0]}" "${cmd[@]:1}" >/sandbox/logs/server.log 2>&1 &
      sleep 1
      echo "server started: $SERVER"
    fi
    # Screenshot the virtual display every ${SHOT_EVERY:-20}s into /sandbox/logs/shots/.
    mkdir -p /sandbox/logs/shots
    ( i=0; while sleep "${SHOT_EVERY:-20}"; do i=$((i+1));
        xwd -root -silent 2>/dev/null | xwdtopnm 2>/dev/null | pnmtopng > "/sandbox/logs/shots/shot-$(printf %03d $i).png" 2>/dev/null || true
      done ) &
    cd /client
    exec "$WINE" /client/Gw2-64.exe "$@" 2> >(tee /sandbox/logs/wine.err >&2)
    ;;
  *) echo "unknown mode $mode"; exit 2 ;;
esac
