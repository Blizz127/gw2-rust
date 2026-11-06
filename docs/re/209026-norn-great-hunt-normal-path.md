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

## Run273: Thora and doorway approach observed; trophy/Ido incomplete

The successor controller permits at most three fresh candidate hovers per role
before its single click. Clicking still requires the latest matching hover image
and numeric pointer; interaction still requires the actual target 227 and a
visible matching NPC. Two focused writer cases and two reader cases passed;
the prior unchanged checks were reused. Run272 inputs were not modified.

Run273 used one hover at 645,378 and displayed Veteran Thora Griffonbane. Its
subsequent click produced target 21; F produced Interact 21, Thora's opening and
step 1 in order. The immediate retained post-F image showed Thora's dialogue.
Escape closed it, and root confirmed the fresh world view before moving.
This proves the reconstructed interaction and rendered panel, not authentic
retail dialogue or a native acknowledgement of 542.

Eight separately reviewed W/S/A/D holds produced 196 dense movement reports.
The character followed the visible courtyard and road toward the open hall
entrance, rather than replaying the retained B1–B5 route. Selected endpoints:

| Action | Actual final map XYZ |
| --- | --- |
| W1500 doorway approach | `[-2866,-26091,-3729]` |
| W1000 toward entrance | `[-2675,-26067,-3754]` |
| W750 toward visible floor | `[-2646,-25989,-3752]` |
| D500 across visible threshold | `[-2674,-26065,-3753]` |

The last two inputs nearly returned to the preceding endpoint. They do not
identify a collision cause or establish a route through the hall. No hostile
was selected, no skill was sent, and neither step 2 nor step 3 was observed.
Root released incomplete at the input deadline. The strict reader accepted the
evidence and correctly returned gameplay completion=false; UI/outer status 13
remained incomplete.

Window 63 was returned at `2026-11-06T21:02:55.481825Z`, with the shared container
absent and all 79 immutable inputs, original synthetic record, executable hashes
and privacy checks verified. REA verified its owned group and removed its
temporary root. The 56 other process-environment EACCES observations were not
attributed or altered. The bind capture retained all raw/normalized output;
its truncation flag concerns one rendered-terminal frame of unknown size.

Additional private evidence:

- `209026-run273-norn-thora-trophy-ido-launch-20261106.json`, REA
  `ev_98acf75b6389b966e8039c2696dadb1ef9aef16cb716d81438af05161f5b9846`.
- `209026-run273-controller-thora-f-20261106.json`, REA
  `ev_6ec101d219cb7a8540262b76e977e885e0e6bd7be288ac3bfa0b2b7cdfb56089`.
- `209026-run273-story-readback-20261106.json`, REA
  `ev_04dbda11368b794b5641a4404893c97c05460952f1bfda9b0158b31950cc1bf8`.
- `209026-run273-resource-return-20261106.json`, REA
  `ev_b91984c58893136e376d3dacc1a42644dd4a6e975744f2efb522c1f57f3860f7`.
- `run273-root-story-route-observation-20261106.json` binds all eight movement
  records, images and captures; SHA256
  `62444ab4bbc80a58114a6cb9e30b24b69e9e845792226e394a48fd14957bee9e`.

No production change or new deployment resulted from this diagnostic. Review
the retained doorway evidence before another normal trophy-to-Ido attempt;
the full starting experience and retail accuracy remain incomplete.

Retained-only REA geometry analysis subsequently placed the three final actor
origins within 2.15–3.18 map units of exported category 1 instance 3491/model 373,
source face 1304. Faces 1304/1305 form a low vertical strip only 28.768 map units
high. The final two displacements were nearly parallel to it and returned
within 2.24 XY units of the earlier endpoint. This identifies a specific static
threshold candidate; it does not prove active collision, contact, step height,
capsule behavior or an impassable doorway.

The useful remaining route dependency is visible onward ground around the
hall's upper exterior end. Use the demonstrated B-prefix as a feedback
reference, then continue toward a confirmed hostile and Ido if the current view
permits. Repeating threshold corrections or stopping merely at C1/C2 would not
resolve that dependency.

`run273-retained-threshold-interpretation-20261106.json` has SHA256
`45c4122b04ce9cf38606722f876ac703a570905e79e7b04794aa1aa2c454cdef`.
Its retained-only REA captures are
`ev_806ec2b680ecfad904c4e629a587aee68d42f4672d99f109e12f67b38fa97c9a`
and `ev_ce4f1746db525ba14a92e98ac2409520ec23c3524db1144223444619f2cdcad8`.
No additional runtime or archive access was used.
