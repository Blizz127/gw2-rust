# Owner Norn Mesmer combat release 3cf2483

Published to the existing Banshee GW2 service on 2026-11-05. The owner asked directly
in the Claude gw2-rust terminal ("also deploy the latest version"), under the standing
continuous-release order. The watched cycle slept the service with 0 humans at
16:52:04Z, switched the link while it was stopped at 16:52:09Z, and reported ready
3.3 s after start at 16:52:14Z. Afterwards 0 humans were connected.

## Artifact and rollback

- Source: `release/claude-combat-20261105` at `3cf2483`. This is main `1d5b3f5` merged with
  `claude/npc-talk-loop` at `28d3154`, plus the deploy profile variables.
- Live kit: `/home/blizz/gw2-rust-3cf2483`; `gw2-rust-current` points there.
- Rollback: `/home/blizz/gw2-rust-b7fbd8c`, retained unchanged.
- Game binary SHA-256: `57d49c2c91bc7d0e1bc6e103eda69d0d014b4c2574031bebb28741bda7107c4e`.
- SHA256SUMS SHA-256: `13ff337dcd5bc835f9243e1a0ded1a533a2a39450d951c6103bc0150b6849a8c`.
- All 39 kit files passed `sha256sum -c` locally and again on Banshee before the switch.
- Private local kit: `/home/blizz/scratch/gw2-kit-3cf2483`.
- Static musl build: `ring` needs `CC_x86_64_unknown_linux_musl` set to a `zig cc -target
  x86_64-linux-musl` wrapper that drops cc-rs's `--target=` argument.

## Profile

The b7fbd8c Norn Mesmer run118 profile is unchanged. On top of it, the research combat
loop is enabled with placeholder values:

| Variable | Value |
|---|---|
| `GW2_HOSTILE_NPC` | `1544,-3000,-25300,-3742` (Branded Minotaur) |
| `GW2_DAMAGE_ON_SKILL` | 30 |
| `GW2_AUTOATTACK_SKILL` | 10170 |
| `GW2_XP_PER_KILL` | 2500 |
| `GW2_STARTER_BAG` | 959 |
| `GW2_KILL_LOOT` | 1 |
| `GW2_NPC_ATTACK` | 40 |
| `GW2_SOURCED_DAMAGE` | 1 |

The live game process was read back with exactly these flags.

## Checks

- `cargo test --workspace`: all passed.
- Local supervisor smoke: all 4 listeners were ready, 0 humans, and the profile flags
  reached the game process.
- Sandbox run 163, with the kit's own binaries and this profile: two kills, each with XP
  and a level-up (to 2, then 3); NPC damage (health 388 → 148); two weapon drops shown in
  the Starter Backpack (2/20); the respawn; no asserts.
- Workspace clippy still fails only on the lints that came with ff70076.
- The downed/defeat/rally path is in this kit and was exercised in runs 159–161 at 200
  damage. At the live 40 damage it needs about 10 NPC hits to trigger.

The frozen client channel (build 209026) is unchanged. No launcher publication was made.

Not retail yet:
- placeholder damage, XP and loot;
- floating damage numbers aren't drawn;
- the waypoint respawn after defeat isn't implemented;
- post-load NPC spawning;
- the real starting-area NPCs, events and story.

## Update: kit 5fbb4b5 (live 2026-11-05 17:15:36Z)

- Source: `5fbb4b5`, the release branch after merging `claude/npc-talk-loop` at `5150a6f`.
- Kit: `/home/blizz/gw2-rust-5fbb4b5`, now the target of `gw2-rust-current`.
- Rollback: `/home/blizz/gw2-rust-3cf2483`.
- Game binary SHA-256: `8d9a05ca13daeb59233dc42a7acf8bdf00a8b09fe33712a92dd9bd99e39dfb34`.
- SHA256SUMS SHA-256: `04a312a79fb2fa12e47e0bf89b5795746cf7febd1ef89703fa50cf051d65fffb`.
- 39 files verified locally and on Banshee.
- The watched cycle stopped the service with 0 humans, switched the link while stopped,
  and reported ready 3.3 s after start. Afterwards 0 humans were connected.

Changes:
- level-1 NPC health 118 and NPC damage 19, from the client's damage model (the profile
  now sets `GW2_NPC_ATTACK=19`);
