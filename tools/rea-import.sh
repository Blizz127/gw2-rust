#!/bin/bash
# Cold REA/Ghidra import of the FROZEN Gw2-64.exe, run in the background so no MCP
# client deadline cuts it off. Produces a snapshot that MCP `open_binary` reuses
# (snapshot_path), plus the analyze overview JSON. Raw evidence stays private.
#
#   tools/rea-import.sh /home/blizz/gw2-client/build-209026/Gw2-64.exe
#
# Refuses non-frozen paths (anything under incoming/ or arenanet-download/, or a
# writable file). It never runs the client.
set -euo pipefail
exe="${1:?usage: rea-import.sh <frozen Gw2-64.exe>}"
case "$exe" in */incoming/*|*/arenanet-download/*) echo "refusing: not a frozen copy"; exit 2;; esac
[[ "$exe" = /* && -f "$exe" ]] || { echo "need an absolute path to an existing file"; exit 2; }
[[ ! -w "$exe" ]] || { echo "refusing: $exe is writable (run tools/freeze-client.sh first)"; exit 2; }

export PATH="$HOME/.local/bin:$PATH"
export GHIDRA_INSTALL_DIR=/home/blizz/re-studio/ghidra_12.1.4_PUBLIC
export JAVA_HOME=/home/blizz/.local/opt/jdk25
export REA_GHIDRA_STARTUP_TIMEOUT_MS="${REA_GHIDRA_STARTUP_TIMEOUT_MS:-3600000}"

notes="$HOME/re/gw2/notes"
mkdir -p "$notes"
sha="$(sha256sum "$exe" | cut -d' ' -f1)"
tag="gw2-64-${sha:0:12}"
snap="$notes/$tag.snapshot"
log="$notes/$tag.import.log"
echo "exe:      $exe"
echo "sha256:   $sha"
echo "snapshot: $snap"
echo "log:      $log"
[[ ! -e "$snap" ]] || { echo "snapshot already exists; reuse it with open_binary snapshot_path"; exit 0; }

start=$(date +%s)
nice -n 10 rea analyze "$exe" --provider ghidra --snapshot "$snap" --format json \
  > "$notes/$tag.analyze.json" 2> "$log"
echo "done in $(( $(date +%s) - start ))s; overview: $notes/$tag.analyze.json"
