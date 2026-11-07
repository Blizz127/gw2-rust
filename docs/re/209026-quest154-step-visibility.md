# Build 209026: quest154 step visibility and trophy fidelity

Static findings, 2026-11-07. **The client supports an explicit operation that
reveals the initially hidden second step.** The inspected server source sends
completion without that operation. This note records evidence and a narrow
correction candidate; it does not establish an implemented or deployed fix,
readable localized text, or authentic trophy encounter rules.

The server comparison is pinned to `a1e411054ec57fc3d48a72f60f92ccad294cab84`.
Its `mission.rs` is 454,380 bytes, SHA-256
`6158110aecf950f89249744beabcf434f8ac11f9b0f14f91903a0ab59c796d64`.
The frozen build209026 executable SHA-256 is
`99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`.

## Client mechanism

[Quest154 content](209026-quest154-content.md) defines goal1 on map379 and
steps **1, 2, 3, 6, 4, 5**. Their definition flags are `0,2,2,2,2,2`;
construction initializes runtime flags `0,0xa,0xa,0xa,0xa,0xa` and objective
IDs zero. The [HUD predicate](209026-quest154-objective-ui-boundary.md) at
`0x1411ffb30..0x1411ffb66` excludes a step for definition NoUi (`0x4`),
runtime bit2, or runtime bit1. Completion bit0 is independent.

| Operation | Exact source effect and bound HUD consumer |
| --- | --- |
| **542, complete** | `0x141258e00` → `0x1412017e0`; `0x141201898` ORs runtime step +0x10 with 1, evaluates goal completion, then notifies quest listeners through `0x14026e370` / `0x14026e848`, virtual +0x20. |
| **547, clear bit1** | `0x141259030` → `0x141201de0`; `0x141201e61` clears only runtime bit1. `0x14025552c` calls listener +0x38; after the visibility predicate permits the step, `0x14026e8bc` calls +0x28. For initially hidden step2, flags become `0xa → 0x8`; completion and objective ID are preserved. |

The actual quest listener is table **`0x141b43900` on widget +0x60**, installed
at `0x14074d093..0x14074d09a`. Registration
`0x14074edd9..0x14074ee09` passes that subobject to quest virtual +0x50:
table `0x142165f58` binds it to `0x141200010`, which registers in quest +0x20.
The adjacent `0x141b438e8` is the primary widget table, not this listener.

Listener +0x20 is `0x14074e1e0..0x14074e598`. It refreshes widget state and
marks/animates an existing row for the completed definition. Its write at
`0x14074e319` is **UI-row +0xc**, not runtime-step +0x10. This complete,
registered HUD completion path does not clear the next step's hidden bit or
create its hidden row. This is not an exhaustive census of other subscribers.

For 547, HUD listener +0x38 is the no-op `0x140163990`; +0x28 is
`0x14074e5a0..0x14074e748`, which finds the exact step and adds/removes its row
according to visibility. These receiver bindings supersede provisional
primary-table/slot interpretations made during discovery.

## Reconstructed trigger and restore bounds

At the inspected server revision, `complete_steps` emits only542. Thora at
stage0 completes step1; a configured nonboss kill at stage1 completes step2;
Ido at stage2 completes step3; Eir at stage3 completes steps6 and4 together;
the configured boss at stage4 completes step5. These are the server's
**reconstructed conditions**, not conditions recovered from the client.
Fresh same-map rejoin reconstructs the quest and replays completed steps,
without547 or544 visibility synchronization.

The narrow candidate is **542(step1), then547(step2)** after the accepted
stage0 Thora transition. Fresh stage1 rejoin needs the same reveal after
quest construction and completed-step replay. It must reject wrong roles,
duplicate transitions, mismatched quest/goal/map and inactive steps. Stage0
must not reveal future steps. Later visibility, including the paired6/4
transition, remains a separate policy question.

547 carries player-index Dword and step-ID Dword after the opcode, using the
existing unsigned-varint transport; it has no quest ID or Boolean. Clearing
the bit twice is idempotent, but notifications repeat. **Do not send a reveal
after final542 destroys quest154's active goal and steps, or reconstruct a
completed goal merely to reveal a row.** [544](209026-quest-step-state.md)
replaces full flags/objective ID without this notification or completion
evaluation; it is not an interchangeable live reveal. Preserve other flag
bits and do not derive an objective ID from a step, sector or text key.

Missing localization for title key **239267** and step keys **34076–34082**
remains independent. 547 does not decrypt those strings or establish readable
quest text. No trophy membership, placement, reward or persistence correction
is established by this visibility trace.

