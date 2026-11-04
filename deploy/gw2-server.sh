#!/bin/bash
# GW2 build-209026 server supervisor (always-on, stackd readiness_only).
#
#   gw2-server.sh CONFIG_FILE
#
# CONFIG_FILE (shell assignments) sets:
#   GW2_BIN      directory with gw2-assetcdn, gw2-login-observe, gw2-game-observe
#   GW2_DATA     fixture directory (see deploy/README.md)
#   GW2_ID       private server identity from tools/gw2-runner/make-identity.py
#   GW2_RUN      runtime directory for pids.json and agreement-version (0700; persistent:
#                the account's accepted User Agreement version, owner decision 2026-11-04)
#   GW2_BIND     tailnet address to bind, e.g. 100.74.44.12 (never 0.0.0.0)
#   GW2_SPAWN    X,Y,Z placeholder spawn in map units
#   GW2_POPULATION_DELAY_MS  optional diagnostic hold between client 420 and the
#                population (default 0). A 15 s hold showed (2026-11-04 03:58Z) that the
#                MODELS_STREAM texture crash followed the population, not map assets.
# Ports: 6112 game, 6113 auth, 6114 login, 6115 asset CDN. Every server runs
# --no-capture: nothing typed by a player is stored or printed. If any child
# exits, all are stopped and the script exits non-zero (systemd restarts it).
set -euo pipefail
config=${1:?usage: gw2-server.sh CONFIG_FILE}
# shellcheck source=/dev/null
source "$config"
: "${GW2_BIN:?}" "${GW2_DATA:?}" "${GW2_ID:?}" "${GW2_RUN:?}" "${GW2_BIND:?}" "${GW2_SPAWN:?}"
[[ $GW2_BIND != 0.0.0.0 && $GW2_BIND != :: ]] || { echo "refusing unspecified bind address" >&2; exit 2; }
umask 077
mkdir -p "$GW2_RUN"
pids=()
stop() { for p in "${pids[@]}"; do kill "$p" 2>/dev/null || true; done; }
trap 'stop; exit 143' TERM INT
trap stop EXIT

"$GW2_BIN/gw2-assetcdn" --listen "$GW2_BIND:6115" --latest "$(cat "$GW2_DATA/latest.txt")" \
  --files "$GW2_DATA/cdn-files" --log /dev/stderr &
pids+=($!); cdn=$!
"$GW2_BIN/gw2-login-observe" --no-capture --listen "$GW2_BIND:6114" \
  --tls-key "$GW2_ID/cligate-101.key.pem" --tls-cert "$GW2_ID/cligate-101.cert.der" \
  --answer-gethostname cligate.101.ncplatform.net. \
  --reply-xml "/Auth/StartSsoLogin=$GW2_DATA/startsso.xml" \
  --reply-xml "/Account/ListMyGameAccounts=$GW2_DATA/accounts.xml" \
  --reply-xml "/Auth/RequestGameToken=$GW2_DATA/token.xml" \
  --reply-xml "/Auth2f/Check=$GW2_DATA/auth2f.xml" \
  --reply-xml "/Auth/GetUserInfo=$GW2_DATA/empty.xml" \
  --reply-xml "/Mail/SetOptIn=$GW2_DATA/empty.xml" \
  --reply-xml "/Presence/SetWatch=$GW2_DATA/empty.xml" \
  --reply-xml "/Game.com.Wallet/SetWatch=$GW2_DATA/empty.xml" \
  --reply-default "$GW2_DATA/empty.xml" &
pids+=($!); login=$!
# Route 20 hands the client the bubble-side game address; the launcher relay
# maps it to $GW2_BIND:6112.
"$GW2_BIN/gw2-game-observe" --no-capture --listen "$GW2_BIND:6113" \
  --params "$GW2_ID/dh-params.bin" --private "$GW2_ID/dh-private.bin" --log "$GW2_RUN/unused" \
  --reply-frame "$GW2_DATA/auth-ready.bin" --auth-reply-body "$GW2_DATA/auth-account-body.bin" \
  --auth-agreement-state "$GW2_RUN/agreement-version" \
  --auth-character-record "$GW2_DATA/character-record.bin" --auth-play-route 127.1.0.4:6112 &
pids+=($!); auth=$!
"$GW2_BIN/gw2-game-observe" --no-capture --listen "$GW2_BIND:6112" --mission-preamble \
  --params "$GW2_ID/dh-params.bin" --private "$GW2_ID/dh-private.bin" --log "$GW2_RUN/unused" \
  --mission-map-key 15 --mission-population "$GW2_DATA/character-record.bin" \
  --mission-spawn "$GW2_SPAWN" --mission-population-delay-ms "${GW2_POPULATION_DELAY_MS:-0}" &
pids+=($!); game=$!

printf '{"bind":"%s","processes":[{"role":"cdn","pid":%d,"port":6115,"exe":"%s"},{"role":"login","pid":%d,"port":6114,"exe":"%s"},{"role":"auth","pid":%d,"port":6113,"exe":"%s"},{"role":"game","pid":%d,"port":6112,"exe":"%s"}]}\n' \
  "$GW2_BIND" "$cdn" "$GW2_BIN/gw2-assetcdn" "$login" "$GW2_BIN/gw2-login-observe" \
  "$auth" "$GW2_BIN/gw2-game-observe" "$game" "$GW2_BIN/gw2-game-observe" > "$GW2_RUN/pids.json.tmp"
mv "$GW2_RUN/pids.json.tmp" "$GW2_RUN/pids.json"
status=0
wait -n || status=$?
echo "a GW2 server process exited (status $status); stopping the rest" >&2
exit 1