- real kill loot through 968 + 276;
- the downed health pool (259/260) with rally restore;
- 255 floater amount Word.

Checks:
- `cargo test --workspace` passed.
- Sandbox kit smoke run 167 with the kit binaries: two kills with level-ups; Broken Claw
  and Tiny Claw shown in the backpack (2/20); respawn at 118; NPC hits of 19; no asserts.
- The live game process was read back with `--mission-npc-attack 19`.

## Update: kit d324165 (live 2026-11-05 17:32:45Z)

- Source: `d324165`, from merging `claude/npc-talk-loop` at `33d661d`, plus deploy support
  for several hostile NPCs.
- Kit: `/home/blizz/gw2-rust-d324165`.
- Rollback: `/home/blizz/gw2-rust-5fbb4b5`.
- Game binary SHA-256: `bab25389a6545eeab594f1b7cc965bc14106b52e91d2442bc9c4bfab0e62ab18`.
- SHA256SUMS SHA-256: `0c4699a4d84a2331e7e9135fed6136c9dbf5b2cedfc93ea77d0316520f69b359`.
- 39 files verified locally and on Banshee.
- The watched cycle ran at 0 humans and reported ready after 3.3 s.

Changes:
- Four hostile NPCs, read back from the live game process: Branded Minotaur
  `1544,-3000,-25300,-3742`, Wolf `4079,-2900,-25850,-3742`, Juvenile Boar
  `8013,-3500,-25350,-3742`, Son of Svanir `7200,-3400,-25900,-3742`.
- Each NPC has its own health, respawn and aggro.
- Auto-attack follows the client's target selection (227) and only hits living NPCs.

Checks:
- `cargo test --workspace`: 35 test targets passed, 0 failed.
- Local supervisor smoke: four `--mission-hostile-npc` flags reached the game process.
- Kit smoke run 170: four NPCs spawned; Tab to the wolf, auto-attack killed it three times
  with level-ups and loot; no asserts.

Owner play on 5fbb4b5, read from the Banshee server log:
- killed the minotaur twice (skill 10176 and auto-attack), reaching level 3;
- received Broken Claw and Tiny Claw;
- the minotaur respawned at 118;
- the session ended with a normal client EOF.

The log also showed auto-attack hitting the dead minotaur (`health 0 -> 0`). The
multi-NPC targeting in this kit only hits living NPCs.

## Update: kit 76fe405 (live 2026-11-05 17:52:45Z)

- Source: `76fe405` = combat release `d324165` merged with main `9dc75e2`, which includes the
  creator lane's tested `b8afb71`:
  - opt-in shared creation/world mission listener;
  - 1065 first-join scalar set to 0;
  - auth replies grouped at message boundaries.
- One conflict in `gw2-game-observe.rs`. Resolved by keeping the combat fixture block and
  then main's relocated CreationScene construction.
- Kit: `/home/blizz/gw2-rust-76fe405`.
- Rollback: `/home/blizz/gw2-rust-d324165`.
- Game binary SHA-256: `aac306f11307480a59d5d7c1a6f176e34ad37374f9fb7ea87e3682640dd02eef`.
- SHA256SUMS SHA-256: `d1064d5d1100a275e67eea28aa1bc9f37937e491aa5f97623ae3d155d67fa621`.
- 39 files verified locally and on Banshee.
- The supervisor is unchanged, so the creator path stays disabled.

Checks:
- `cargo test --workspace` passed. `gw2-game-observe` passed 101 tests, including the
  creator lane's 14 focused tests.
- Local supervisor smoke: 4 listeners ready and 4 hostile-NPC flags reached the game
  process.
- The sandbox kit smoke (run 173) runs after the deploy, as a diagnostic.
- Known: combining combat flags with the creator's map-0 phase aborts. This affects only
  the opt-in creator path. The creator lane is preparing a phase-scoped fix.

## Update: kit 3d079ad (live 2026-11-05 18:00:18Z)

- Source: `3d079ad`, the creator lane's `46c5beb` cherry-picked onto `27a4a2c`.
- Fix: a combined creator join now skips the world combat setup, and a saved world keeps
  every combat option.
