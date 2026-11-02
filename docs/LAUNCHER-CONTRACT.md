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
thread.

**Resolve through DNS, not a hosts file.** With cligate resolved from `/etc/hosts`
and no DNS responder running, the real client aborts TLS with alert 10 before its certificate check (runs 60, 60b,
60c, 60e). The same layout answered by a DNS responder passes login, auth and Play
(run 60g; run 61 also passes with the `tools/gw2-runner` patcher). The cause inside the client is not traced. The sandbox uses
`gw2-dns-observe` on 127.0.0.1:53 UDP with resolv.conf `nameserver 127.0.0.1`:

```
DNS_SINK=1 DNS_ANSWER=127.1.0.9 DNS_MAP="assetcdn.101.arenanetworks.com=127.1.0.1 cligate.101.ncplatform.net=127.1.0.2"
```

It answers A queries only (AAAA gets an empty answer) and forwards nothing. Names
queried in run 60g: the two mapped names, plus `wpad`, `www.guildwars2.com`,
`a.cdn.ua.ncsoft.com`, `cms-live`/`exchange-live`/`gemstore-live`/`tradingpost-live`
(`.ncplatform.net`) and `update.googleapis.com`. The auth and game hops use IP
addresses and resolve no name.

## Owner launch path: network bubble (agreed 2026-11-02)

The owner approved running the client in a network sandbox that can reach only our
server ("yes to bubble it"). The launcher (banshee-realm-client) runs the client and the
GW2 runner inside `bwrap --unshare-net`, so the namespace has loopback only. An in-sandbox
bridge relays each endpoint below over a Unix socket to the launcher. The launcher
connects only to the matching real server endpoint. TCP only, except the in-bubble
DNS responder on 127.0.0.1:53 UDP, which never forwards. The bubble's hosts file holds
localhost only.

Sandbox run 60g verified this exact layout with the real client: login TLS on .2, auth on
.3, through Play to the User Agreement panel. The owner accepts the agreement himself on
his own device; automation never clicks it.

Uid split on the owner's device (launcher lead's probes): the outer bubble maps uid 0, so
the bridge and DNS can bind 53/80. Nested user namespaces are refused there, so wine and
gdb run as mapped uid 0 through `setpriv` with all capabilities dropped and no_new_privs.

| Inside the bubble | Client source | Role |
| --- | --- | --- |
| 127.1.0.1:80 | DNS: `assetcdn.101.arenanetworks.com` | Patch CDN |
| 127.1.0.2:6112 | DNS: `cligate.101.ncplatform.net` (the client uses the trailing-dot form; the query arrives without the dot) | Portal / login (STS + TLS) |
| 127.1.0.3:6112 | client arg `-authsrv 127.1.0.3` | Binary auth |
| 127.1.0.4:6112 | address our auth server returns in route result 20 | Game / mission |
| 127.1.0.9 (no listener) | DNS: cms-live, exchange-live, gemstore-live, tradingpost-live (`.ncplatform.net`), `www.guildwars2.com`, `a.cdn.ua.ncsoft.com`, `update.googleapis.com`, `wpad` | Must stay unreachable (connection refused, as in the tested runs) |

**Deployed 2026-11-02** on bansheerealm as `gw2-server.service` (systemd --user, kit
`/home/blizz/gw2-rust-33ce1bf`, readiness_only in stackd). The relay maps:

| Inside the bubble | Banshee (tailnet only) |
| --- | --- |
| 127.1.0.1:80 | 100.74.44.12:6115 (CDN) |
| 127.1.0.2:6112 | 100.74.44.12:6114 (login) |
| 127.1.0.3:6112 | 100.74.44.12:6113 (auth) |
| 127.1.0.4:6112 | 100.74.44.12:6112 (game) |

The runner package (public identity matching the server's) is
`/home/blizz/gw2-runner-pkg-098ced7/` on the devbox. Sandbox run 61b used the deployed
argv and data and reached the agreement panel. **Everything past the agreement is untested**:
- what I ACCEPT sends (an STS path with no fixture gets no reply; the server log names it)
- character select / Play route 20
- mission join, 1063/1060
- population, and the placeholder spawn 0,0,7.4

The devbox reaches only port 22 on Banshee. The launcher lead confirmed headlessly that
the owner's device reaches all four ports.

Sandbox run 62 ran the launcher's generated `bridge.py` (branch cdaf52d9, sha256
c9515fdb…) as the only DNS responder and listener, using `sandbox/run.sh` with `BUBBLE_DIR`. A stand-in
relay (`sandbox/bubble-relay.py`) forwarded its Unix sockets to our servers on Banshee's
port numbers. The real client went through login, auth and Play to the agreement panel.
Not covered: the device-only uid-0/setpriv wrapper (AppArmor refuses it on the devbox).

**Prefix needs DXVK.** Every passing run used DXVK. The runner's default `d3d11,dxgi=n`
means native only, so a prefix without the DLLs fails at D3D11 load.
- Provision: on first use, copy GE-Proton's `files/lib/wine/dxvk/x86_64-windows/`
  d3d11, dxgi, d3d10core and d3d9 into `drive_c/windows/system32`, as the sandbox's
  `prepare_prefix` does with DXVK 3.1.1.
- `--no-dxvk` (wined3d) is untested.
- Run 64 covered the GE-Proton11-6 Wine and GE's own DXVK (the client logs "v3.10") in a fresh
  prefix provisioned this way, against the live Banshee server. It reached the agreement
  panel. A real GPU (all runs used lavapipe) and the device's namespace shape are untested.

Sandbox run 63b used the **live** Banshee server. The pieces:
- the shipped runner package (its identity and gdb script) and the launcher's `bridge.py`
- the host-side relay over `ssh -L`, with `BUBBLE_UPSTREAM=external`, and no local servers

It reached the agreement panel. Banshee's log shows login, auth and the play-phase
messages, and the marker password is absent. This is the owner's device path minus his
display and the setpriv wrapper.

**Service mode:** a server facing a person runs `gw2-login-observe --no-capture` and
`gw2-game-observe --no-capture`.
- Neither creates capture files.
- The console shows only STS request paths, message IDs and sizes, never typed values.
- `tools/test-starttls.py` sends a login carrying a marker password and checks that
  no-capture mode writes no file and never prints it.
- Players sign in with **made-up** credentials only.

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
2. **Game server:** route result 20, the game join, the map-load opening (1063, placeholder
   1060), a candidate character population (428…444, 46) and 412 → BeginPlaying (1036) exist
   as opt-in offline observer fixtures only, never run against the client. Remaining: valid
   1060 values and a valid spawn position, the many placeholder population values, player
   startup (431) and everything after 1036, and a deployable service.
3. **Launcher runner:** host mapping, the guarded memory substitutions, and stdin ticket
   handoff on the owner's device. D2 (per-process network restriction) is unanswered.
4. **Accounts:** Banshee ticket verification and character storage.

Until 1–4 are done, the honest launcher state is a visible but disabled GW2 entry with
the reason "server not deployed".
