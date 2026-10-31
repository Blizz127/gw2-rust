# Banshee Realm identity and launcher entry for GW2 (plan)

Owner, 2026-10-31 2:25 PM CT: "add gw2 to the launcher when it's connectable, use banshee
system". Tracked as Devbox task 088310d27083. The launcher lead prepares a hidden/disabled
entry now. The GW2 lead tells them the moment the server is connectable, and what the
client needs.

**Status: NOT connectable yet.** The login and game handshake are still being reverse
engineered (stage 3).

## Intended flow (mirrors lotro-rust `docs/BANSHEE.md`)

1. The launcher signs in to Banshee (`POST /v1/login`) and requests a GW2 join ticket
   (`POST /v1/games/gw2/ticket`). The issuer key and audience still need setting up on the
   accounts service side, and that is the owner's call.
2. The launcher starts the player's **own installed** GW2 client (build 209026) with
   arguments that point it at our server and pass the Banshee username and ticket. The
   client never talks to ArenaNet, Steam or Epic.
3. Our login server verifies the ticket **offline** with the issuer key. It does not
   proxy to ArenaNet and makes no callback to Banshee. The signed numeric Realm `user_id`
   becomes the account principal that owns characters. There is no local password
   fallback, except an explicit test-only flag.

## What the launcher needs from us (to be filled in once REA confirms)

| Item | Status |
| --- | --- |
| Host | Our server host. To be decided; loopback/LAN for the sandbox first. |
| Portal/login port | **Unknown.** 6112 and 6600 are 2012–13 public figures, unverified. |
| Game port | **Unknown.** |
| Redirect method | **Unknown.** Candidates: `-authsrv <host>`, if the flag still exists (REA string probe), and/or an in-memory patch of the client's baked DH server key and auth host. Never a file edit: the client verifies and repairs `Gw2.dat`. |
| Credential handoff | **Unknown.** Candidate: the client's own login arguments (historically `-email`/`-password`/`-autologin`, unverified on 209026), carrying the Banshee username and ticket. The ticket would then be checked as the "password" in our SRP portal step. |
| Patch suppression | Candidate `-nopatchui` (2013 public flag, unverified). The launcher must never let the client self-patch. |
| Steam/Epic | Both are delay-loaded (REA `ev_2b7cf3c4…`, docs/re/209026-pe-imports.md). The plan is a plain ArenaNet-provider launch with neither loaded. |

## Hard rules

- Never the owner's real ArenaNet, Steam or Epic account. No real credentials anywhere.
- The client only ever reaches our own server, firewalled.
- An in-memory patch, if needed, is applied by the launcher's runner to the running
  process only. Distributed client files are never modified.
