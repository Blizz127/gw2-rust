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
