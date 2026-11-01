# Scope and progression (time-locked)

Owner instruction (Devbox task 1eae43f85f70, 2026-10-31): GW2 joins the time-locked
server / store-filter plan.

- **Ordering:** develop the base game to 100% first, then each expansion in release
  order.
- **What the ordering covers:** content and progression gates on our server. The
  retained modern client (build 209026, frozen) stays the transport and rendering
  vehicle. We assume no different client and invent no era data.
- **Prerequisite:** the native connection / authentication milestone. That is the
  launcher patch step, portal login (cligate STS on 6112), then character select. See
  [STATUS.md](STATUS.md).
- **Not a readiness claim:** a catalogue tag is not "connectable". Nothing below is
  playable yet.

## Content tiers (release order)

| Tier | Content | Gate on our server |
| --- | --- | --- |
| 0 | Base game ("core Tyria"): five races, the core professions without elite specializations, personal story, the core open-world maps, dungeons, WvW / PvP basics | always on |
| 1 | Living World Season 1 remnants and Season 2 | after tier 0 sign-off |
| 2 | Heart of Thorns | after tier 1 |
| 3 | Living World Season 3 | after tier 2 |
| 4 | Path of Fire | after tier 3 |
| 5 | Living World Season 4, Icebrood Saga | after tier 4 |
| 6 | End of Dragons | after tier 5 |
| 7 | Secrets of the Obscure | after tier 6 |
| 8 | Janthir Wilds and later | after tier 7 |

How gates are enforced, inferred and still to be designed: the server refuses map
transfers into later-tier maps, hides later-tier masteries, elite specializations and
mounts, and filters vendors and loot tables by tier. Map, skill and specialization tier
membership comes from the web API data cached per build (`gw2-api`), not from invented
lists.

## Core-game completion matrix (tier 0)

Every row starts **untested**. A row moves to **passed** only with a recorded owner
playtest or a focused automated check, with the date and evidence.

| Area | Item | State | Evidence |
| --- | --- | --- | --- |
| Connect | Launcher patch step passes offline | in progress | docs/re/209026-patcher.md |
| Connect | Portal login (STS on cligate:6112) with Banshee ticket | untested | |
| Connect | Character select lists characters | untested | |
| Characters | Create a character (each of 5 races, each core profession) | untested | |
| World | Load into Queensdale (map 15) and spawn | untested | |
| World | Move, waypoint travel within Queensdale | untested | |
| World | Other core starter maps (Plains of Ashford, Wayfarer Foothills, Metrica Province, Caledon Forest) | untested | |
| World | All core open-world maps load | untested | |
| NPCs | Ambient NPCs and vendors in Queensdale | untested | |
| Combat | Weapon skills, healing, utilities, elites (core) | untested | |
| Combat | Mobs, death and downed state | untested | |
| Progression | XP, levels 1–80, traits (core lines) | untested | |
| Progression | Hearts, events, POIs, vistas, waypoints | untested | |
| Story | Personal story chapters 1–10 | untested | |
| Items | Inventory, equipment, loot, salvage, crafting (core) | untested | |
| Social | Chat, party, guild basics | untested | |
| Instanced | Core dungeons (story and explorable) | untested | |
| PvP / WvW | Basic PvP lobby and WvW maps | untested | |

Expansion work (tier 1 and later) starts only after this matrix is complete and the owner
has signed it off.
