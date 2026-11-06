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

### Current-build startup and binder correction

Run274 used the independently verified `633fff8-q170` kit and its exact 102
story arguments. The current-kit auth/world checks and staged checks passed
without listeners. Normal quest 154 handler files and fixture pins match the
previous `c2ae0e8` baseline; this source comparison is not native equivalence.
The only gameplay diagnostic argument remained
`--mission-movement-log-every 1`. The controller input/release/hard bounds were
600/630/660 seconds, within UI 1020, wrapper 1065 and total 1125 seconds.

Its binder failed the combined container name/network/Running assertion before
creating GO or issuing controller input. A subsequent REA observation found the
exact owned run mount, network-none and running container, with GO still absent.
The initial failed inspection was not retained, so a `created` startup state
is an inference, not a directly recorded observation. Both UI and outer capture
exited 11; no Thora selection, quest progress or combat was observed.

Window64 was returned at **2026-11-06T21:28:23.318172Z**. Return verification
confirmed the container absent, all 79 immutable nonprefix inputs unchanged,
the original 154-byte synthetic record, executable hashes and console privacy.
The full capture was untruncated and verified its owned process-group cleanup;
24 unverified process observations were retained without attribution or action.

The fresh run275 binder reproduces the suspected failure in a synthetic
`created → running` sequence: the old binder fails on the first inspection;
the corrected binder admits exactly one GO after the second. It validates the
container name, network, mount, run marker and stable container ID before
treating `created` as pending. Terminal, unknown, paused, restarting and
mismatched ownership states still fail. Three focused tests passed, including
the bounded created-state timeout. The 65-second container wait, 365-second
total preclaim cap and exclusive GO creation remain intact. Run274 was neither
modified nor retried.

Private evidence:

- `run274-current-633fff8-preflight-20261106.json`, SHA256
  `270db86a7a59d4d3ebc240b941e103fc02f03c9564e7ca535418b39ed3ec2750`.
- `run274-root-startup-findings-20261106.json`, SHA256
  `757c2f622afd5631da5a2245c94e2fd9e84b7e9f4a6c5f1fbb061f3d14f3dd4f`.
- Binder failure: `ev_05bfd55eef58192e4ea010231ce89e68d7f97f0922d08f6c6d683d85404c43c0`.
- Later running-state observation: `ev_2214338e601877c43a99d415782ffd89583af83ebbf4ae842dd7fcffcf29d967`.
- Full launch: `ev_e17193168eb8fe0c30cf45dfd505ba0b88e9f5f3ce97710221c2747700290c9d`.
- Readback: `ev_8ac1e50373163527a76f61bc075dbf0ed7ef9fe3fe5ea044752f85a0edfee6ec`.
- Resource return: `ev_a1766dff71b2fff98f2c494a074a559b59c04ea2a0b7bc8ae3979de991d732b1`.
- `run275-binder-lifecycle-receipt-20261106.json`, SHA256
  `b408ff221a4da2b4a3c94966e0ebf7a4110bcc1c4c99af958626417d059aa121`.

This correction changes private diagnostic startup only. It supplies no new
normal gameplay or retail-fidelity evidence and is not a production release.

### Run275: native traversal beyond the previous exterior endpoint

The corrected binder successfully bound the fresh run275 container, claimed
the controller and captured its first world image at **21:33:38.779574Z**.
The exact current `633fff8` normal stage 0 configuration, original synthetic
record and production runner were retained. Thora's visible identity, actual
target 21, interaction, step 1, rendered dialogue and Escape dismissal were
observed again before movement.

Seventeen individually reviewed W/A/D holds produced 263 dense movement
reports. They followed the exterior table/grass strip rather than the shop
doorway, then continued beyond the previous B5/C2 neighborhood:

| Observation | Actual final XYZ |
| --- | --- |
| Exterior grass strip | `[-2518,-25139,-3747]` |
| Upper corner | `[-2072,-25139,-3765]` |
| Short right step | `[-2072,-25204,-3761]` |
| Near C2 reference, about 16.5 XY units away | `[-2013,-25204,-3763]` |
| Onward edge | `[-1969,-25265,-3757]` |
| Last clearly readable view reference | `[-1922,-25265,-3756]` |
| Final gap position | `[-1882,-25345,-3756]` |

The requested holds total 7900ms; that is neither measured native input
processing time nor a movement-speed measurement. Each successful hold has
fresh displacement and image evidence. This is new traversed ground, not the
near-return doorway loop observed in run273, but it does not prove passage to
the configured hostile area.

The visible ground strip narrowed between rock and hall. The camera view then
became filled by the avatar and wall; a passive `rear-view` image remained
obstructed. No hostile selection, skill hit, death, step 2 or Ido/step 3 was
observed. A proposed A250 retreat was recorded 53.703 seconds after its scene,
beyond the 45-second freshness limit. Its scene/admission/focus fields are null,
and both native-input flags are false. Source order and retained timing support
stale-scene refusal; no exception string was captured. The controller released
incomplete without retrying that input.

UI and outer capture exited 13. The strict readback is valid and incomplete,
with 23 action records, including the refused retreat. Window65 returned at
**2026-11-06T21:42:14.382260Z**, after verifying the container absent, 79 immutable
nonprefix inputs, original 154-byte record, executable hashes and console
privacy. The 56 retained images were not all visually reviewed by the cleanup
helper. Full launch cleanup verified its owned process group; 62 unverified
process observations were retained without attribution or intervention.

Private evidence:

- `run275-root-story-route-observation-20261106.json`, SHA256
  `eacd91f64f0bdaf92064eb353e18814d882f3ec9a6f934cf8d3da7c86694e873`,
  binds all 17 movement records, images and complete captures.
- First view/bind: `ev_0ad0667d1005e83cd17e28f9edd02c09f58ff591d643120afce4ee057f1e3f44`.
- Thora dialogue: `ev_30430a515428d6d496362e22a6b0a41d5e50a3e90730535a65131883ccb20042`.
- Passive obstructed view: `ev_a1566198096337403ed8404685b9c86a47f4c53a0706abaf0b5d54a0125c42f6`.
- Refused retreat: `ev_71fa35cbb5597d6966017fee015baf4f224dec64452275f901343e43fcc39d8f`.
- Full launch: `ev_053f8206eb006973ceaad3474d9b3a8e28e4faa8d118c1bdf5ad13abeebad738`.
- Readback: `ev_5af2120dc26393edc2cae838ab1c2dad6ac4873a56a8497380a57582f4bfbbe3`.
- Resource return: `ev_c8675e85dabd9112d5104f9ed5d961417b10932fc6ed8ad1d8999dbc1089b44b`.
- `run275-gap-retreat-and-view-scope-findings-20261106.json`, SHA256
  `13f2ee25205baab7b3bdf0502124612ab00ee77b5403ef5a977cbaae1bc76843`.

Bind and readback each omitted one rendered-terminal frame of unknown size;
their raw bytes/frames and normalized frames are complete. Individual input
records and images remain separately pinned. Neither truncation flag is
silently treated as full rendered-terminal coverage.

The next dependency is an unobstructed onward view before further translation.
A separate bounded native view/look operation needs its own controller and
reader evidence in a fresh resource scope; closed window 65 cannot be reused.
Retain the freshness limit and refresh the image before a delayed action.
No new production change or deployment resulted from run275, and normal
trophy/Ido progression and the full retail-accuracy goal remain incomplete.
