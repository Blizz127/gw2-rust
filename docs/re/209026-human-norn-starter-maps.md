# Build 209026: human and norn starter map identities

Read-only archive and executable-table audit, 2026-11-05. This supports the
owner's norn-first, then human Mesmer starter-area work. It does not establish a
complete tutorial, a spawn point, or a playable area.

## Separate tutorial and open-world map keys

The names below come from the existing build-209026 API cache
(`data/api/209026/maps.json`). The map keys, archive references and numeric
route types were checked separately against the frozen client content.

| Map key | Cached API name | Map archive | Client map type | Client game type |
| ---: | --- | ---: | ---: | ---: |
| 379 | The Great Hunt | 187611 | 7 | 9 |
| 28 | Wayfarer Foothills | 187611 | 5 | 3 |
| 138 | Defense of Shaemoor | 192711 | 7 | 9 |
| 15 | Queensdale | 192711 | 5 | 3 |

Each tutorial references the same map archive as its corresponding open-world
map, but has a separate type-45 definition and a different game route type.
Sharing geometry does not establish shared spawning, NPC population, events,
story objectives, completion state, or server behavior. Sending the open-map
route type for a tutorial would disagree with the client's type lookup.

The current auth handler already echoes the client's requested game type,
reserved value and map key in server 20 (`game_observe/auth.rs`, client 29
branch). It does not hardcode game type 3 or reject type 9. No auth route
change is needed solely to preserve the tutorial game type. This source
behavior does not prove a successful tutorial connection or that the mission
endpoint serves the requested map.

This table does not independently prove a race-to-tutorial selection rule.
The client race enum is established as Human = 2 and Norn = 3 in
[character creation](209026-character-creation.md). The initializer or
server rule connecting that enum to these map keys remains to be established.

## Content-reference evidence

The audited field is the file relocation at type-45 record **+0x68**, using the
already documented [map-definition lookup](209026-map-definition-content.md)
and [shared file-table selection](209026-content-file-references.md).

| Map key | Record container | Content-relative record offset | Shared file-table index |
| ---: | ---: | ---: | ---: |
| 138 | 1282833 | 3784456 | 60539 |
| 15 | 1282830 | 3754040 | 60539 |
| 379 | 1282833 | 3863896 | 60261 |
| 28 | 1282830 | 3731704 | 60261 |

The helper checked the container hashes, record type and data key, file-fixup
membership, shared-table index bounds and encoded archive-ID form. All four
references have suffix zero. The table is in container **1282830**, including
for records stored in container 1282833; using each record's own file table is
incorrect for this base-content set.

Container SHA-256 values:

- 1282830: `e5d11e4fae8627d3efced749e7ab599b26abac59e502716aaf7d63ef509bb81f`
- 1282833: `5ab847328dd307407d9701711890f08282f9c6f3639d24bb9684ea46006e26ef`

## Route-type evidence

The [previous Play-route trace](209026-play-route.md#queensdales-map-type-and-game-type-2026-11-02)
identifies archive **198302**, `mMet/Main`, as the map-type lookup. The new
audit verified its hash, decoded all 1,244 sorted unique records and selected
the four keys above. It then read the 18 `{gameType, mapType}` pairs at
executable VA **0x1422dda10**, preserving the client's first-match rule.

- Executable SHA-256:
  `99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`
- Archive 198302 SHA-256:
  `d0f758fa69a37e1c10a9e4e90ab2707c4e1fb9ebf9ef282eba05ac44c632b333`

## Remaining work and next source step

The current mission implementation only handles map readiness and one player
population. It does not implement tutorial objectives, ambient/enemy NPC
population, combat, hearts, events, map discovery, waypoints or persistent
completion. The legacy `PopulationFixture::new` retains the Guardian fixture;
explicit Mesmer initialization is now available through `with_profession`
([Mesmer state](209026-mesmer-profession.md)), without retail defaults. This is a source audit of the current
implementation, not a statement that these features are absent from the
shipped client.

No retail starting coordinate has been established. The existing operator
spawn and `mapc/msn` interest points must not be promoted to retail spawn
defaults. A next client-source investigation should follow map-specific
navigation/marker definitions and their type-45 association, together with
character-creation completion and map-transfer consumers, to locate a
source-backed spawn and tutorial-selection rule. Existing navigation leads
are types **306/307**, population helper **0x1413f66e0** and endpoint resolver
**0x1413f60f0**, documented in
[quest navigation](209026-quest-navigation-guard.md). They establish
navigation data, not NPC encounter scripts or spawn semantics.

The biography answer extraction has applicability flags and wire answer bytes
but no established localized answer meanings. It does not yet identify a
circus choice; see [creation answers](209026-creation-answers.md).

## Evidence and limits

REA process capture ran read-only Python extraction helpers, not the game
client. Both final captures exited with code 0 and verified cleanup of their
owned process groups. The helper imports and external files are not
automatically bound to the Python executable identity; the archive/executable
hash assertions above establish the selected inputs. No game server, client
fixture, account, deployment or map configuration was changed.

| Evidence | Private retained file | Result |
| --- | --- | --- |
| `ev_eb1a5cd7adc007be29315e618cb6f72965e6043297313025a1c77948546bc408` | `~/re/gw2/notes/209026-human-norn-map-links-20261105.json` | Four type-45 file references resolve through the shared table. |
| `ev_ea74e7b80fbc230fa8ddef09e216545953b18c946536bfc4f92fac61d1980520` | `~/re/gw2/notes/209026-human-norn-map-types-20261105.json` | Four map types and their matching game types, with input hashes. |

The second capture reported unrelated `/proc` entries that it could not
inspect and left untouched. This did not prevent the selected helper from
exiting or its owned cleanup from being verified. No runtime map load,
tutorial execution, new character creation or retail-behavior comparison
was performed.
