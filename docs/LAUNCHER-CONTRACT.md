# GW2 launcher and Server Operations contract (current state)

Devbox task dbf27f7e17a0, 2026-11-01. This is for the launcher lead (launcher852bb105)
and Server Operations. It records what exists now and separates **private synthetic
offline fixtures** from a **deployed, connectable game server**. There is no deployed
GW2 server. The launcher entry must show **unavailable / not deployed**, with PLAY blocked
([BANSHEE.md](BANSHEE.md) rule). This contract does not accept the User Agreement
(approval f7209359 is still pending) and supplies no real credentials.

## Client

| Item | Value |
| --- | --- |
| Build | 209026, frozen 2026-10-31 ([client-build.md](client-build.md)) |
| Executable | `Gw2-64.exe` = frozen `Gw2-64.new.exe`, SHA-256 `99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621` |
| Package | `/home/blizz/gw2-client/build-209026.ship-manifest.tsv` (10 files, owner tier, never auto-updated, never public) |
| Runtime | Wine 10 / GE-Proton 11 on the devbox, inside `sandbox/run.sh` (`--network none`). No owner-device runtime has been validated. |
| Data | `Gw2.dat` must stay intact (1,124,232 entries). A wrong patch manifest makes the client trim it; recover with `sandbox/run.sh restore 209026`. |

## Endpoints the client uses (observed in the offline sandbox)

| Role | Name the client resolves | Port | Our component | Status |
| --- | --- | --- | --- | --- |
| Patch / asset CDN | `assetcdn.101.arenanetworks.com` | TCP 80 (`GET /latest64/101`, then files) | `gw2-assetcdn` + `gw2-manifest-gen` | Works offline. The launcher reaches READY / PLAYABLE (run 35). |
| Portal (STS) | `cligate.101.ncplatform.net.` (trailing dot) | TCP 6112 | `gw2-login-observe` (STS + minimal TLS 1.2) | Synthetic fixtures through news panel and Play (runs 44–58) |
| Auth (binary) | Second TCP connection after the game token | per portal reply | `gw2-game-observe` | Synthetic fixtures: IDs 2/8/10/7 accepted (runs 52–58) |
| Game / mission | Address from auth route result 20 | per route result | `gw2-game-observe --mission-preamble --mission-map-key KEY` | Synthetic offline opening only: join via key exchange, then 1063 + placeholder 1060 ([mission entry](re/209026-mission-entry.md)). Not deployed, not run against the client; no population |

**Name mapping rule:** resolve these names into **127.1.0.x, never 127.0.0.0/24**.
In the 127.0.0.0/24 range the client's address lookup recurses and kills its socket
thread. The sandbox uses:

```
DNS_SINK=0 ADD_HOSTS="cligate.101.ncplatform.net=127.1.0.1 cligate.101.ncplatform.net.=127.1.0.1 assetcdn.101.arenanetworks.com=127.1.0.2"
```

## Arguments and redirection

- `-authsrv host[:6112]` exists in build 209026 and sets the later AuthSrv hop
  ([cmdline-and-hosts](re/209026-cmdline-and-hosts.md)).
  `-nopatchui`/`-nopatch` and `-autologin`/`-token` exist as switch names.
  Their effect on our flow is unverified.
- **Current runs also need two in-process memory substitutions.** Each is
  guarded against the original bytes, and neither changes files:
  - our portal TLS trust modulus ([login TLS](re/209026-login-tls.md))
  - our DH public value ([game handshake](re/209026-game-handshake.md))

  A launcher runner would have to apply these to the running process. Distributed
  files are never modified.
- The Banshee ticket handoff is still **unimplemented**. The planned path is the ticket
  passed to our runner on stdin. There is no ticket verification, account database or
  credential validation.

## Readiness and lifecycle (Server Operations)

| Item | Current state |
| --- | --- |
| Deployed service | **None.** The components above are developer/observer binaries run by hand inside the sandbox. They have no unit files, health endpoints or supervisor. |
| Readiness signal | None. The Server Operations entry should report `not deployed`. |
| Sleep / wake (task 05f562fd7d03) | Not applicable until a service exists. Nothing is running to sleep. |
| Persistence | None. There is no character or account storage, so no save/restore semantics yet. |
| Ports to firewall | 80 and 6112 on the chosen loopback/LAN host, plus the future game port. |

## Exact remaining blockers to a usable launch path

1. **Owner decision (Run59 hold):** run 59 shows the User Agreement panel after the
   launcher's Play and before character select. Its code path is untraced. No progress
   past it without explicit owner direction.
2. **Game server:** route result 20, the game join and the map-load opening (1063, placeholder
   1060) exist as offline observer fixtures only. Valid 1060 values, character population
   (428…444), world entry (46), 412 → BeginPlaying (1036) and a deployable service remain.
3. **Launcher runner:** host mapping, the guarded memory substitutions, and stdin ticket
   handoff on the owner's device. D2 (per-process network restriction) is unanswered.
4. **Accounts:** Banshee ticket verification and character storage.

Until 1–4 are done, the honest launcher state is a visible but disabled GW2 entry with
the reason "server not deployed".