## Remaining trophy binding

Step2 is **1282830:4146408**, text34079, with a sector200 reference to
**1282846:3233008**. Its raw +0x28 value17 remains unnamed. The retained graph
does not bind an actor, kill condition or event instance. Quest +0x78's three
references are follow-up quests159/162/155, not three trophy actors. See the
[bounded species-link audit](209026-great-hunt-species-link-audit.md) and
[step identity consumers](209026-quest154-step-identity-consumers.md).

As supplemental player-facing evidence, the ArenaNet-hosted, community-authored
[official wiki walkthrough](https://wiki.guildwars2.com/wiki/The_Great_Hunt)
names Vicious Dire Boar, Owl Griffon Sire and Alpine Bull Minotaur trophy
events; credit is not an inventory trophy. It describes Thora → trophy event
→ Ido → Eir → hatchlings → Issormir, with
[dialogue recorded separately](https://wiki.guildwars2.com/wiki/The_Great_Hunt#Dialogue).
The retained 2026-11-07 retrieval used the indexed canonical page; direct open
returned403. This is not a build209026 population, coordinate or protocol
capture and does not authorize choosing actor/rule values from the wiki.

The readable client-name subset has no exact match for those three trophy
bosses; that does not exclude encrypted records. No new typed selector follows
from the reviewed step2 graph. The smallest missing input is an authentic
trophy event/content identifier linked to actor species/creation/location and
the step2 transition. That would justify one exact cached-record lookup and
its demonstrated consumer. Repeating catalog names, sector links, incoming
relocations or literal GUID scans would not close that dependency.

## Retained evidence

Private receipts under `/home/blizz/re/gw2/notes/` preserve full captures and
source pins; raw shipped content stays outside the repository:

- `quest154-visibility-a1e4110-findings-20261107.json`, SHA-256
  `c56e8bb2a0393b6d2c7790ddef0549e43f748643f68452ea7ac9b972e40475f1`.
- `quest154-fidelity-matrix-10803b4-20261107.json`, SHA-256
  `dfa9e47922ca223c8a8c2bb9147daa17e584ae37608d9dc61b738a9c35f805c7`.

The final visibility receipt pins these complete source captures:

| Private capture basename | Evidence ID |
| --- | --- |
| `209026-quest154-visibility-bound-completion-20261107.json` | `ev_3edc126fcff4db18934f2d610258be853e28834dca05e77eb4ffe137f54f49a9` |
| `209026-quest154-visibility-ui-refresh-20261107.json` | `ev_0205f38b7d0c19c62b636bce57dec12e1ec988eb11910f76485159fe2764e3c3` |
| `209026-quest154-visibility-widget-registration-20261107.json` | `ev_f41dacf676f8e7c179ab4d2bea9e530ac005720a4fe9c013b96873b6a1fbfba1` |
| `209026-quest154-visibility-reveal-callbacks-20261107.json` | `ev_7c562a7aeeeee23ac8f1b7808390b25965f78cc44a7eb975d20962555a9ffebb` |
| `209026-quest154-visibility-tables-20261107.json` | `ev_304ab001d3fcad155b7f31f1156bd9fe76bc59c11cd2a846d9c1b973b6eecd04` |

Selected raw and normalized output is complete; some captures omit rendered
frames, so aggregate truncation is not universally false. Final function
captures follow chained unwind fragments; earlier partial discovery captures
are retained but are not the basis for the complete-body claims above.

Prior transport evidence is in
`209026-quest-step-state-contract-20261105-evidence.json`, SHA-256
`32071b055f19cef3bf4ae59fdd0e412f1dd4b16c6a267a98786fefc9798b5c48`,
Evidence `ev_96951d789bb2bd71135793ce808c37f10e531431cb56dfc693517bb6f375fb65`
and `ev_f5191874f9a8309f54f321d94a1589fd3102abcdd5b2454ff48f4c6169dc62d9`.
Content, visibility and HUD evidence is in
`209026-quest154-objective-ui-boundary-20261106.json`, SHA-256
`0ba15b623f38124f796785280c2e711271aab2e74cc0647b7387a957acc4e704`,
Evidence `ev_38781654b454545ebff182fb280ca8f64e3d035f3abc51bc5a1f78d2367232c0`,
`ev_eb79fe185a11c27c74812fc7ebf5b547e2ff2373c887b27af8177ea9dfe3905a`
and `ev_f664f7414fdf58ee6dc8f97e12e8e6f7a045b5f94a4cafffad6a46657cd34ae1`.
