# GW2 server deploy bundle (bansheerealm)

Always-on and bound to the tailnet address only. stackd treats it as **readiness_only**
(DIT). Players reach it only through the Banshee launcher's network bubble and relay
(see [LAUNCHER-CONTRACT](../docs/LAUNCHER-CONTRACT.md)).

## Ports (bind 100.74.44.12 only; DIT-allocated 6110–6119)

| Port | Process | Bubble side (launcher relay maps to it) |
| --- | --- | --- |
| 6112 | `gw2-game-observe --mission-preamble` (game) | 127.1.0.4:6112 |
| 6113 | `gw2-game-observe` (binary auth + Play route) | 127.1.0.3:6112 |
| 6114 | `gw2-login-observe` (portal STS/TLS) | 127.1.0.2:6112 |
| 6115 | `gw2-assetcdn` (patch CDN) | 127.1.0.1:80 |

Every server runs `--no-capture`: credentials and decrypted streams are neither
captured nor printed. Explicit saved mode persists submitted character creation
records privately. The binaries refuse a non-loopback listener without this flag.

## Layout (systemd --user, Banshee convention)

```
/home/blizz/gw2-rust-<sha>/          pinned kit (read-only by convention)
  bin/                               gw2-assetcdn, gw2-login-observe, gw2-game-observe (static musl)
  deploy/                            gw2-server.sh, gw2-saved-store.py, gw2-probe.py, gw2-server.service
  tools/                             gw2-runner/make-identity.py, gen-test-cert.sh, dh-block.bin
  data/                              latest.txt, cdn-files/, startsso.xml, accounts.xml, token.xml,
                                     auth2f.xml, empty.xml, auth-ready.bin, auth-account-body.bin,
                                     character-record.bin
/home/blizz/gw2-rust-current         symlink to the kit in service
/home/blizz/gw2-rust-state/          0700
  gw2.conf                           GW2_BIN, GW2_DATA, GW2_ID, GW2_RUN, GW2_BIND=100.74.44.12, GW2_SPAWN=0,0,7.4
  id/                                PRIVATE identity, created ON THE HOST:
                                     make-identity.py --dh-block <kit>/tools/dh-block.bin id runner-public
  runner-public/                     PUBLIC values for the launcher's runner package
  run/                               pids.json, server.log
  characters-norn-owner/             optional saved-mode records (0700; files0600)
  backups/                           private stopped-service snapshots (0700)
~/.config/systemd/user/gw2-server.service
```

`dh-block.bin` is the client's public 136-byte DH parameter block (SHA-256 pinned in
make-identity.py), so the host never needs a copy of the client.

## stackd probes (read-only, under 5 s, one JSON line)

- health: `/usr/bin/sg blizz -c "exec /usr/bin/python3 -I /home/blizz/gw2-rust-<sha>/deploy/gw2-probe.py health /home/blizz/gw2-rust-state/run"`
  - Ready only when each process in `pids.json` is alive, runs its recorded exe, and
    owns its listener (socket inode under `/proc/<pid>/fd`).
  - Needs to read our processes' `/proc` entries. stackd runs with gid docker, so the
    `sg blizz` wrap is required. DIT pins the real kit path, not the `current` symlink.
- humans: the same with `humans`; prints `{"humans":N,"bots":0}`
  - Counts distinct remote peers ESTABLISHED on the login (6114), auth (6113) or game
    (6112) listener, owned by that process. A player at the login screen holds only 6114.
  - A fully stopped server reads 0. A partly running one, a reused pid, or anything
    unreadable exits non-zero, never a false 0.

## Sleep and wake (stackd r57: wake_only)

- start: `sg blizz -c "exec /usr/bin/env XDG_RUNTIME_DIR=/run/user/1000 /usr/bin/systemctl --user start gw2-server.service"`
- sleep: `gw2-sleep.sh <kit>/deploy/gw2-probe.py /home/blizz/gw2-rust-state/run`
  - It stops the unit only when humans reads exactly 0, otherwise it exits 1.
  - Saved character creation is synced before success is returned. Keep the
    durable store outside the kit and snapshot it during a stopped-service
    deployment as described below; sleep never deletes it.
  - Not atomic: a player who connects between the count and the stop is dropped.
- Switch `gw2-rust-current` only while the unit is stopped. Health compares each
  process's real exe with the realpath of its pids.json path, which goes through that link.
- `deploy/watched-cycle.sh` (devbox side) is a dry run by default.
  - `--execute` runs the full watched cycle with DIT's pinned argv: sleep, switch the link
    to `KIT`, start, then time start->ready.
  - Run it only after launcher b106 (PLAY wake-and-wait) is live and the owner's first
    b107 device run is done.

## Pinned mission profile

A kit can include `data/mission-profile.conf` to override the fixed mission
settings after the host configuration is loaded. It can set `GW2_MAP_KEY`,
`GW2_SPAWN`, `GW2_MESMER_VALUES`, `GW2_REWARD_TRACKS`, `GW2_HEALTH`,
`GW2_QUEST`, and the data-relative filenames `GW2_EQUIPMENT` and `GW2_SKILLS`.
The corresponding `character-record.bin` must name the same map and profession.
Without that file, the existing Human Guardian/Queensdale profile is retained.
Rollback restores the entire kit, including its fixture and mission settings.

