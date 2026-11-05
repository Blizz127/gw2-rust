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

Every server runs `--no-capture`. Nothing typed into the client is stored or printed,
and the binaries refuse a non-loopback listener otherwise.

## Layout (systemd --user, Banshee convention)

```
/home/blizz/gw2-rust-<sha>/          pinned kit (read-only by convention)
  bin/                               gw2-assetcdn, gw2-login-observe, gw2-game-observe (static musl)
  deploy/                            gw2-server.sh, gw2-probe.py, gw2-server.service
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
  - Nothing to save: no player state is kept.
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
It supplies a prepared character; normal creation and saved-character mode are
not enabled by this supervisor.

## What is synthetic

All fixtures are synthetic:
- Login accepts any made-up credentials. There is no account or Banshee ticket check.
- There is one fixed character, selected by the kit's record/profile.
- The default map is Queensdale (key15); the Norn profile uses tutorial379.
- Population still contains explicit research values. Both the default spawn
  `(0,0,7.4)` and the Norn profile's spawn are diagnostic positions, not proven
  retail starting coordinates. Combat and the complete tutorial are unfinished.

End-to-end status is in [COVERAGE](../docs/COVERAGE.md).
