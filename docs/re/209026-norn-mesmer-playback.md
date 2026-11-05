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
