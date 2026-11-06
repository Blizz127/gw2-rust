# Mesmer starting experiences: completion ledger

Owner goal, 2026-11-05: finish the Mesmer starting area to 100%, retail
accurate, using the shipped client as the base source of truth. Latest
ordering: **Norn first**, then Human. This ledger preserves the full goal;
passing a codec or entering an empty map is not completion.

The pinned executable is build 209026 (`Gw2-64.new.exe`, SHA-256
`99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`).
Source-derived behaviors must retain REA evidence and distinguish client
observations, implementation choices and missing server behavior. Cached
API/wiki names may locate content but do not replace client verification.

## Required end state and evidence

| Requirement | Current state | Evidence needed for completion |
| --- | --- | --- |
| Norn Mesmer creation/selection and correct appearance | Private UI creation/save, restart relog and visible world entry pass; run144 passes the combined full creator/save/world/waypoint flow with the current offered runner and deployed metadata. Live saved-mode activation remains held. | Owner-mode activation with existing prerequisites and preservation of biography/appearance/equipment. |
| Correct Mesmer components and startup skills/equipment | Partial: explicit Mesmer component and source-backed item delivery; runs115/116 render armor, weapon and weapon/healing icons | Client-backed defaults/selection rules, real skill bar and equipped items, correct unlocks and stats. |
| Norn opening tutorial (Great Hunt, key 379) | Isolated map entry and local movement pass at diagnostic position; gameplay missing | Source-backed spawn, intro/dialogue, NPC encounters/objectives/boss, rewards and tutorial exit in client. |
| Wayfarer Foothills (key 28) terrain, movement and travel | Shared terrain loaded through Great Hunt; open map 28 not played | Correct entry/spawn, collision, movement, map transitions, waypoint use/death recovery in client. |
| Mesmer combat and profession mechanics | Diagnostic target damage, downed/defeat and waypoint rejoin work; retail skill effects and profession mechanics remain incomplete. | Relevant weapons, attacks, healing, clones/phantasms/shatters, cooldowns, conditions/boons, target/death/downed behavior verified in encounters. |
| Starting-area NPCs, enemies, vendors and interactions | NPC/dialogue/combat/loot foundations and ambient ravens are available; authored Great Hunt population and placements remain open. | Client-backed population/identity/placement plus exercised dialogue, shops, combat, loot and respawns. |
| All starting-map hearts, events and discovery objectives | Heart56 progresses/rewards in a diagnostic fixture, with coordinate and reward-once fixes; this is not authored Great Hunt content or whole-map coverage. | Source-backed inventory of objectives, implemented outcomes/rewards, completion counts and client playback for every objective. |
| XP, leveling, skills and equipment progression within area | Explicit fixture XP/level, loot, armor, biography mask and dyes are exercised; retail reward/unlock rules remain incomplete. | Correct rewards/stat progression/unlocks and inventory changes across the starting experience. |
| Relevant starting story, scenes and audio | Missing | Correct sequence, dialogue/choices/cinematics/audio and branch completion observed in client. |
| Persistence and interruption recovery | Initial character records are durable and restart relog passes; world XP/loot/heart/rejoin state is process-local only. | Save/relog and disconnect/restart preserve character, items, position and progress without duplicate rewards. |
| Human Mesmer opening and Queensdale, after Norn | Incomplete | Equivalent creation/tutorial (138)/area (15) coverage, including source-backed identification of the remembered circus/town sequence. |
| Final completion audit | Not reached | Every required item above proven with current source and runtime evidence; no placeholder counted as retail accurate. |

## Current checkpoint

Latest independently verified live kit: **82d8d04-cdn900**, prepared mode,
deployed 2026-11-05 23:56:31 UTC; kit/metadata hashes and four listeners
reverified at 2026-11-06 00:54:19 UTC. The read-only check found the same
kit and b126 launcher feed; all hashes and four owned listeners passed,
with0 human connections. Receipt:
`~/re/gw2/notes/209026-live-cdn900-independent-readback-20261106.json`,
REA `ev_5b73e33ac58e8437dbe33ce42a687c1168c6036a207f12924e36320efbb5c34c`.
The [saved creator record](re/209026-saved-norn-creator.md) covers real UI
creation, durable initial save, restart relog, visible direct world entry and
same-GUID waypoint rejoin. [Run142](re/209026-texture-remapall-private-candidate.md)
passes complete male Norn Mesmer preview rendering with no new assertion.
The sole publisher deployed that tested metadata after a successful
prepared-profile smoke with the current launcher runner; the release receipt
is linked from the run142 note. Run144 then passed the combined full flow
with the actual offered runner0e6ac30, without the runner63 repair: visible
creator and returned-world avatar, unchanged initial record and no new
crash-log bytes. It carries XP/loot/heart state only within the server process.
Runner63 owner objects are uploaded, but the last
launcher readback still selected b126/0e6ac30. The publisher marked rendering
resolved and requested a roster choice; this new activation hold is being
reconciled with existing owner authorization. Normal owner character creation
must not be advertised as available from these private tests alone.

Next work preserves the release workflow while the sole publisher reassesses
activation dependencies and source-backed tutorial/skill behavior advances. The full goal
remains open; diagnostic fixture results are not retail encounter completion.

