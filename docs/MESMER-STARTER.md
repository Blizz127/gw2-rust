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
| Norn Mesmer creation/selection and correct appearance | Partial: archived Norn Mesmer appearance renders in isolated playback; actual creation missing | Real client creation, biography/appearance/equipment validation, saved character and relog. |
| Correct Mesmer components and startup skills/equipment | Partial: explicit Mesmer component and source-backed item delivery; run114 shows first weapon icon but armor is not visible | Client-backed defaults/selection rules, real skill bar and equipped items, correct unlocks and stats. |
| Norn opening tutorial (Great Hunt, key 379) | Isolated map entry and local movement pass at diagnostic position; gameplay missing | Source-backed spawn, intro/dialogue, NPC encounters/objectives/boss, rewards and tutorial exit in client. |
| Wayfarer Foothills (key 28) terrain, movement and travel | Shared terrain loaded through Great Hunt; open map 28 not played | Correct entry/spawn, collision, movement, map transitions, waypoint use/death recovery in client. |
| Mesmer combat and profession mechanics | Missing | Relevant weapons, attacks, healing, clones/phantasms/shatters, cooldowns, conditions/boons, target/death/downed behavior verified in encounters. |
| Starting-area NPCs, enemies, vendors and interactions | Missing | Client-backed population/identity/placement plus exercised dialogue, shops, combat, loot and respawns. |
| All starting-map hearts, events and discovery objectives | Missing | Source-backed inventory of objectives, implemented outcomes/rewards, completion counts and client playback for every objective. |
| XP, leveling, skills and equipment progression within area | Missing | Correct rewards/stat progression/unlocks and inventory changes across the starting experience. |
| Relevant starting story, scenes and audio | Missing | Correct sequence, dialogue/choices/cinematics/audio and branch completion observed in client. |
| Persistence and interruption recovery | Missing | Save/relog and disconnect/restart preserve character, items, position and progress without duplicate rewards. |
| Human Mesmer opening and Queensdale, after Norn | Incomplete | Equivalent creation/tutorial (138)/area (15) coverage, including source-backed identification of the remembered circus/town sequence. |
| Final completion audit | Not reached | Every required item above proven with current source and runtime evidence; no placeholder counted as retail accurate. |

## Current checkpoint

- Existing baseline: Guardian can enter Queensdale and walk (sandbox run 110;
  live kit f36bb98). No existing NPC/combat/progression implementation.
- [Mesmer state](re/209026-mesmer-profession.md): raw messages and explicit
  candidate population path implemented and checked against client schemas.
  No retail startup values established. [Runs 111/112](re/209026-norn-mesmer-playback.md)
  render the Norn Mesmer in map 379 and demonstrate local movement; skills and
  equipment remain empty.
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
  and fixture checks. Run114 gains a first weapon icon; visible armor remains
  missing. Its placement is inside the [first objective sector](re/209026-norn-sector-polygons.md),
  not a claimed retail spawn.
- Next source work: Norn start/entry coordinates, creation-template equipment,
  Mesmer palette/availability selection, then first tutorial encounter.

No live service or owner-device changes were made in this checkpoint. Local
synthetic fixtures remain explicitly labeled; deployment/playback follows a
concrete usable increment, retaining active-player protection.
