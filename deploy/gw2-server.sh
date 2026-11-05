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
#   GW2_CHARACTER_MODE  prepared (default) or explicit saved-norn
#   GW2_CHARACTER_STORE  absolute private state path, required for saved-norn
#   GW2_POPULATION_DELAY_MS  optional diagnostic hold between client 420 and the
#                population (default 0). A 15 s hold showed (2026-11-04 03:58Z) that the
#                MODELS_STREAM texture crash followed the population, not map assets.
# Ports: 6112 game, 6113 auth, 6114 login, 6115 asset CDN. Every server runs
# --no-capture: no credentials or plaintext captures are stored or printed.
# Saved mode persists only explicitly submitted character records. If any child
# exits, all are stopped and the script exits non-zero (systemd restarts it).
set -euo pipefail
config=${1:?usage: gw2-server.sh CONFIG_FILE}
# shellcheck source=/dev/null
source "$config"
# Mode and durable state are host settings; a kit profile cannot enable them.
readonly GW2_CHARACTER_MODE="${GW2_CHARACTER_MODE:-prepared}"
readonly GW2_CHARACTER_STORE="${GW2_CHARACTER_STORE:-}"
: "${GW2_BIN:?}" "${GW2_DATA:?}" "${GW2_ID:?}" "${GW2_RUN:?}" "${GW2_BIND:?}" "${GW2_SPAWN:?}"
# A pinned kit may supply an explicit mission profile. Keeping it with the
# fixture makes switching the current kit also restore the previous profile.
if [[ -f "$GW2_DATA/mission-profile.conf" ]]; then
  source "$GW2_DATA/mission-profile.conf"
fi
[[ $GW2_BIND != 0.0.0.0 && $GW2_BIND != :: ]] || { echo "refusing unspecified bind address" >&2; exit 2; }
umask 077
auth_character_args=(--auth-character-record "$GW2_DATA/character-record.bin")
game_population_args=(--mission-population "$GW2_DATA/character-record.bin")
case "$GW2_CHARACTER_MODE" in
  prepared)
    [[ -z "$GW2_CHARACTER_STORE" ]] || { echo "prepared mode must not specify a character store" >&2; exit 2; }
    ;;
  saved-norn)
    [[ "${GW2_MAP_KEY:-}" == 379 ]] || { echo "saved-norn requires map379" >&2; exit 2; }
    : "${GW2_CHARACTER_STORE:?saved-norn requires a character store}" \
      "${GW2_MESMER_VALUES:?}" "${GW2_REWARD_TRACKS:?}" "${GW2_EQUIPMENT:?}" \
      "${GW2_SKILLS:?}" "${GW2_HEALTH:?}" "${GW2_QUEST:?}"
    python3 -I "$(dirname "$0")/gw2-saved-store.py" \
      "$GW2_CHARACTER_STORE" "$config" "$GW2_BIN" "$GW2_DATA"
    auth_character_args=(--character-store "$GW2_CHARACTER_STORE" --auth-create-route 127.1.0.4:6112)
    game_population_args=(--character-store "$GW2_CHARACTER_STORE" --creation-scene 127.1.0.4:6112)
    ;;
  *) echo "unknown GW2_CHARACTER_MODE" >&2; exit 2 ;;
esac
mission_args=()
if [[ -n "${GW2_MESMER_VALUES:-}" ]]; then
  mission_args+=(--mission-mesmer-values "$GW2_MESMER_VALUES" --mission-reward-tracks "${GW2_REWARD_TRACKS:?}")
