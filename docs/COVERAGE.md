# GW2 player-experience coverage record

Devbox task a314b17e43a8 (owner: "it needs to include everything the human sees").
Updated 2026-11-01 for build 209026.

Status values:
- **PASS (client)**: the real frozen client, run offline in the sandbox, visibly reached the stated screen or behavior
- **HEADLESS**: offline codec/unit evidence only; no client playback
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

## In-game flows (all after the User Agreement)

| Flow | Status | Evidence | Gap |
| --- | --- | --- | --- |
| Character select list | HEADLESS | Character record/summary codecs, `gw2-summary-check` | No client display yet |
| Character creation (UI, cinematics, answers) | HEADLESS | Creation frame checker, answer/color/equipment notes | Creation cutscenes NOT RUN |
| Play → route → game join | HEADLESS (static RE) | [play route](re/209026-play-route.md), [mission join](re/209026-mission-join.md) | No server implementation |
| Map load / loading screen | NOT RUN | [map ready](re/209026-map-ready.md) states | Needs game server |
| Spawn, camera and control return | NOT RUN | Agent/character codecs | Needs game server |
| Cutscenes / dialogue / choices | NOT RUN | — | Needs game server and content |
| Movement / combat | NOT RUN | Collision and terrain readers (offline) | — |
| Inventory / rewards | NOT RUN | Inventory create codec only | — |
| Quests (accept, progress, complete) | NOT RUN | Quest startup static RE | — |
| Travel / map transfer | NOT RUN | Transfer messages mapped statically | — |
| Save / relog / interruption recovery | NOT RUN | No persistence exists | — |
| Audio | NOT RUN | Sandbox has no audio capture | — |

## Next coverage increment

1. Once the hold is lifted: an automated sandbox run that captures screenshots at each
   launcher state, checked against stored reference frames. Visual checks so far
   were not automated.
2. Character select display from a fixture character list (first in-game screen).
3. Each later flow becomes runnable only as the game server reaches it. Rows move from
   NOT RUN to HEADLESS to PASS (client) with run numbers.
