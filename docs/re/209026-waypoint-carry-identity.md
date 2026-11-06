# Waypoint progress belongs to the selected character and map

The observer kept one pending waypoint handoff for the entire process.
After character A requested a waypoint redirect, any subsequent world join
could consume A's spawn, XP/earned level, loot count and heart state. Leaving
the redirect and selecting character B could therefore transfer A's progress
to B and remove it from A's pending rejoin.

The population fixture now retains the character GUID already present in its
validated auth record. Saved-character selection obtains that record using
the selected mission GUID, as documented in the
[existing binding trace](209026-creation-mission-binding.md). This fix uses
that established identity; it introduces no packet or schema interpretation.

A pending handoff stores the character GUID and destination map key alongside
its progress. MapLoaded applies and consumes it only when both match the
joining population. A different character, different map, or connection
without population leaves it pending. Matching rejoin still consumes it
once. The GUID is omitted from the handoff's Debug output, so the existing
progress log does not acquire a character identifier.

This remains a single pending handoff in a research observer process. A newer
waypoint request replaces that pending handoff; this change does not provide
a multi-player transfer store or durable progression. Process restart still
loses XP, loot and heart state. Character creation records, roster mode,
launcher channels and the native publisher prompt are unchanged.

The change is an ordinary server-source correction. No new shipped-client
execution, archive extraction or REA observation is claimed.

## Focused verification

The new isolated lifecycle test creates two saved characters in a temporary
store and resolves each through its selected mission GUID. It verifies that
B's join, A's join on another map, and a connection without population do
not consume A's pending handoff. It then checks A's exact population/spawn
and XP/loot/heart restoration, duplicate MapLoaded, a subsequent fresh join,
and the identifier-free Debug representation.

All four waypoint-filter tests, 42 mission-filter tests and seven
saved-population tests passed in a separate Cargo target directory. The
existing heart/earned-attribute lifecycle tests retain same-character
rejoin and reward-once coverage. No failed-before run is claimed.
Private receipt: `~/re/gw2/notes/waypoint-carry-identity-tests-20261106.json`,
SHA-256 `f1935d8b5d054a3eebb649db8716817129e6f98849cd2d434df9363fb7dc442a`.

## Checked cumulative build; not deployed

Source commit **5516cb419639b8a88c8b9e67f0cf66267110888c** includes the earlier
earned-attribute curve and skill-input corrections. The isolated offline
musl release build succeeded:

- Binary: `/home/blizz/scratch/gw2-carry-identity-20261106-target/x86_64-unknown-linux-musl/release/gw2-game-observe`.
- Size: **1582864 bytes**; static PIE, BuildID `131a86a7a2cdeabf8baabb59bf37876c55698516`.
- SHA-256: **eafed1d81abd58613c44df7783cfd53d475c6ac57ef0397118f9f06a790027dd**.
- Source worktree: `/home/blizz/scratch/gw2-carry-identity-20261106-src`.
- Paired supervisor: that worktree's `deploy/gw2-server.sh`, unchanged SHA-256
  **c006b89524e011039a9b5bdb4df913d2505e82a9f44f7e69a31cd02b8735b6ad**.

At **2026-11-06T03:12:49.772264Z**, the exact binary passed auth, prepared
curve, prepared fixed37, and saved-curve configuration checks without starting
a listener. The saved case used an empty private temporary store. The
existing explicit damage/echo inputs remained diagnostic. Retired-flag
rejection, two legacy-setting compatibility cases, shell syntax, eight
fixture hashes, four schema input pins and the held run145 manifest passed.

The candidate profile remains
`~/re/gw2/notes/mission-profile-attribute-curve-20261106.conf`, SHA-256
**d7e25b1da72a9354f7cf84c7d46ad893158fac3c24a1cda13a68d73a6a9ee3ef**.
Relative to the pinned live baseline it only adds
`GW2_BASE_ATTRIBUTES=default-curve` and removes retired
`GW2_AUTOATTACK_SKILL=10170`; preserve other publisher changes and roster
selection. The identity fix itself adds no setting.

Private build/input receipts are `carry-identity-build-checks-20261106.json`
and `carry-identity-build-inputs-20261106.json` under `~/re/gw2/notes/`.
The final test receipt above includes source identity and result summaries;
the earlier transient receipt hash in the build commit's note is superseded
by the final hash recorded here. No test or production source changed in
that receipt finalization.

The existing publisher remains at the unanswered native saved-character
versus prepared-character choice. This artifact is ready for that publisher;
it is **not live**. Prior tested artifacts and live rollback remain intact.
Last verified live stays **82d8d04-cdn900**,
`/home/blizz/gw2-rust-82d8d04-cdn900`, deployed **2026-11-05T23:56:31Z**.
No deployment, runtime lease, run145 launch or publisher restart occurred.
