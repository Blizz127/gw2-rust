# Regeneration scheduling and quest154 visibility

## Verified regeneration release

The sole Claude publisher deployed `3b68f65-regen` at
**2026-11-07 00:40:24 UTC**, kit `/home/blizz/gw2-rust-3b68f65-regen`,
with rollback `/home/blizz/gw2-rust-d01fbef-world` retained. It integrates
creator change `542d45a0` onto the newer world/inventory release. The older
creator binary was not deployed over that newer code.

The scheduler now includes eligible regeneration deadlines when choosing
its next wakeup. Previously an idle connection could wait for unrelated
traffic despite a pending regeneration tick. Stale or full-health deadlines
are cleared. Existing eligibility, delay, interval and amount are preserved;
this is not a new claim about retail regeneration timing.

Four focused tests passed, including an actual regression failure before the
fix. The creator artifact passed musl build and offline auth/world startup
checks. The publisher integrated and built the final current release.

Independent readback at **00:45:51.475326 UTC** verified all53 current and
rollback manifest entries, the shared auth/game executable, four healthy
owned listeners, zero humans/bots, saved-character mode, the unchanged
profile/supervisor, all126 story arguments and CDN responses. Only BUILD
and the executable differ from rollback. Private character-record contents
were not read; record/store modes and the existing backup were preserved.

- Deployed executable SHA256:
  `7f13309e2d9ae9210726e3ee9d912ed8aa0b72dca5a75d896f8044e243bbe90c`.
- Manifest SHA256:
  `9b62702dc84cc17d4bd53dea9fbba3d664586a70b90708cd1c59e2c66ed25e7f`.
- Private receipt `3b68f65-regen-release-verification-20261107.json`, SHA256
  `b21a0ce0331c583c52261bb79824bf8822b24b0e840006b8f6aecf982b013283`.
- Full readback `209026-3b68f65-regen-release-readback-20261107.json`, SHA256
  `2b18d0c3ef9ab82d51cad922fd457411ae6e5d83cf466919c1cb7cd08815809d`,
  Evidence `ev_99bfe4c118b4f17bb35c5ea8c57bf0e8da91b2f18905ec76f95a92b68f2e7f1a`.

The readback exited0, untruncated, with owned capture cleanup verified. This
confirms deployment and configuration; it is not a native regeneration
playback result. Optional new chapter2 and vendor-stock configuration remains
disabled. The owner-only launcher publication hold is unchanged.

## Quest154 visibility candidate

Creator commit `f9a48c0daef337b339cdd7d9233c47c12011405a` is based on the
regeneration release. It adds the recovered547(step2) notification after
the existing accepted Thora542(step1) transition and fresh same-map stage1
restoration. Nine focused tests pass, including wire/schema/truncated-frame
checks and genuine missing547 regression failures before the correction.

The first attempted regression run failed on a test-fixture profession
mismatch; it was not evidence of the production bug. That setup failure and
an intermediate test-reader limitation remain separately recorded in
`quest154-step2-visibility-20261107-checks/receipt.json`, SHA256
`69f80ae671a2b30dc4a61c9e2e86b5e43a70ff886a0daa15dda09295b1a26de7`.
The final regression checks used the corrected tests against the exact parent
implementation, then passed after restoring the committed fix.

Candidate musl executable: 2,092,480 bytes, SHA256
`c92c98a5bb9a03eacc3a99fc35347ee67910ca3a164fb0e73c03d601eded8ed4`.
This candidate section does not itself establish publication. The
[visibility findings](209026-quest154-step-visibility.md) distinguish the
client mechanism from reconstructed server triggers. Missing localization,
authored trophy actor/credit rules and later objective visibility remain
unresolved; full starter fidelity is incomplete.
