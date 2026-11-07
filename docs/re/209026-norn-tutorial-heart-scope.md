# Keep Raven heart progress out of the Norn tutorial

The cb8622a-q171 profile assigns `GW2_HEART=56,5` to tutorial379. That
legacy option installs an ungated heart: trophy and boss kills can advance
heart56 and carry its progress into map28. The map28 profile already has
its separate area-based heart collection.

The correction is an explicit `GW2_HEART=` assignment in the kit profile.
The supervisor sources that profile after host configuration, so the blank
also clears an inherited value. Removing the assignment would not do that.
`GW2_WORLD_HEARTS="map28-hearts.json,5"` remains unchanged.

## Source evidence

Retained client layout data for map379 has zero entries in the type283
heart collection at+0x60. The map28 collection contains15 entries, including
heart56. This establishes scope; it does not recover heart task weights,
authored populations or every possible dynamic region.

- Tutorial collection: REA
  `ev_ed8200ac4a13f8f41a088b885f630666de4a838059cc2f16c6c7b67dc0167c80`,
  type283 in file1282852 at4461360.
- World collection: REA
  `ev_d2f9cf402e44baeab1221d67c40bd6c77a43a42582a0d66c21ef40205b4dd50c`,
  type283 at5919776, heart56 type287 at4224016, resolving type72 in
  file1282835 at6239832.

The full source audit is private at
`~/re/gw2/notes/norn379-heart-scope-candidate-20261107/fidelity-audit.json`,
SHA256 `9f1309554cd61f36d025e97f259506cf61b3fff3525d85a52b67b0636de5efec`.

## Checked candidate and release boundary

The private candidate `mission-profile.conf` in that directory has SHA256
`c29a8dc91c8baf51f12a145720c68ba80989a6ca911b5ffa9023ff621afd771c`.
Root verified it differs from the current kit by that one assignment only.
Offline projection with the pinned supervisor and inherited heart56 removes
only the game arguments `--mission-heart 56,5`. Auth arguments,126 story
arguments, map28 hearts, combat, quest154 and transfer options are identical.
Both saved-mode `--check-config` invocations pass without listeners, using
an owned empty temporary store and ephemeral test identity.

Private `checks.json` SHA256:
`4f0d0d19fc7560b294a84506aa16a89ccd2f7c3de10f206f5482ad90a23e508c`.
The ready candidate was delivered once to the existing sole publisher under
`root_norn379_heart_scope_ready_20261107`; this document alone is not a live
deployment receipt. Preserve the explicit blank in subsequent kits.

This change does not replace reconstructed trophy actors/placements, alter
the kill-to-step2 predicate, repair tutorial respawn coordinates or establish
full retail fidelity. No owner records or live services were changed by
the audit and offline checks.
