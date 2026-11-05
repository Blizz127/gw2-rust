# Build 209026: first Norn Mesmer map playback

2026-11-05. Isolated runs 111 and 112 establish character selection,
entry into **The Great Hunt (map 379)**, rendered Norn appearance and local
movement. They do not establish the retail spawn, starting loadout, tutorial
objectives, encounters, rewards or persistence.

## Explicit fixture and sources

The [appearance extractor and fixture builder](209026-character-fixture-cli.md)
selected Norn male Mesmer: race 3, sex 0, profession 7, archived default
key 57. The 45 appearance bytes have SHA-256
`b3b957c919a12d3b030b3ed4cc75c0117b2fb564c5859b18e2d97b98af7f9c4f`.
The synthetic character `Banshee Mesmer` has a 159-byte version-11 summary
and 224-byte auth record body. Preview equipment and inventory are empty.

Mission options selected map 379, position **[0,0,-1970]**, Mesmer raw
values **[1,0]**, and reward-track keys **[56,88,47,112]**. The position
is a diagnostic point near the [measured terrain surface](209026-norn-terrain-v14.md),
not the authored starting location. The Mesmer values preserve the inspected
constructor values without claiming retail initialization. The track keys
are referenced by profession-7 specialization definitions 10/1/45/23;
their selection/order is an explicit research input, not proven startup policy.
Health 1645, movement speed 294 and other bootstrap placeholders remain.

Root asset manifest 2130706600 (276 bytes) references Launcher64, English,
**MapAlpineSnowline** and **RegionAlpine**, using the actual map-definition
manifest names. Launcher64 retains the full archive asset list. The auth
route preserves the client's tutorial game type. No map/name remapping or
shipped executable patch was used.

## Observed playback

Both runs reached client **420 MapLoaded**, then **412 AgentsLoaded**.
The observer sent the explicit population, followed by **1036 BeginPlaying**,
**431 player startup** and **83 agent control**. Character selection displayed
`Level 1 Norn Mesmer` and `The Great Hunt`; the preview scene remained black.
The loading screen identified The Great Hunt, and the world rendered the
Norn character on snowy terrain near a bridge.

Run 111 issued movement input before the interface finished settling. Its
screenshots establish map rendering but are insufficient to establish
movement. Run 112 waited 35 seconds after BeginPlaying, took a baseline
screenshot, held W for four seconds, and captured the result and a short
soak. The character visibly runs forward and the nearby terrain, trees and
bridge shift relative to the camera. Client movement traffic also continues.
This establishes local movement for the fixture, not server-authoritative
collision, speed accuracy, travel or persistence.

No assertion or fatal signal was captured during either bounded run.
The final UI shows an empty/locked skill bar, unequipped character and
generic exploration direction. No tutorial quest or NPCs were sent.

Run 112 screenshot SHA-256 values:

| Screenshot | SHA-256 |
| --- | --- |
| `norn-mesmer-before-move.png` | `ac967cdb17ab5073196420c59f0778e9358c8abef18cb3393faa3a0815d80c6b` |
| `norn-mesmer-after-move.png` | `60d3b8cd04cfac19a4c7ab87c618c8cbe275a2c1e4e6e1daebb471411d1ad5f3` |
| `norn-mesmer-soak.png` | `743d793ae376cd2ff2d8b457a844f382b996a1e38e37a722985ba414472caf7f` |

## Isolation and retained evidence

Both runs used a synthetic local account with the previously accepted
agreement fixture, Docker `network=none`, and credential-safe observer
logging. Their exact containers were stopped and absence verified. Frozen
and working executable hashes remained
`99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`;
the working archive retained 1,124,232 entries. No live deployment changed.

REA captured each orchestration process and screenshot-directory hashes.
Both exited 0 with owned-process cleanup verified; inaccessible unrelated
`/proc` entries were left untouched. Screenshots were inspected separately
and support the visual observations above; successful orchestration alone
does not imply a retail-accurate session.

Private runs: `~/gw2-sandbox/runs/run111-norn-mesmer-bootstrap/` and
`~/gw2-sandbox/runs/run112-norn-mesmer-movement/`.
Bundles: `~/re/gw2/notes/209026-norn-mesmer-run11{1,2}-20261105.json`.

| REA evidence | Scope |
| --- | --- |
| `ev_2735e186d58277705b77f685ad09f4a2ebbf2d0bafcf184765062e51bbad22c8` | Run 111, map entry and rendered character. |
| `ev_819dd224eb242ce57be9ff752a4e42dccf23d055017b497805f6c25d8b099919` | Run 112, delayed movement input and retained before/after screenshots. |
| `ev_a2fdfeed4c520d4bab829025ef9f20cb947285be87f9976d2995fda838148d99` | Hash-checked profession-specialization reward-track references. |

## Quest initialization follow-up: run 113