- Kit: `/home/blizz/gw2-rust-3d079ad`.
- Rollback: `/home/blizz/gw2-rust-76fe405`.
- Game binary SHA-256: `23c7d5ba15f5a048ad6c3ec0b747918bead3be3ca524252e95b6c5e8307267ba`.
- SHA256SUMS SHA-256: `c03a36882f0c67fbe756967bdf74820033bbdece612d1c8a16fbbeb7c98fcfe7`.
- 39 files verified locally and on Banshee.

Checks:
- `cargo test --workspace` had no failures; the 6 `shared_creation` tests passed.
- Supervisor smoke: 4 listeners ready, 4 hostile-NPC flags.
- Post-deploy diagnostic run 173 on the 76fe405 kit binaries:
  - default login and BeginPlaying worked;
  - four NPCs spawned and 227 targeting worked;
  - kills, loot and level-ups;
  - no asserts.
- The watched cycle ran at 0 humans and reported ready after 3.3 s. The creator stays
  disabled.

## Update: kit 56242d1 (live 2026-11-05 18:06:54Z)

- Source: `56242d1`, the creator lane's `08b8c55` cherry-picked cleanly onto `72cca31`.
- Change: an explicit, host-only saved-Norn supervisor mode with private-store checks and
  a `--check-config` preflight. It adds `deploy/gw2-saved-store.py`. The default prepared
  argv is unchanged, and saved mode is not enabled.
- Kit: `/home/blizz/gw2-rust-56242d1`, 40 files.
- Rollback: `/home/blizz/gw2-rust-3d079ad`.
- Game binary SHA-256: `7866799636831581f899ebc8470836eaf6da34e88381cab8f7ec569436ac6a1e`.
- `gw2-saved-store.py` SHA-256: `03e982d9ff7389eaf5d69dded8b4958db29491e560e5f89aede4ea3ee65bbcc6`.
- SHA256SUMS SHA-256: `1f781e476bc38c2c6366742c9ed7bf8af0d3f65b91b4f95265009a69cc3b1296`.
- Verified locally and on Banshee.

Checks:
- `deploy/test_saved_mode.py`: 6/6 passed.
- `check_config`: 1/1 passed.
- Workspace tests: no failures.
- Supervisor smoke (prepared default): 4 listeners ready, 4 hostile-NPC flags.
- The watched cycle ran at 0 humans and reported ready after 3.3 s.

## Update: kit 1adda67 (live 2026-11-05 18:20:25Z)

- Source: `1adda67`, which merges `claude/npc-talk-loop` at `abe45d6` onto `f224348`. The
  merge resolved a conflict by keeping the creator lane's phase-scoped combat block and
  adding the heart and late-NPC options inside it. This merge commit also carries the
  `GW2_HEART` deploy variable.
- Kit: `/home/blizz/gw2-rust-1adda67`, 40 files.
- Rollback: `/home/blizz/gw2-rust-56242d1`.
- Game binary SHA-256: `15144676db6f370ceadd7b4c7f33d7468b312256a0af764afbf60847ed77fae6`.
- SHA256SUMS SHA-256: `5a9e1efd94d6a68c06fd2a02d49c3a8908539bd706ffd4659a842a1df2613960`.
- Verified locally and on Banshee.

Change: renown heart 56 "Honor the Spirit of Raven".
- It is shown at spawn (651, 1090, 657, 655) and goes up by one with each NPC kill (1089).
- It completes at 5 kills (654), with the client's completion toast.
- The profile adds `GW2_HEART=56,5`, and the live game process was read back with
  `--mission-heart 56,5`.

Checks:
- `cargo test --workspace`: no failures.
- saved-mode tests 6/6; `shared_creation` 6/6.
- Sandbox run 174, branch build at 3 kills: the heart was shown, advanced and completed.
- Supervisor smoke passed.
- The watched cycle ran at 0 humans and reported ready after 3.3 s.

Run 174 also showed a floating damage number ("30") on screen.

## Update: kit f751c25 (live 2026-11-05 19:16:10Z)

- Source: `f751c25`, which merges `claude/npc-talk-loop` at `211b4fc`. The merge resolved
  the conflict inside the phase-scoped block and also carries the `GW2_RESPAWN` deploy
  variable.
