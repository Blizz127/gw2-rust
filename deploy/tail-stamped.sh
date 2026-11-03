#!/bin/bash
# Record Banshee's GW2 server.log from now on, each line stamped with UTC arrival time,
# for lining up with the launcher's gw2-runner.log during an owner device run.
#
#   deploy/tail-stamped.sh [OUT_DIR]     (default /home/blizz/.cache/gw2-owner-logs)
#
# server.log itself has no timestamps. Arrival over ssh is within a second or so of the
# write. Prints the baseline line count first. Stop with Ctrl-C (or kill); reconnects if
# ssh drops. Read-only on Banshee.
set -uo pipefail
HOST=blizz@100.74.44.12
LOG=/home/blizz/gw2-rust-state/run/server.log
out_dir=${1:-/home/blizz/.cache/gw2-owner-logs}
umask 077
mkdir -p "$out_dir"
base=$(ssh -o BatchMode=yes -o ConnectTimeout=10 "$HOST" "wc -l < $LOG") || { echo "cannot reach Banshee"; exit 2; }
out="$out_dir/banshee-server-$(date -u +%Y%m%dT%H%M%SZ).log"
echo "baseline: server.log line $base at $(date -u +%FT%TZ); recording to $out"
echo "# baseline line $base at $(date -u +%FT%TZ)" > "$out"
next=$((base + 1))
# On exit, stop the ssh tail too (it is in this script's process group).
trap 'trap - EXIT INT TERM; kill 0 2>/dev/null' EXIT INT TERM
while true; do
  ssh -o BatchMode=yes -o ServerAliveInterval=30 "$HOST" "tail -n +$next -F $LOG" 2>/dev/null |
    while IFS= read -r line; do printf '%s %s\n' "$(date -u +%FT%TZ)" "$line"; done >> "$out"
  # ssh dropped: resume after the lines already recorded.
  next=$((base + 1 + $(grep -vc '^#' "$out")))
  sleep 2
done