The [quest source trace](209026-quest-lifecycle.md) establishes a direct
quest154 → goal1 → map379 reference. Run 113 adds the explicit observer
option **`--mission-quest 154,0,1`**, retaining run 112's other inputs and
delayed movement check. Flag byte zero is a research choice with the proven
completion bit clear; it is not a claimed retail default.

After the usual population/readiness/startup sequence, the observer sends
527 Create, 532 SelectGoal and 528 Notify exactly once. The client remains
in the map, renders and moves without a captured assertion or fatal signal.
The objective area changes from generic exploration guidance to a green
story star and `(Level 1)`. **Quest title and objective text remain blank**
in the before/after/soak screenshots. This verifies a bounded initialization
path, not correct objective presentation or functioning tutorial gameplay.
The missing text is an active investigation, not an accepted visual result.

REA evidence:
`ev_7100e4d3cb54b67cc4aa4a9c4a505a59c14bc9a154a155edbe5f2d80ca0eda65`.
Private run: `~/gw2-sandbox/runs/run113-norn-mesmer-quest/`.
Bundle: `~/re/gw2/notes/209026-norn-mesmer-run113-20261105.json`.
Soak screenshot SHA-256:
`af2d84133eb565d6a44e72ca479d959479b5e321120e6a693d0a877dcb40acf8`.
The run exited 0 with exact container cleanup, unchanged executable hashes,
full archive entry count and credential-safe logging verified as above.

## Equipment and first-objective sector: run 114

Run 114 adds the explicit [equipment batch](209026-item-equipment-delivery.md)
and moves the diagnostic placement to **[-3200,-25600,-3742]**. The
[sector trace](209026-norn-sector-polygons.md) verifies XY inside authored
sector201, referenced by quest154's first step, and measures a nearby flat
terrain surface. This remains an analyst-selected location, not a recovered
retail spawn or a complete collision check.

The117-byte batch uses the archived three armor items, weapon, skins,
constructor prefix112 and Norn Mesmer color rows. It creates synthetic
instances1001..1004, equips slots2/6/3/29 and selects set4. All unresolved
raw item fields are explicit research zeros, with no upgrade items and an
absent optional Guid. Batch SHA-256:
`75c98bd961c2b03731e62af98ceb78d2d8015f12a7afd7b07399433affc6a808`.

The client reaches MapLoaded/AgentsLoaded and accepts this batch followed
by the quest sequence without a captured assertion or fatal signal. The
first weapon-skill slot gains an icon; this does not establish its identity,
correct availability or execution. **The character still renders in
underwear**, so inventory delivery is not accepted as complete visible
equipment initialization. The inventory report/model/wardrobe notification
path is under investigation. Quest text remains blank.

The new position renders a camp area and supports local movement. After the
four-second forward input, the camera is obstructed by a cart. Distant visual
artifacts also remain in the pre-movement screenshot. This run supplies no
full scene-quality or collision acceptance. These observations are retained
instead of counting a successful process exit as successful equipment or
tutorial gameplay.

REA evidence:
`ev_c75e359629441b5fbabbf00ba6a86ea53589aaa0872ebd994738febdd67b78cd`.
Private run: `~/gw2-sandbox/runs/run114-norn-mesmer-equipment/`.
Bundle: `~/re/gw2/notes/209026-norn-mesmer-run114-20261105.json`.
Pre-movement screenshot SHA-256:
`163d5f356674916f810048e78865918181cce4e566f0d74b82aeb16eb7e8c0e7`.
The run exited0, its exact container was removed, both executable hashes
were unchanged, and credential-safe logging/archive retention passed.

## Equipment before finalization: run 115

The [model-order trace](209026-initial-equipment-model-order.md) supports
moving the unchanged117-byte equipment batch immediately before server170.
Run115 applies that order with the same source item references, colors,
research raw fields, quest and diagnostic position as run114. It does not
add an inventory report or wardrobe update. The UI check waits after
BeginPlaying and captures the character stationary instead of walking into
the cart.

**The starter armor is now visible.** The inspected ready/soak screenshots
show purple torso, leg and foot armor; the soak also shows the equipped
weapon. The first weapon-skill icon remains visible. This verifies initial
model consumption of the supplied equipment for this fixture. It does not
establish all raw item values, retail template selection, weapon execution
or healing. Quest text remains blank, and the distant blue scenery artifacts
remain visible; neither is accepted as correct presentation.

REA orchestration evidence:
`ev_7a33d45eb65ff775d8e1b5cf862bb4dd54f8b15f07334af22d916dde496af707`.
The run reached MapLoaded/AgentsLoaded/BeginPlaying, exited0, and verified
exact container cleanup, unchanged executable hashes, full archive retention
and credential-safe logging. Post-run audit
`ev_5744b9d7f6900c35d4f57a9288908cd113ae6b3c4aee8b56bfcebde61725d218`
found no captured runner assertion/fatal or SIGSEGV/SIGABRT marker and bound
the unchanged equipment digest to these screenshots:

| Screenshot | SHA-256 |
| --- | --- |
| `norn-mesmer-gear-ready.png` | `fd0d7813a09670beff1e3a5cbe2fa535a9e9228718799fe3088f621db5a8e9bb` |
| `norn-mesmer-gear-after-wait.png` | `21fc86947df4c8f30fe661e2256a2275f11a385770923360bb84f90674b190d9` |
| `norn-mesmer-gear-soak.png` | `3672dc2900893f3cfe52f5e83eabbfd0de24bc1bed7f3f3ec8c6f71ab1fe143d` |

Private run: `~/gw2-sandbox/runs/run115-norn-mesmer-initial-equipment/`.
Bundle: `~/re/gw2/notes/209026-norn-mesmer-run115-20261105.json`.
Three focused equipment tests pass, including byte preservation before170,
quest deferral until readiness, and duplicate-readiness rejection. The musl
observer used for playback was rebuilt with this ordering change.

## Explicit healing selection: run 116

Run116 adds the [traced skill batch](209026-mesmer-starter-skill-requirements.md):
605 grants skill10176, then612 selects palette366 in standard slot0/locale0
for player1. The12-byte batch is sent once after1036/431/83. Equipment,
quest, diagnostic spawn and other bootstrap inputs remain as in run115.
No additional weapon skill availability is granted.

The inspected ready and post-input screenshots show **an icon in the
healing slot**, alongside the existing first weapon icon and rendered
starter armor. This establishes selection/presentation for the explicit
candidate. The script sends key6 after the ready screenshot; the server
console subsequently frames three client23 messages of15/23/15 bytes.
Their action payload semantics have not been decoded here. No server
skill execution, health effect or cooldown is implemented, and neither
input delivery nor an icon is counted as a successful cast.

Quest text and scenery artifacts remain unresolved. REA orchestration
`ev_2206d42345f027207c17fe22a29f1b0b06da09f9916dd0d67e544b04c2b65543`
reached MapLoaded/AgentsLoaded/BeginPlaying and exited0. Exact container
cleanup, frozen/working executable hashes, archive retention and
credential-safe logging passed. Audit
`ev_26d6352d7f3ee47cadaa9c4f2f7426dc182cf024b1503c917ccb2f91b3a6fd70`
found no captured assertion/fatal marker, retained the input digest and
bound these screenshots:

| Screenshot | SHA-256 |
| --- | --- |
| `norn-mesmer-skills-ready.png` | `808f9c33fe462fd6af725119b80ddb8cc2d291ff9be8a0b46ec68a05f1ba7052` |
| `norn-mesmer-skills-after-key6.png` | `669722d623bc15a4fb90fff1836974fdaa3d1bd7951f655b76ab81b626e55f7b` |
| `norn-mesmer-skills-soak.png` | `e46319ad6a144f5050fa2b284f655d8360ef66181efeee0c847f87b21a5eb437` |

Private run: `~/gw2-sandbox/runs/run116-norn-mesmer-skill-selection/`.
Bundle: `~/re/gw2/notes/209026-norn-mesmer-run116-20261105.json`.

## Explicit health baseline: run 118

Run118 retains run116's equipment/skill/quest inputs and adds
`--mission-health 388`. The [equipment-health trace](209026-mesmer-starter-equipment-health.md)
supports this explicit full-health, unmodified level1/default-curve profile.
The inspected ready screenshot shows **388** in the health globe, alongside
starter armor and the weapon/healing icons. This verifies transport and
presentation of that value, not all stat modifiers or functional healing.
The remaining auxiliary health inputs, endurance and movement parameters
retain their earlier research status.

REA orchestration:
`ev_6c72a68c21a9c0c650c87a5de3b9bf80ebc062f4bd1512218682a6cc6bf337f0`.
The run reached readiness, exited0, removed its exact container, retained
both executable hashes and full archive, and passed credential-safe logging.
Audit `ev_31639faf26d78e37f7385fd3ad513148d047e33a7b9f79d67ee665fc964a4773`
found no captured assertion/fatal marker and bound the explicit option to:

| Screenshot | SHA-256 |
| --- | --- |
| `norn-mesmer-health-ready.png` | `4c49022e61f10433488afd8b15debcdb35807c1c696225a3780704c92447fe51` |
| `norn-mesmer-health-after-key6.png` | `a71f3c0952ff7e718990931215ff4f771fc6823b163ab5b910759312a4a3003e` |
| `norn-mesmer-health-soak.png` | `4daf401264c3aab9b58b7fcdd4ffa74b4e43f829dbf7c233065f33626cdbbb32` |

Private run: `~/gw2-sandbox/runs/run118-norn-mesmer-health-baseline/`.
Bundle: `~/re/gw2/notes/209026-norn-mesmer-run118-20261105.json`.