fi
[[ -z "${GW2_EQUIPMENT:-}" ]] || mission_args+=(--mission-equipment "$GW2_DATA/$GW2_EQUIPMENT")
[[ -z "${GW2_SKILLS:-}" ]] || mission_args+=(--mission-skills "$GW2_DATA/$GW2_SKILLS")
[[ -z "${GW2_HEALTH:-}" ]] || mission_args+=(--mission-health "$GW2_HEALTH")
[[ -z "${GW2_QUEST:-}" ]] || mission_args+=(--mission-quest "$GW2_QUEST")
# Opt-in research combat loop (docs/re/209026-npc-talk-loop.md); every value
# is a sandbox-tested placeholder, not a retail encounter definition.
# GW2_HOSTILE_NPC may list several SPECIES,X,Y,Z entries separated by spaces.
for npc in ${GW2_HOSTILE_NPC:-}; do mission_args+=(--mission-hostile-npc "$npc"); done
# GW2_AMBIENT_NPC: neutral load-time NPCs, space-separated SPECIES,X,Y,Z entries.
for npc in ${GW2_AMBIENT_NPC:-}; do mission_args+=(--mission-ambient-npc "$npc"); done
[[ -z "${GW2_DAMAGE_ON_SKILL:-}" ]] || mission_args+=(--mission-damage-on-skill "$GW2_DAMAGE_ON_SKILL")
[[ -z "${GW2_AUTOATTACK_SKILL:-}" ]] || mission_args+=(--mission-autoattack-skill "$GW2_AUTOATTACK_SKILL")
[[ -z "${GW2_XP_PER_KILL:-}" ]] || mission_args+=(--mission-xp-per-kill "$GW2_XP_PER_KILL")
[[ -z "${GW2_STARTER_BAG:-}" ]] || mission_args+=(--mission-starter-bag "$GW2_STARTER_BAG")
[[ "${GW2_KILL_LOOT:-0}" != 1 ]] || mission_args+=(--mission-kill-loot)
[[ -z "${GW2_NPC_ATTACK:-}" ]] || mission_args+=(--mission-npc-attack "$GW2_NPC_ATTACK")
[[ -z "${GW2_SOURCED_DAMAGE:-}" ]] || mission_args+=(--mission-sourced-damage "$GW2_SOURCED_DAMAGE")
[[ -z "${GW2_HEART:-}" ]] || mission_args+=(--mission-heart "$GW2_HEART")
[[ "${GW2_RESPAWN:-0}" != 1 ]] || mission_args+=(--mission-respawn-flag)
# GW2_RESPAWN_POINT X,Y,Z: the defeat respawn rejoins the map there via 1065 to
# GW2_REJOIN_ROUTE (the bubble-side game address, as the auth play route).
[[ -z "${GW2_RESPAWN_POINT:-}" ]] || mission_args+=(--mission-respawn-point "$GW2_RESPAWN_POINT" --mission-rejoin-route "${GW2_REJOIN_ROUTE:-127.1.0.4:6112}")
auth_args=(--no-capture --listen "$GW2_BIND:6113"
  --params "$GW2_ID/dh-params.bin" --private "$GW2_ID/dh-private.bin" --log "$GW2_RUN/unused"
  --reply-frame "$GW2_DATA/auth-ready.bin" --auth-reply-body "$GW2_DATA/auth-account-body.bin"
  --auth-agreement-state "$GW2_RUN/agreement-version"
  "${auth_character_args[@]}" --auth-play-route 127.1.0.4:6112)
game_args=(--no-capture --listen "$GW2_BIND:6112" --mission-preamble
  --params "$GW2_ID/dh-params.bin" --private "$GW2_ID/dh-private.bin" --log "$GW2_RUN/unused"
  --mission-map-key "${GW2_MAP_KEY:-15}" "${game_population_args[@]}"
  --mission-spawn "$GW2_SPAWN" --mission-population-delay-ms "${GW2_POPULATION_DELAY_MS:-0}" "${mission_args[@]}")
if [[ "$GW2_CHARACTER_MODE" == saved-norn ]]; then
  # Validate both roles, the durable store and all profile fixtures before
  # starting any listener. A failure never falls back to a prepared character.
  "$GW2_BIN/gw2-game-observe" --check-config "${auth_args[@]}"
  "$GW2_BIN/gw2-game-observe" --check-config "${game_args[@]}"
fi

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
"$GW2_BIN/gw2-game-observe" "${auth_args[@]}" &
pids+=($!); auth=$!
"$GW2_BIN/gw2-game-observe" "${game_args[@]}" &
pids+=($!); game=$!

printf '{"bind":"%s","processes":[{"role":"cdn","pid":%d,"port":6115,"exe":"%s"},{"role":"login","pid":%d,"port":6114,"exe":"%s"},{"role":"auth","pid":%d,"port":6113,"exe":"%s"},{"role":"game","pid":%d,"port":6112,"exe":"%s"}]}\n' \
  "$GW2_BIND" "$cdn" "$GW2_BIN/gw2-assetcdn" "$login" "$GW2_BIN/gw2-login-observe" \
  "$auth" "$GW2_BIN/gw2-game-observe" "$game" "$GW2_BIN/gw2-game-observe" > "$GW2_RUN/pids.json.tmp"
mv "$GW2_RUN/pids.json.tmp" "$GW2_RUN/pids.json"
status=0
wait -n || status=$?
echo "a GW2 server process exited (status $status); stopping the rest" >&2
exit 1