The Norn Mesmer owner profile uses the exact inputs exercised by sandbox run118:
map379, spawn `-3200,-25600,-3742`, Mesmer values `1,0`, tracks `56,88,47,112`,
health388, quest `154,0,1`, and its equipment/skill batches. See
[health playback](../docs/re/209026-mesmer-health-speed-baseline.md) and
[initial equipment](../docs/re/209026-initial-equipment-model-order.md).
It supplies a prepared character by default. Saved creation requires the explicit
host opt-in below; shipping a new kit does not enable it.

A profile can also enable the research combat loop with `GW2_HOSTILE_NPC`
(one or more space-separated SPECIES,X,Y,Z entries, at most 16), `GW2_DAMAGE_ON_SKILL`, `GW2_AUTOATTACK_SKILL`, `GW2_XP_PER_KILL`,
`GW2_STARTER_BAG` (959), `GW2_KILL_LOOT=1`, `GW2_NPC_ATTACK` and
`GW2_SOURCED_DAMAGE` (1) and `GW2_HEART` (TYPE72_KEY,KILLS: a renown heart shown at
spawn that advances per kill) and `GW2_RESPAWN=1` (the defeat screen's "Return to a
Waypoint" revives the player in place). These map one-to-one to the `--mission-*` flags in
[the talk-loop note](../docs/re/209026-npc-talk-loop.md). The values are
sandbox-tested placeholders, not retail encounter data.

## Explicit saved Norn mode

This is one local owner account, not Banshee account authentication. The creator
UI remains under investigation; configuration support does not establish a
complete normal-creation flow. Leave the current owner configuration unchanged
until the intended saved-mode activation.

The host `gw2.conf` may explicitly set:

```sh
GW2_CHARACTER_MODE=saved-norn
GW2_CHARACTER_STORE=/home/blizz/gw2-rust-state/characters-norn-owner
```

The default is `GW2_CHARACTER_MODE=prepared` with no store setting. That default
retains the existing auth/game argv and prepared record. Both settings are
read-only while loading `mission-profile.conf`, so a kit cannot opt the host in.
Unknown modes, a store specified in prepared mode, or invalid saved settings
fail closed. No empty/corrupt-store or unknown-GUID fallback is allowed.

The store path must be absolute, beneath the host configuration directory and
outside the actual kit directories, including symlink-resolved paths. The host
state directory, existing parent and existing store must be private directories
owned by the service user; the store itself cannot be a symlink. The observer
creates a missing leaf directory with0700 and records with0600. Its preflight may
create this directory and its lock, but does not synthesize characters.

Saved mode requires map379, spawn, Mesmer values, reward tracks, equipment,
skills, health and quest settings. The equipment batch must leave slot5 free
for the submitted biography mask. Before starting any child, the supervisor runs
both exact observer argument sets with `--check-config`. This checks the actual
store, fixtures, all mask/dye capacity cases, combat settings and local DH
identity, then exits without binding or creating captures. Include the new
`gw2-saved-store.py` helper in the kit alongside the supervisor.

Auth replaces `--auth-character-record` with the store and
`--auth-create-route 127.1.0.4:6112`. Game replaces `--mission-population` with
that same store and `--creation-scene 127.1.0.4:6112`. The existing four listeners
and launcher routes stay unchanged. The validated map0 prefix selects creation;
map379 requires a known saved GUID. World combat options apply only to the
selected world phase.

Admission remains Norn/Mesmer/level1 with the supported scalar bounds, exactly
one mask answer187/188/189 and representable dyes. The store preserves the exact
request and stable GUID; it does not persist later combat progression. The
prepared fixture is not imported or merged into the saved roster.

## Saved-state backup and rollback

Use the existing player-count protection and stopped-unit boundary. After the
unit is confirmed stopped and before changing the kit link or host mode, snapshot
the host config and existing store. For the path above, the host-side commands
are:

```sh
umask 077
install -d -m 700 /home/blizz/gw2-rust-state/backups
snapshot=$(mktemp /home/blizz/gw2-rust-state/backups/saved-norn-XXXXXXXX.tar)
tar -C /home/blizz/gw2-rust-state -cf "$snapshot" gw2.conf characters-norn-owner
sha256sum "$snapshot" > "$snapshot.sha256"
```

Use the configured relative store path when it differs; before the first store
exists, back up `gw2.conf` alone. These are private operator steps, not commands
to run against an active service. `watched-cycle.sh` still performs its existing
stop/switch/start flow and does not take this snapshot automatically; use the
documented stopped-unit steps for a saved-state deployment.

Retain the store and snapshot when rolling back a kit. Returning to prepared
mode requires `GW2_CHARACTER_MODE=prepared` and an empty/unset
`GW2_CHARACTER_STORE`; it hides the saved roster without deleting it. Do not
restore an older snapshot over newer character records automatically. A failed
preflight leaves existing records intact and starts no listener.

## What is synthetic

All fixtures are synthetic:
- Login accepts any made-up credentials. There is no account or Banshee ticket check.
- Prepared mode has one fixed character selected by the kit's record/profile;
  explicit saved mode has one local owner store.
- The default map is Queensdale (key15); the Norn profile uses tutorial379.
- Population still contains explicit research values. Both the default spawn
  `(0,0,7.4)` and the Norn profile's spawn are diagnostic positions, not proven
  retail starting coordinates. Combat and the complete tutorial are unfinished.

End-to-end status is in [COVERAGE](../docs/COVERAGE.md).
