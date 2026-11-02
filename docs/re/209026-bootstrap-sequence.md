# Build 209026: candidate bootstrap sequence (synthesis)

Synthesis, 2026-11-01, of the existing build-209026 notes into one ordered
candidate sequence, from character select to a controlled player in
Queensdale. **No new REA was performed and nothing was replayed.** Every
step cites the note that establishes it.

Each ordering edge has one of three statuses:
- **stated**: the cited note asserts it
- **inferred**: it follows from stated prerequisites, but no note asserts it
- **open**: unknown

Runtime observation reaches only the Play button and the User Agreement
panel (runs 58/59). Everything after that is static. The Run59 agreement
hold still applies, so no step below has been sent to the client.

## Phase 1: portal and auth (runtime observed)

The STS/TLS portal flow, the game token and the binary auth exchange
through server IDs 2, 8, 10 and 7 are observed at runtime up to the news
panel and the Play button. See [auth messages](209026-auth-messages.md) and
[post-auth portal](209026-post-auth-portal.md). Character-list records
(protocol-3 server 17, completed by a generic ID-4 completion) are static
plus a parsed fixture ([character protocol](209026-character-protocol.md)).

## Phase 2: route to the game server

| Step | Message | Status | Source |
| --- | --- | --- | --- |
| 2-pre | client 22 (name) → server 3 (completion); server 8 (type-0x15 value, while pending) | Stated: Play calls view slot +0x60, the name transaction, and server 3 completes it to reach `0x14094db50`. Server 8 is inferred: it supplies the character value used downstream but is not asserted on the traced Play path. | [character creation](209026-character-creation.md), [play route](209026-play-route.md) |
| 2a | client auth 29, route request | Creation route: `[req, 1, 0,0,0,0]` + zero GUID. **Play route traced:** game type from the summary map's type, map key, the character's 16-byte value and 20-byte string | [mission join](209026-mission-join.md), [play route](209026-play-route.md) |
| 2b | server auth 20, route result (address, params) | Stated: follows 29 | mission join |
| 2c | protocol-0 game join, 84-byte preamble | Stated: follows 20. Observer support exists; untested | mission join |
| 2d | (no message) join completion | Stated: the client completes the join itself on the type-1 key reply (constant success result) and sets PLAYER_FLAG_CONNECTED (ctx+0x228 bit 0x2). No server acknowledgement; 1267 is unnecessary (it matches no state-3 transaction) | [mission entry](209026-mission-entry.md) |

The [Play route](209026-play-route.md) closes the main phase-2 gap. Still
open: Queensdale's map type (and so its game-type value), the ctx +0xdd
source, and the hashed string.

## Phase 3: map load (loader states)

The loader states come from [map ready](209026-map-ready.md):
1 LOAD_CONTENT → 2 LOAD_MANIFEST → 3 SERVER_WAIT → 5 MAP_DOWNLOAD → 7
MAP_STREAM → 9 MODELS → 10 ASSETS → 11 AGENT_STREAM → 12 READY_WAIT → 13
READY.

| Step | Message | Effect / prerequisite | Order status | Source |
| --- | --- | --- | --- | --- |
| 3a | server 1063, MapData | Stores the map ID (ctx +0x1f0) and the map definition (+0x1e8); sets readiness mask 0x80 | **Stated before 3b:** 1063 is the only nonzero writer of ctx+0x1e8, which LoadMap asserts equals the loader's definition. Route preparation (BeginMap) sets the loader's own definition, not ctx+0x1e8. Inferred: the map ID must equal route result 20 field 3 | [mission entry](209026-mission-entry.md), [mission transfer](209026-mission-transfer.md) |
| 3b | server 1060, OnRecvLoadMap | Sets the command-received flag +0x2c4, which SERVER_WAIT requires | Stated: needed to leave state 3 | mission transfer, map ready |
| 3c | client 420, MapLoaded | Sent on MAP_STREAM success (418 on failure) | Stated: precedes READY; "too early" as a readiness signal | map ready, mission transfer |
| 3d | client 423, language prefs | Ordinary branch: masks 0x40 (join completion) + 0x80 (1063) set, 0x1 clear | Stated: sent once both 2d and 3a happened, in either order | [mission entry](209026-mission-entry.md) |
| 3e | (no message) AGENT_STREAM | Waits for a **nonnull controlled character** plus streaming conditions | Stated. Phase 4 must complete here. Other prerequisites are open. | map ready |
| 3f | client 412, AgentsLoaded | Loader READY, ctx +0x228 mask 0x2, view state 0xd | Stated (implementation inference in the note) | map ready |
| 3g | server 1036, BeginPlaying | Asserts loader READY; view state → 0x10 | Candidate: send after 412 | map ready |

