# GW2 player-experience coverage record

Devbox task a314b17e43a8 (owner: "it needs to include everything the human sees").
Updated 2026-11-06 for frozen build 209026. Current owner scope is the Norn
Mesmer starting experience first, Human afterward, using the client as the
source of truth. Working protocol fixtures are not proof of retail fidelity.

Status values:
- **PASS (client)**: the real frozen client, run offline in the sandbox, visibly reached the stated screen or behavior
- **HEADLESS**: offline codec/unit evidence only; no client playback
- **PARTIAL (client)**: the client exercised the stated subset; the named gap remains
- **FAIL**
- **NOT RUN**

A request/response match or a parsed frame is never counted as visible playback.

## Launcher, login and account flows

| Flow | Status | Evidence | Gap |
| --- | --- | --- | --- |
| Launcher start offline, patch check | PASS (client) | Run 35: 100%, PLAYABLE, READY, login form, build 209026 ([patcher](re/209026-patcher.md)) | No automated visual assertion |
| Portal TLS handshake | PASS (client) | Run 39 ([login TLS](re/209026-login-tls.md)) | Uses guarded in-memory trust substitution |
| Login (GetHostname, SSO, game accounts, token) | PASS (client) with synthetic fixtures | Runs 44–47 | Fixture identity only; no credential validation |
| Binary auth handshake and account data | PASS (client) with synthetic fixtures | Runs 48–55; all captured bytes match client plaintext | `s_configDataError` crash fixed by ID 10 (run 55) |
| Email verification and 2FA reminder screens | PASS (client) | Runs 56–58 | Text/layout not compared to retail |
| News panel and Play button | PASS (client) | Run 58 | — |
| User Agreement panel | Displayed (run 59); **interaction NOT RUN** | Run 59 | **Owner hold**: no accept/decline without explicit owner direction |

The agreement row records an actual UI-interaction hold. Later authorized
synthetic sessions use a server-side preaccepted agreement fixture and do not
click Accept or Decline. The historical hold is not a claim that those later
sessions or their evidence do not exist. Current release authority and real
holds are recorded in [AGENTS.md](../AGENTS.md).

## In-game flows in the authorized private fixtures

| Flow | Status | Evidence | Gap |
| --- | --- | --- | --- |
| Character select list | PARTIAL (client) | Saved-character auth/relog in runs126/127; [creator record](re/209026-saved-norn-creator.md) | Full visual list behavior, multiple-character management and retail account semantics remain unverified |
| Character creation (UI, cinematics, answers) | PARTIAL (client); complete preview passes | Run142 passes male Norn Mesmer preview; run144 passes combined creator/save/world/waypoint with the offered runner and corrected metadata; [creator record](re/209026-saved-norn-creator.md) | Authored cutscenes remain unverified; live saved-mode activation awaits the existing native owner roster choice |
| Play → route → game join | PASS (client) for fixture | Run134 actual415/1048 save,430/1065 transfer and visible first world entry; [pipeline](re/209026-creation-model-pipeline.md) | Synthetic identity and supported Norn/Mesmer profile only |
| Map load / loading screen | PASS (client) for map379 fixture | Run134 reaches420/412/1036 on both initial entry and waypoint rejoin | Other maps, retail loading transitions and complete authored population remain unverified |
| Spawn, camera and control return | PARTIAL (client) | Run134 visible player on direct entry and waypoint rejoin; [release record](re/209026-combat-release-3cf2483.md) | Real tutorial entry position, camera/intro sequence and complete population remain unproven |
| Cutscenes / dialogue / choices | PARTIAL (client) for fixture UI only | Creator choices and separate talk/interaction fixtures; [fixture release record](re/209026-combat-release-3cf2483.md) | Authored Great Hunt dialogue, cinematics and encounter choices are not implemented or verified end to end |
| Movement / combat | PARTIAL (client) | Walking fix and combat releases; run134 kill, retaliation, downed state, defeat and waypoint recovery | Fixture actors/damage/XP do not establish retail encounters, full Mesmer skills or combat balance |
| Inventory / rewards | PARTIAL (client) | Run134 delivers one Broken Claw and XP/level change; same-process carry after rejoin | Full inventory/equipment interactions, retail reward values and durable progression remain incomplete |
| Quests (accept, progress, complete) | PARTIAL (client) for fixture heart | Heart56 tracker/reward fixtures and reward-once regression; [release record](re/209026-combat-release-3cf2483.md) | Heart56 comes from map28; its replay in tutorial379 is not the authored Great Hunt quest154 sequence |
| Travel / map transfer | PARTIAL (client) | Run134 client210/1065 returns the same saved GUID to map379 waypoint | General inter-map travel and retail travel policy remain unverified |
| Save / relog / interruption recovery | PARTIAL (client) | Durable initial character record, process-restart auth relog in127, unchanged record through134 rejoin | XP/level/loot/heart carry is process-local; durable world progression and full interruption recovery remain unfinished |
| Audio | NOT RUN | Sandbox has no audio capture | — |

## Next coverage increment

1. Preserve the verified creator/save/direct-entry/waypoint flow from run144.
   Normal owner creation is still held at the sole publisher's native roster
   choice. The separate [launcher publication record](re/209026-empty-composite-runner.md)
   distinguishes uploaded runner objects from an actual update offer; run144
   succeeded with the already offered runner0e6ac30 and corrected metadata.
2. Establish authored Great Hunt NPC identities/models, placements, encounter
   triggers and entry position. Map379/quest154 links and bounded search limits
   are in [the Great Hunt audit](re/209026-great-hunt-species-link-audit.md).
   A matching NPC name alone is not encounter membership or a spawn coordinate.
3. Replace fixture gameplay values and progression with source-backed behavior;
   verify full Mesmer skill, dialogue, inventory/reward and persistence scope.
   Record each visible check at its actual scope. Do not call the starting
   experience complete while these requirements remain open.
