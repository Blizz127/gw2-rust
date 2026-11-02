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

- health: `/home/blizz/gw2-rust-current/deploy/gw2-probe.py health /home/blizz/gw2-rust-state/run`
  - Ready only when each process in `pids.json` is alive, runs its recorded exe, and
    owns its listener (socket inode under `/proc/<pid>/fd`).
  - Needs to read our processes' `/proc` entries (stackd wraps it in `sg`).
- humans: `/home/blizz/gw2-rust-current/deploy/gw2-probe.py humans /home/blizz/gw2-rust-state/run`
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
