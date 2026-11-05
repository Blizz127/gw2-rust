# Owner Norn Mesmer release b7fbd8c

Published to the existing Banshee GW2 service on 2026-11-05 at 03:10:43 UTC,
under owner deploy-now assignment `fa1a612f4d31d674`. Readiness returned at
03:10:48 UTC, 3.4 seconds after start. This is a prepared-character release,
not normal character creation or a completed tutorial/combat implementation.

## Published artifact and rollback

- Source: `b7fbd8c86ba4b2d2be7bf9b7b2922c8034f9ff9f`, based on tested `cd5cb78`.
- Live kit: `/home/blizz/gw2-rust-b7fbd8c`; `gw2-rust-current` points there.
- Rollback: `/home/blizz/gw2-rust-f36bb98`, retained with its original profile.
- Game binary SHA-256: `ff31dba8c4e90320e2834e375af258ce8de4ca2b44668d726c929dd2f76f3e8d`.
- SHA256SUMS file SHA-256: `629c00bc3e5a330f18481543b93d37ff5613870e9a832eb9062a4a6f4374c3e3`.
- All 38 kit files passed remote checksum verification before and after rollout.
- Private local kit: `/home/blizz/scratch/gw2-kit-b7fbd8c`.

The walking fix remains included. The kit's mission profile uses the exact
record, equipment, skills and world inputs exercised in sandbox run118:
Norn Mesmer, map379, diagnostic spawn `-3200,-25600,-3742`, profession state
`1,0`, reward tracks `56,88,47,112`, health388 and quest `154,0,1`.
See [health playback](209026-mesmer-health-speed-baseline.md),
[equipment order](209026-initial-equipment-model-order.md) and the retained
run118 REA evidence `ev_6c72a68c21a9c0c650c87a5de3b9bf80ebc062f4bd1512218682a6cc6bf337f0`.
This does not claim retail spawn/state values or implemented healing/combat.

The static musl build passed. A focused local supervisor smoke verified all
four listeners, actual Norn mission arguments, no-capture mode and cleanup.
The guarded rollout checked zero players twice before stopping, switched the
kit only while stopped, and retained automatic rollback. Live health and zero
players were rechecked at 03:11:57 UTC. No additional online client run was
performed or required by the owner for this release.

## Launcher boundary

The owner client channel at
`banshee-owner:banshee-realm-owner/games/gw2/owner/channel.json` remains the
frozen client version209026. It is not the Rust server update pointer. All five
existing runner0e6ac30 objects were read back and matched their size/hash pins.
The existing launcher routes reach the updated server without a new endpoint.

At this release's readback, the launcher self-update channel advertised b124.
The verified launcher lead acknowledged the GW2 dependency at 03:10:50 UTC
under their existing owner publish assignment `f7769cac1b863c25`, to reconcile
the b125 owner-feed receipt. This note does not claim a launcher publication.

Unfinished creator/store code in the main worktree was excluded from this
release using a separate release worktree. Its extra creation endpoint is
also absent from the existing four-route launcher relay. Normal creation
remains disabled; the current runtime blocker is missing preview/navigation
on the appearance screen, recorded in the [creator note](209026-saved-norn-creator.md).
