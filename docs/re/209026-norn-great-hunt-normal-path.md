# Normal saved Norn Great Hunt path

## 2026-11-06: current-build preparation and run272

The intended observation is normal saved-stage0 Thora → trophy kill → Ido.
Run272 did **not** reach Thora selection, combat or story progression. It ended
with UI/outer status 13 and valid incomplete evidence. This is not a server
regression, native quest pass, or retail-accuracy result.

The baseline was the independently verified live `c2ae0e8-backpack` kit:
game SHA256 `e6fdd1226de8d577b3affa68361415957639523f0a797fd2180de315a9908149`.
Both role startup checks passed with all 85 story arguments. Staging preserved
the original 154-byte synthetic saved record, pristine run145 prefix, current
profile/skills and production runner/GDB. The sole diagnostic argument delta
was `--mission-movement-log-every 1`; no stage, spawn or actor override was used.

A phased controller and reader replace the retired waypoint-only terminal
policy. They require current images, owned focus, append-only logs, confirmed
targets and ordered health/death/story evidence. Focused synthetic checks
covered 21 parser cases, 16 controller cases and 8 filesystem-reader cases.
These checks are not native traversal evidence. Reader checks explicitly cover
delayed Ido dialogue images and a consumed unsuccessful passive movement check.

The first native image was reviewed. Root chose the right-side candidate at
pixel 800,400; the subsequent image displayed **Elite Eir Stegalkin**. No click,
target 227, interaction 23, movement, skill or quest step followed. Root released
incomplete rather than attest the wrong NPC as Thora. The staged controller's
self-imposed single-hover-per-role rule prevented correcting that candidate.
The prior successful run271 candidate at 645,378 is a retained reference only;
future input still needs a fresh image and actual hover label.

Window 62 was returned at `2026-11-06T20:46:24.786614Z`. The shared container was
absent; all 79 immutable non-prefix inputs, the synthetic record, both executable
hashes and credential-safe console check passed. REA verified its owned process
group and removed its temporary root. Its 34 other process-environment reads
returned EACCES; those processes were not attributed or altered.

Private evidence under `/home/blizz/re/gw2/notes/`:

- `run272-norn-thora-trophy-ido-ready-20261106.json`, SHA256
  `4ac6ed2b6eb5e1cf347ae9ea65817833cce98c697718a6498b424982972fac2b`.
- `209026-run272-norn-thora-trophy-ido-launch-20261106.json`, REA
  `ev_540c2ddd36d17307080c17d500dc794bfd5840f8e6b7c6420030c832ffddc75d`.
- `209026-run272-controller-thora-hover-20261106.json`, REA
  `ev_258b7786999c02a8d6a740dca5022362291cdda6e9531385800d01f6970c3130`.
- `209026-run272-story-readback-20261106.json`, REA
  `ev_769f37ebab9ae82c7a1c2a60f8e92b62112a82c32bbc65d006311cc8831da0cc`.
- `209026-run272-resource-return-20261106.json`, REA
  `ev_6a0e0c7ed46ded981404fcdaeeef76928ac0ba9359d1f07bb654ea67b8f98551`.
- `run272-root-hover-observation-20261106.json`, SHA256
  `a5871f5ad4202a62a63ab7c9d0fb475b7bcd52b12d6ea30c4ace4ad4a60f1f22`.

The next controller correction permits bounded fresh candidate hovers before
the single role selection; it does not relax target identity or interaction
checks. No production change or new deployment resulted from run272.
The continuous ground beyond the retained B5 view and the actual trophy/Ido
path remain unverified. The configured non-boss trophy predicate and actor
placements remain reconstructed; see the Great Hunt species-link audit for
the unresolved authored cohort.
