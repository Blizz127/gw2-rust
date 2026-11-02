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

## Layout

```
/srv/gw2/bin/         gw2-assetcdn, gw2-login-observe, gw2-game-observe (release build)
/srv/gw2/deploy/      gw2-server.sh, gw2-probe.py
/srv/gw2/data/        latest.txt, cdn-files/, startsso.xml, accounts.xml, token.xml,
                      auth2f.xml, empty.xml, auth-ready.bin, auth-account-body.bin,
                      character-record.bin
/srv/gw2/id/          PRIVATE identity (0700), created ON THE HOST:
                      tools/gw2-runner/make-identity.py <frozen exe> /srv/gw2/id <runner-public-dir>
/srv/gw2/run/         pids.json (0700)
/srv/gw2/gw2.conf     GW2_BIN, GW2_DATA, GW2_ID, GW2_RUN, GW2_BIND=100.74.44.12, GW2_SPAWN=0,0,7.4
```

The private identity never leaves the host. Only the runner-public directory (our TLS
modulus and DH public block) goes into the launcher's runner package.

## stackd probes (read-only, under 5 s, one JSON line)

- health: `/srv/gw2/deploy/gw2-probe.py health /srv/gw2/run`
  - Ready only when each process in `pids.json` is alive, runs its recorded exe, and
    owns its listener (socket inode under `/proc/<pid>/fd`).
  - Needs to read our processes' `/proc` entries (stackd wraps it in `sg`).
- humans: `/srv/gw2/deploy/gw2-probe.py humans /srv/gw2/run`
  - Counts ESTABLISHED connections on the game port owned by the game process.
  - Exits non-zero when the count can't be determined.

## What is synthetic

All fixtures are synthetic:
- Login accepts any made-up credentials. There is no account or Banshee ticket check.
- There is one fixed Human Guardian character.
- Queensdale (map key 15).
- Population values are placeholders, and the spawn (0,0,7.4) is an offline terrain
  probe hit, not a validated spawn.

End-to-end status is in [COVERAGE](../docs/COVERAGE.md).