- Kit: `/home/blizz/gw2-rust-f751c25`, 40 files.
- Rollback: `/home/blizz/gw2-rust-1adda67`.
- Game binary SHA-256: `3512cebfe540aa29bf65261bfc018446cd8f29e8442f5079960dabc70c4f16e7`.
- SHA256SUMS SHA-256: `1f6b232ca65e545afbe439d31213c6974df971b22b3fd15ac0009ba09db493c1`.
- Verified locally and on Banshee.

Changes:
- The defeat screen's "Return to a Waypoint" now revives the player in place. On defeat
  the server sends `169 {char, 1}`; the client then sends 210, answered with 0x49 and full
  health.
- `m_inventorySlots` is now 160, because the client scans 5 × 32 slots on defeat.
- The heart-NPC marker and dialogue code is included but inactive, since there is no talk
  NPC in this profile.
- The profile adds `GW2_RESPAWN=1`, and the live game process was read back with
  `--mission-respawn-flag`.

Checks:
- `cargo test --workspace`: no failures.
- saved-mode tests 6/6; `shared_creation` 6/6.
- Sandbox run 179: defeat → 210 → revived; the inventory still showed 20 cells; no asserts.
- Supervisor smoke passed.
- The watched cycle ran at 0 humans and reported ready after 3.3 s.

## Update: kit 018cb96 (live 2026-11-05 19:22:47Z)

- Source: `018cb96`. It is the creator lane's `d175bd5` (heart configuration validated
  before binding) cherry-picked as `2fab752`, plus a test fix: with a talk NPC, the saved
  world's heart opening is followed by the 384 marker.
- Kit: `/home/blizz/gw2-rust-018cb96`.
- Rollback: `/home/blizz/gw2-rust-f751c25`.
- Game binary SHA-256: `f539827260514d0be8a7dc241e704c36a08729968a0a0be30e6aa3f6266a04ce`.
- SHA256SUMS SHA-256: `36097b8816ed35d48bef3ca0dd1ecf59d97efdf49c7c6c5f0d5f4b7fbd29f3fc`.

Checks:
- workspace tests ok; `shared_creation` 6/6; `check_config` 1/1; saved-mode 6/6.
- Supervisor smoke passed.
- The watched cycle ran at 0 humans and reported ready after 3.3 s.

The prepared profile is unchanged.

## Update: kit 9961b56 (live 2026-11-05 19:43:41Z)

- Source: `9961b56`, which merges `claude/npc-talk-loop` at `34f3bc5` and adds the
  `GW2_RESPAWN_POINT` and `GW2_REJOIN_ROUTE` deploy variables.
- Kit: `/home/blizz/gw2-rust-9961b56`.
- Rollback: `/home/blizz/gw2-rust-018cb96`.
- Game binary SHA-256: `ba65788f09a2dfc3ec8fd7fc98cea0337aa415a96ba25ffc934ef4dfb57ba275`.
- SHA256SUMS SHA-256: `c56c54a51b95b49258ab890f9f857b51eb8ffbd1c03772add930cd6c0cd6c15a`.
- 40 files verified locally and on Banshee.

Change: the defeat respawn now rejoins the map at Horncall Waypoint (POI 372) through
1065 to `127.1.0.4:6112`, keeping XP, loot and heart progress. The profile adds
`GW2_RESPAWN_POINT=-9057.7,-24303.5,-3736.1`, and the live game process was read back with
`--mission-rejoin-route 127.1.0.4:6112`.

Checks:
- workspace tests ok; saved-mode 6/6; `shared_creation` 6/6.
- Sandbox run 182 passed.
- Supervisor smoke passed.
- The watched cycle ran at 0 humans and reported ready after 3.3 s.

## Update: kit 9961b56-hm (live 2026-11-05 19:58:44Z)

- This is a profile-only update of 9961b56; the binaries are identical.
- The respawn point is now **Hero's Moot Waypoint**: POI 371, `597.3,-30352.1,-1563.2`,
  inside heart 56's area and closer to the starting NPCs.
- In sandbox run 183 the defeat → 210 → 1065 rejoin put the player standing on a hillside
  by archery targets, at full health with the heart at 1/5, and no asserts.
- Kit: `/home/blizz/gw2-rust-9961b56-hm`; SHA256SUMS SHA-256
  `8c67470ce5cbda0c6cfdf0e7d2c86d8026e5b0e79b386f6dda0531cdb6366ef1`.