The tested server guard `5169b98` excludes Ether Feast10176 from generic
attack damage and implicit attack-target changes. All32 mission tests, build
and no-listener startup preflight passed; it is handed to the publisher,
without a newer live receipt yet. See the
[damage correction](re/209026-ether-feast-damage-guard.md).
The [literal amount reader](re/209026-mesmer-tooltip-facts.md) passes eight
focused tests and actual frozen-record checks. The
[character-backed context](re/209026-tooltip-character-context.md) grounds
its explicit fixture level/attribute inputs, while
[recharge consumers](re/209026-skill-recharge-consumers.md) distinguish
ordinary cooldowns from count replenishment. These are source foundations;
the cast transition and actual healing/recharge execution remain unfinished.

The [activation-tooltip trace](re/209026-mesmer-cast-tooltip.md) binds the
primary execute event1067 to the displayed1 through quarter-second formatting;
it does not supply an authoritative server cast duration. The
[seven-site lifecycle observer](re/209026-skill10176-lifecycle-observer.md)
and [recharge/context observer](re/209026-skill-recharge-consumers.md#prepared-passive-observation-contract)
pass28 and16 host mock tests respectively. Both are staged in the private
run145 diagnostic using the offered runner0e6ac30 and live900 metadata.
No run145 client has launched: a fresh shared-resource window from the
existing integrator remains required. The
[echo-precondition trace and fix](re/209026-skill-order-echo-preconditions.md)
replace the unlaunched Word0 experiment with commitcac0e9a's explicit
nonzero-duration option and controlled-order admission flag. Eight focused
tests and debug/static builds pass. Run145 now stages that static binary with
analyst-selected1000ms; no positive duration is claimed as10176's retail cast
time. Its1831-entry manifest is pinned, auth/mission no-listener preflights
pass, and both archive diagnostic readers are private pinned copies.
The [map gate](re/209026-skill-context-map-gate.md) independently resolves
context1 once map379 is installed, selecting the base recharge content20000
before display modifiers. Actual recharge execution remains unimplemented.

## Earlier source checkpoints

- Historical baseline: Guardian entered Queensdale and walked (run110,
  then-live kit f36bb98), before the later NPC/combat/progression fixtures.
- [Mesmer state](re/209026-mesmer-profession.md): raw messages and explicit
  candidate population path implemented and checked against client schemas.
  No retail startup values established. [Runs 111/112](re/209026-norn-mesmer-playback.md)
  rendered the Norn Mesmer in map 379 and demonstrated local movement; those
  early fixtures had empty skills and equipment.
- [Starter maps](re/209026-human-norn-starter-maps.md): Norn tutorial379/open28
  share archive187611, with game types9/3. Human tutorial138/open15 share
  archive192711, with game types9/3. Auth already preserves requested types.
- [Equipment reader](re/209026-creation-equipment.md): typed traversal and
  real-archive checks identify the shared Human/Norn level-1 Mesmer template
  across both sexes. Runtime template selection/overrides remain unresolved.
- [Skill candidates](re/209026-mesmer-palette-candidates.md): all-container
  Rust census and independent Python traversal agree on three Mesmer
  category2 palettes; default selection and skill execution remain open.
- [Norn terrain](re/209026-norn-terrain-v14.md): version-14 decoding follows
  the client upgrade callback; real Norn and Queensdale probes pass.
- [Fixture CLIs](re/209026-character-fixture-cli.md): explicit archived
  appearance extraction and bounded synthetic auth framing replace manual bytes.
- [Quest initialization](re/209026-quest-lifecycle.md): direct quest154/goal1
  link to map379, explicit 527/532/528 observer experiment and once-only guard.
  Run113 shows a story marker but blank objective text; no encounter implemented.
- [Text-key trace](re/209026-quest154-content.md): all eight quest text IDs
  require keys; the inspected no-key decoder path returns an empty string.
  Explicit747/748 codecs are ready, but valid keys remain unresolved.
- [Equipment delivery](re/209026-item-equipment-delivery.md): explicit
  armor/weapon creation, equip and set-selection frames pass source-schema
  and fixture checks. Run115 renders the starter armor after moving the batch
  before character finalization; the weapon and first weapon icon are visible.
  Its placement is inside the [first objective sector](re/209026-norn-sector-polygons.md),
  not a claimed retail spawn.
- [Health baseline](re/209026-mesmer-health-speed-baseline.md): the default
  level1 Mesmer calculation yields388; run118 displays it through an explicit
  full-health option. Starter prefix112 contributes no attributes. Other
  modifiers and movement speed remain unverified.
  [NPC foundation](re/209026-npc-character-foundation.md)
  establishes species-based character creation; actual encounter content is open.
- [Initial skills](re/209026-mesmer-starter-skill-requirements.md): explicit
  605/612 observer input adds a healing icon in run116. Selection and
  presentation pass; cast execution, effects and cooldowns remain missing.
These earlier source checkpoints preceded the later private creator tests and
prepared-mode server releases above. Synthetic fixtures remain explicitly
labeled; releases retain active-player protection, rollback and the existing
sole publisher. No owner-device action is implied by a local playback.
