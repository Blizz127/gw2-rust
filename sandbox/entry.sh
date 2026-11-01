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
      # 127.1.0.1, not 127.0.0.x: the client's StsLib lookup recurses forever on 127.0.0.0/24
      # when the host has only loopback (docs/re/209026-connect-path.md).
      /server/gw2-dns-observe --answer "${DNS_ANSWER:-127.1.0.1}" --log /sandbox/logs/dns.log >/sandbox/logs/dns.out 2>&1 &
      echo "dns sinkhole started (all names -> ${DNS_ANSWER:-127.1.0.1}, log /sandbox/logs/dns.log)"
    fi
    if [[ -n "${SERVER:-}" ]]; then
      # SERVER may hold several commands separated by ';' (each a binary under /server).
      IFS=';' read -ra cmds <<<"$SERVER"; n=0
      for c in "${cmds[@]}"; do
        read -ra cmd <<<"$c"; [[ ${#cmd[@]} -gt 0 ]] || continue; n=$((n+1))
        "/server/${cmd[0]}" "${cmd[@]:1}" >"/sandbox/logs/server$n.log" 2>&1 &
        echo "server $n started: ${cmd[*]}"
      done
      sleep 1
    fi
    # Screenshot the virtual display every ${SHOT_EVERY:-20}s into /sandbox/logs/shots/.
    mkdir -p /sandbox/logs/shots
    ( i=0; while sleep "${SHOT_EVERY:-20}"; do i=$((i+1));
        xwd -root -silent 2>/dev/null | xwdtopnm 2>/dev/null | pnmtopng > "/sandbox/logs/shots/shot-$(printf %03d $i).png" 2>/dev/null || true
      done ) &
    # UI_CMD: a shell command run in the background after UI_DELAY seconds (default 60),
    # e.g. xdotool input for a test login against our own server. Never real credentials.
    if [[ -n "${UI_CMD:-}" ]]; then
      ( sleep "${UI_DELAY:-60}"; bash -c "$UI_CMD" >/sandbox/logs/ui.log 2>&1 ) &
    fi
    cd /client
    if [[ -n "${GDB_SCRIPT:-}" ]]; then
      # Debug aid: run the client under gdb with a caller-supplied script (it must end in `run`).
      exec gdb -q -batch -x "$GDB_SCRIPT" --args /usr/lib/wine/wine64 /client/Gw2-64.exe "$@" \
        2> >(tee /sandbox/logs/wine.err >&2)
    fi
    if [[ "${GDB_STACKDUMP:-0}" == 1 ]]; then
      # Debug aid: run the client under gdb; when Wine reports a fatal "stack overflow",
      # dump that thread's PE stack (STACK_LO..STACK_HI) for offline return-address analysis.
      cat > /sandbox/logs/stackdump.gdb <<GDB
set pagination off
set confirm off
set disable-randomization off
set follow-fork-mode parent
set detach-on-fork on
handle all nostop noprint pass
set breakpoint pending on
# matches "XXXX:err:virtual:virtual_setup_exception stack ov..." ("err:virt" at +5, "stack ov" at +41)
break write if \$rdi == 2 && \$rdx > 49 && *(unsigned long long *)(\$rsi + 5) == 0x747269763a727265 && *(unsigned long long *)(\$rsi + 41) == 0x766f206b63617473
commands
  silent
  printf "GDB: stack overflow write in thread %d\\n", \$_thread
  dump binary memory /sandbox/logs/stack-dump.bin ${STACK_LO:-0x7c0000} ${STACK_HI:-0x8c0000}
  printf "GDB: dumped ${STACK_LO:-0x7c0000}-${STACK_HI:-0x8c0000}\\n"
  continue
end
run
GDB
      exec gdb -q -batch -x /sandbox/logs/stackdump.gdb --args /usr/lib/wine/wine64 /client/Gw2-64.exe "$@" \
        2> >(tee /sandbox/logs/wine.err >&2)
    fi
    exec "$WINE" /client/Gw2-64.exe "$@" 2> >(tee /sandbox/logs/wine.err >&2)
    ;;
  *) echo "unknown mode $mode"; exit 2 ;;
esac