Server 1061 (OnRecvLoadAgents) is empty and reaches a no-op callback. It
cannot stand in for population data.

## Phase 4: player and character population (before AGENT_STREAM completes)

The chain below comes from
[character finalization dependencies](209026-character-finalization-dependencies.md),
[core stats](209026-character-core-stats.md),
[population](209026-character-population.md),
[profession](209026-character-profession.md),
[skill bar](209026-character-skillbar.md),
[control](209026-character-control.md) and
[control view](209026-character-control-view.md).

| Step | Message | Order status |
| --- | --- | --- |
| 4a | server 428, player create (index ≠ 0) | Stated first; 170 validates it |
| 4b | server 37, type-0 player character agent ([agent creation](209026-agent-creation.md)) | Stated: 428 → 37 → 443 → 182/183 for a fresh owned character |
| 4c | server 443, owned-character assignment | Stated: before coreStats and before 170 |
| 4d | server 182 or 183, coreStats (alternatives) | Stated: after 443, before 170 |
| 4e | server 282, inventory | Stated prerequisite of 170 |
| 4f | one profession creator (e.g. 477 Guardian) | Stated: before 170 |
| 4g | server 151, attitude < 4 | Stated prerequisite of 170 (the attitude must be < 4). The default value is not established. |
| 4h | servers 247/248, wardrobe manager | Candidate before 170's model path; required before 431 |
| 4i | server 170, finalize character | Stated: after 4a–4g |
| 4j | server 618, skill bar | Stated: before equal-ID 444 when the ordinary view is subscribed; not required by 170 |
| 4k | server 444 (A == B), controlled character | Stated: after 170 and 618; dereferences coreStats |

Further state, placed by stated prerequisites only:
- **health 256 / endurance 217:** need the character to exist
- **skill availability 605:** after 428, before 618 (stated as supported)
- **palette 612:** needs 443 and 618
- **specialization 640/642:** non-null needs 443
- **progress 509–511:** need a controlled player; absent → skipped

## Phase 5: agent world entry

From [world entry](209026-agent-world-entry.md),
[scene entry](209026-agent-scene-entry.md) and [reveal](209026-agent-reveal.md):
- **Server 46** sets ENTERED_WORLD and normally enters the collision scene, which reaches REVEALED. No separate reveal packet is needed on this route.
- **Server 49** is the scene-only alternative.
- **Positions 86/87:** 87 needs the ENTERED_SCENE collider flag.
- **Spawn constraints:** 37's facing must be unit length, its radius > 0, and its Point3 inside the scene bounds.

**Open:** where 46 falls relative to 444 and to the loader states.

## Phase 6: player startup event (server 431)

| Prerequisite | Supplied by | Status | Source |
| --- | --- | --- | --- |
| Controlled player, skill bar | 444, 618 | Asserted by 431 | [wardrobe startup](209026-wardrobe-startup.md) |
| Wardrobe manager | 247/248 | "must already exist" | wardrobe startup |
| Reward-track manager | 565 (after 428) | Consumed by 431 | [reward-track messages](209026-reward-track-messages.md) |
| Quest manager reported | 538 (after 428) | Stated: before 431 reaches the content-direction callback | [content direction](209026-content-direction-startup.md) |

**Open:** where 431 falls relative to 1036; its frame is two bytes,
`af 01` ([player startup frame](209026-player-startup-frame.md)). Per the
[quest-store chain](209026-quest-store-subscription-flag.md), the store
request reached from 431's quest callback needs no answer for startup,
statically.

## Candidate order (for fixture design, not a validated sequence)

1. Phase 2: 22 → (server 8) → server 3 → 29 → 20 → join (completes on the key exchange).
2. 1063 → (client 423) → 1060.
3. 428 → 37 → 443 → 182/183 → 282 → profession → 151 → 247/248 → 565 → 538 → 605 → 170 → 618 → 444.
4. 46.
5. Client 420 / 423 / 412.
6. 1036.
7. 431.

Placing 565, 538 and 605 early merely satisfies their stated
prerequisites. **The relative order of steps 2–4, the exact timing of 431,
and phase 2 for Play are open.**

## Highest-value open RE items

1. ~~The existing-character Play route~~ (traced in [play route](209026-play-route.md)); Queensdale's map type → game type remains.
2. AGENT_STREAM `0x140951ce0`: its full conditions beyond a controlled character.
3. Whether population (phase 4) and world entry (phase 5) must precede 1060, or only precede AGENT_STREAM.
4. The position of 431 relative to 1036 and READY.
5. Valid default values (attitude, flags, 46 option bits). The individual notes list these as open.
