#!/bin/bash
# Put the GW2 server to sleep for stackd, only when nobody is on.
#
#   gw2-sleep.sh PROBE RUN_DIR
#
# Stops gw2-server.service (systemd --user) only if `gw2-probe.py humans` reads
# exactly 0; an unreadable count or any human refuses (exit 1). The server keeps
# no player state (no accounts or characters are stored), so there is nothing to
# save first. Prints one JSON line.
# Not atomic: a player who connects between the count and the stop is dropped.
# Acceptable while only the owner plays; stackd should only call it after idle.
set -uo pipefail
probe=${1:?usage: gw2-sleep.sh PROBE RUN_DIR}
run=${2:?usage: gw2-sleep.sh PROBE RUN_DIR}
export XDG_RUNTIME_DIR=${XDG_RUNTIME_DIR:-/run/user/$(id -u)}
count=$(/usr/bin/python3 -I "$probe" humans "$run") || { echo '{"slept":false,"detail":"human count unreadable; refusing"}'; exit 1; }
humans=$(/usr/bin/python3 -I -c 'import json,sys; print(int(json.loads(sys.argv[1])["humans"]))' "$count") || { echo '{"slept":false,"detail":"bad human count; refusing"}'; exit 1; }
if [[ $humans != 0 ]]; then
  echo "{\"slept\":false,\"detail\":\"$humans human(s) connected; refusing\"}"; exit 1
fi
if /usr/bin/systemctl --user stop gw2-server.service; then
  echo '{"slept":true,"detail":"gw2-server.service stopped with 0 humans"}'
else
  echo '{"slept":false,"detail":"systemctl --user stop failed"}'; exit 1
fi
