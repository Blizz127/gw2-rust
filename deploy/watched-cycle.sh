#!/bin/bash
# Watched sleep -> wake cycle for the GW2 server on Banshee, run from the devbox.
#
#   deploy/watched-cycle.sh            # dry run: read-only checks, prints the plan
#   deploy/watched-cycle.sh --execute  # really sleeps, switches the kit link, wakes
#
# Only run --execute after launcher b106 (PLAY wake-and-wait) is live, the owner's
# first b107 (Wine 10) device run is done, and DIT is watching. It uses the exact
# argv stackd has pinned (r57), so it tests what stackd will run:
#   1. read-only: health ready, humans == 0, target kit exists and its SHA256SUMS verify
#   2. sleep: gw2-sleep.sh (refuses unless humans == 0)
#   3. while stopped: gw2-rust-current -> the target kit (health's exe check needs the
#      switch to happen with no process running)
#   4. wake: systemctl --user start, then poll health until ready; prints start->ready
#   On any failure after step 2 it says what state Banshee is in; the old link target is
#   printed so it can be restored by hand.
set -euo pipefail
HOST=blizz@100.74.44.12
KIT=${KIT:-/home/blizz/gw2-rust-5825dfb}
STATE=/home/blizz/gw2-rust-state/run
PROBE="$KIT/deploy/gw2-probe.py"
execute=0; [[ ${1:-} == --execute ]] && execute=1
remote() { ssh -o BatchMode=yes -o ConnectTimeout=10 "$HOST" "$@"; }
ts() { date -u +%H:%M:%SZ; }

echo "[$(ts)] read-only checks"
remote "test -d '$KIT' && cd '$KIT' && sha256sum --quiet -c SHA256SUMS" || { echo "target kit missing or SHA256SUMS mismatch"; exit 2; }
health=$(remote "sg blizz -c \"exec /usr/bin/python3 -I $PROBE health $STATE\"") || { echo "not ready before the cycle: $health"; exit 2; }
humans=$(remote "sg blizz -c \"exec /usr/bin/python3 -I $PROBE humans $STATE\"") || { echo "human count unreadable: $humans"; exit 2; }
old=$(remote "readlink -f /home/blizz/gw2-rust-current")
lines=$(remote "wc -l < $STATE/server.log")
echo "health=$health humans=$humans link=$old server.log lines=$lines"
[[ $(python3 -c 'import json,sys; print(json.loads(sys.argv[1])["humans"])' "$humans") == 0 ]] || { echo "someone is on; not cycling"; exit 1; }
if [[ $execute == 0 ]]; then
  echo "dry run: would sleep via gw2-sleep.sh, link gw2-rust-current: $old -> $KIT, start, poll health"
  exit 0
fi

echo "[$(ts)] sleep"
remote "sg blizz -c \"exec /bin/bash $KIT/deploy/gw2-sleep.sh $PROBE $STATE\""
stopped=$(remote "XDG_RUNTIME_DIR=/run/user/1000 systemctl --user is-active gw2-server.service" || true)
[[ $stopped != active ]] || { echo "unit still active after sleep; stopping here (link unchanged: $old)"; exit 3; }
asleep=$(remote "sg blizz -c \"exec /usr/bin/python3 -I $PROBE health $STATE\"" || true)
echo "asleep: unit=$stopped health=$asleep"

echo "[$(ts)] switch link while stopped"
remote "ln -sfn '$KIT' /home/blizz/gw2-rust-current.new && mv -Tf /home/blizz/gw2-rust-current.new /home/blizz/gw2-rust-current && readlink -f /home/blizz/gw2-rust-current"

echo "[$(ts)] wake"
t0=$(date +%s.%N)
remote "sg blizz -c \"exec /usr/bin/env XDG_RUNTIME_DIR=/run/user/1000 /usr/bin/systemctl --user start gw2-server.service\""
# Fail fast: a crash-looping kit never gets ready, so 60 polls (~1-2 min with ssh) is enough.
for _ in $(seq 60); do
  if out=$(remote "sg blizz -c \"exec /usr/bin/python3 -I $PROBE health $STATE\"" 2>/dev/null); then
    printf '[%s] ready after %.1f s: %s\n' "$(ts)" "$(echo "$(date +%s.%N) - $t0" | bc)" "$out"
    remote "python3 -I $PROBE humans $STATE; tail -n +$((lines + 1)) $STATE/server.log | tail -5"
    exit 0
  fi
  sleep 1
done
echo "[$(ts)] NOT ready after start; rolling back to $old"
remote "export XDG_RUNTIME_DIR=/run/user/1000; systemctl --user stop gw2-server.service; ln -sfn '$old' /home/blizz/gw2-rust-current.new && mv -Tf /home/blizz/gw2-rust-current.new /home/blizz/gw2-rust-current; systemctl --user reset-failed gw2-server.service 2>/dev/null; systemctl --user start gw2-server.service" || true
for _ in $(seq 60); do
  if out=$(remote "sg blizz -c \"exec /usr/bin/python3 -I $PROBE health $STATE\"" 2>/dev/null); then echo "[$(ts)] rolled back, ready: $out"; exit 4; fi
  sleep 1
done
echo "[$(ts)] ROLLBACK NOT READY: check Banshee now"; exit 5