- Verified locally and on Banshee.
- Rollback: `/home/blizz/gw2-rust-9961b56`.
- The watched cycle ran at 0 humans and reported ready after 3.3 s. The live game process
  was read back with `--mission-respawn-point 597.3,-30352.1,-1563.2`.

## Update: kit bc997f0 (live 2026-11-05 20:16:47Z)

- Source: `bc997f0`. The profile is unchanged from 9961b56-hm (Hero's Moot respawn).
- Kit: `/home/blizz/gw2-rust-bc997f0`.
- Rollback: `/home/blizz/gw2-rust-9961b56-hm`.
- Game binary SHA-256: `9c56bfc9b7a3d5859b0a62f28bbdd641bf1f9b37dd3051938644b66a89b515fe`.
- SHA256SUMS SHA-256: `80d32acbd8fd42f7a1b71bff502b5fd7674182c46a2e9e9aa56b145fb2c1b662`.
- Verified locally and on Banshee.

Change: heart completion now sends the 556 type-14 reward panel (coin 30 and karma 50
are placeholders) and +5147 XP.

Checks:
- workspace tests ok; saved-mode 6/6; `shared_creation` 6/6.
- Sandbox run 184: reward icon and level-ups, no asserts.
- Supervisor smoke passed.
- The watched cycle ran at 0 humans and reported ready after 3.3 s.

## Update: kit d7252a8 (live 2026-11-05 20:27:13Z)

- Source: `d7252a8`, the creator lane's `1e4e3e1` cherry-picked cleanly onto `bc997f0`.
- Fix: a heart's issued reward now survives the waypoint rejoin carry-over, so 556 and the
  +5147 XP are no longer awarded twice.
- Kit: `/home/blizz/gw2-rust-d7252a8`.
- Rollback: `/home/blizz/gw2-rust-bc997f0`.
- Game binary SHA-256: `3dfc144dfd949590d9a78caf27ea093f1bbcc1df5814538d6e5fd2f807ac2667`.
- SHA256SUMS SHA-256: `aab3cbd3bffa5ab23052bc3a85f5c91eace3499743ec877c5c85eb802554280e`.
- Verified locally and on Banshee.

Checks:
- workspace tests had no failures.
- saved-mode 6/6; `shared_creation` 6/6.
- Supervisor smoke passed.
- The watched cycle ran at 0 humans and reported ready after 3.4 s.

## Update: kit 0846fcb (live 2026-11-05 20:40:04Z)

- Source: `0846fcb`.
- Kit: `/home/blizz/gw2-rust-0846fcb`.
- Rollback: `/home/blizz/gw2-rust-d7252a8`.
- Game binary SHA-256: `364320c8c88b1af25b420998d1afc9a54658319e1ec709100308a09beb9c10e9`.
- SHA256SUMS SHA-256: `0c75892f569525076cd433a7e40aa2872f4837efc9f29e44a9a9ab053e5bf881`.
- Verified locally and on Banshee.

Changes:
- Heart 56's area now comes from its client polygon. The tracker opens when the player
  enters the area and hides when they leave (client 18/19 movement → 655/656).
- The profile moves the four NPCs inside the heart area, east of the spawn:
  `1544,-300,-26800,-3800 4079,-200,-26300,-3800 8013,-800,-26750,-3800 7200,-700,-26200,-3800`.

Checks:
- workspace tests had no failures.
- saved-mode 6/6; `shared_creation` 6/6.
- Sandbox run 185 passed.
- Supervisor smoke passed.
- The watched cycle ran at 0 humans and reported ready after 3.4 s.

## Update: kit cd4c3b6 (live 2026-11-05 20:58:54Z), fixing a 0846fcb regression

- **Bug in 0846fcb:** client 18/19 movement positions are in collision space
  `[X/32, Y/32, -Z/32]`. Run 186 showed the spawn `0,-26300,-3800` reported as
  `0,-822,118`. 0846fcb read them as map units, so the player always tested "outside"
  heart 56 and the tracker never opened on live. The fix converts them back to map units.
  The kit also includes the creator lane's heart-area regression (`d21d9c6`).
- The Banshee server log had only 5 new lines while 0846fcb was live, so no session
  appears to have used it.
- Kit: `/home/blizz/gw2-rust-cd4c3b6`.
- Rollback: `/home/blizz/gw2-rust-0846fcb`.
- Game binary SHA-256: `3fc2b6306c0162ed6989555b68a8ad58fb1c5a46e3ff34565b6d1411a877be13`.
- SHA256SUMS SHA-256: `eb96eaa642addc8f35a09574742dfd824ab788c5ab02121bcd0851c91ad02a73`.

Checks:
- workspace tests had no failures; `heart_area` 2/2; saved-mode 6/6.
- Supervisor smoke passed.
- The watched cycle ran at 0 humans and reported ready after 3.3 s.
- A live crossing check (run 187) follows.

## Update: kit 82d8d04 (live 2026-11-05 22:27:03Z)

- Source: `82d8d04`, which merges `claude/npc-talk-loop` at `27e58c7` and adds the
  `GW2_AMBIENT_NPC` deploy variable.
- Change: three Juvenile Ravens (species 6045) stand in the Spirit of Raven heart area, at
  `150,-26150 250,-26350 100,-26450` (z -3800). They are spawned at load and are neutral.
- Kit: `/home/blizz/gw2-rust-82d8d04`.
- Rollback: `/home/blizz/gw2-rust-cd4c3b6`.
- Game binary SHA-256: `10bd95cf90858ace19de8040311d7ac74bce403dfee92bf93b10f997035eee29`.
- SHA256SUMS SHA-256: `650b7d58f1a1e933a18d4b1940cff73da638a650012a6bd3e26e2f5cbec7bf03`.

Checks:
- workspace tests had no failures; saved-mode 6/6.
- Sandbox run 194 passed.
- Supervisor smoke: 3 ambient flags reached the game process.
- The watched cycle ran at 0 humans and reported ready after 3.3 s.

Independent creator-lane readback at **2026-11-05 22:30:53 UTC** confirmed
the current symlink to this kit, every `SHA256SUMS` entry, both hashes above,
and ready/running health with all four listeners owned by their recorded
processes (zero humans). Evidence:
`ev_3b32ef7315c1aee81d1c9915a1f366f63ad82e014fc9c3be4b315ad987de94b1`,
retained in `~/re/gw2/notes/209026-live82d8d04-independent-readback-20261105.json`.
This readback did not restart or change the service.

## Update: kit 82d8d04-cdn900 (live 2026-11-05 23:56:31Z)

- Change (data only): the creator lane's full 26322-remap patch metadata. It adds
  `data/cdn-files/2130706900`–`2130706904`. `data/latest.txt` becomes
  `209026_0_0_2130706900_276`. Server binaries are byte-identical to kit 82d8d04.
- Source and evidence: `docs/re/209026-texture-remapall-private-candidate.md` on main.
  Receipt `/home/blizz/re/gw2/notes/remapall-cdn-20261105-receipt.json`, SHA-256
  `3ff42843e4dc6bbb2608a7cf9c5454aa698317ba6e6685710edd498477841292`.
- Kit: `/home/blizz/gw2-rust-82d8d04-cdn900`.
- Rollback: `/home/blizz/gw2-rust-82d8d04` (root 2130706600).
- SHA256SUMS SHA-256: `ca037744d43485eb298ba3385965fb35a5ac043424c704d54fe16365bfa84741`
  (44 files; verified locally and on Banshee).

Checks:
- Creator sandbox run 142 (runner `63bc59a`): complete Norn Mesmer preview, no
  mask-size assert.
- Dependency smoke, sandbox run 202, used the live launcher's runner `0e6ac30` (gdb
  `8d5b4be7…`). It ran the kit binaries and CDN files with the live profile. The client
  fetched 2130706900–2130706904. Character select showed the fully textured Norn
  Mesmer, and the world loaded. A kill gave XP, a level-up, loot and heart progress
  1/5, and the NPC counter-attack worked. No asserts and no crash log.
- The watched cycle ran at 0 humans and reported ready after 3.3 s.
- Live readback on Banshee: `/latest64/101` = `209026 0 0 2130706900 276`. Files
  2130706900, 2130706901 and 2130706902 match the receipt hashes. The game binary is
  `10bd95cf…`.

Not changed: saved-Norn activation stays held, and the launcher feed (b126 pins runner
`0e6ac30`) belongs to the launcher lead. On first connect, the owner's client
downloads the new 13.9 MB Launcher64 metadata through its normal patcher.
