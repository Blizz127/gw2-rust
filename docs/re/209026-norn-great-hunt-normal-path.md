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

### Run276: bounded turn support, stopped before the first turn

The new controller admits Q/E for 50–500ms, at most eight times, after Thora
and explicitly closed dialogue. It retains the existing ownership, freshness
and deadline checks, releases keys in `finally` and exit cleanup, rejects any
sampled XYZ change or unintended target/interaction/combat/story event during
a turn, and requires a short measured movement before longer holds afterward.
No movement report is required for a turn; missing reports establish neither
stationarity nor yaw. Six focused controller groups, seven parser groups
(11 scenarios), and five actual-writer reader cases passed. These are mock
checks, separate from native runtime evidence.

Run276 retained the current `633fff8` configuration. Thora, step 1, dialogue
and dismissal were observed. Eight successful holds produced 177 movement
reports, ending at `[-2166,-25151,-3760]`. The next W200 request and its one
passive observation produced no new report. The controller recorded
`missing_translation_sample` and released incomplete, before executing Q/E.
It still had 432 input seconds at the hold and 423 at the passive check, so
this was not deadline exhaustion. Collision versus lost input remains unknown.

UI and outer capture exited 13; strict readback is valid and incomplete.
Window66 returned at **2026-11-06T22:09:11.072855Z**, with the container absent,
79 immutable inputs, original synthetic record, executable hashes and privacy
verified. Full capture, readback and return are untruncated; the first-view
binder retains its rendered-terminal truncation flag. The full capture verified
owned cleanup and retained 37 unverified process observations without acting
on them. No hostile, hit, step 2, Ido/step 3 or native turn was observed.

Private evidence:

- `run276-native-controller-turn-checks-20261106.json`, SHA256
  `037b08f49959bb974b0d044bb6fc74e0335fe7bcdff6c21a37b19c89e5017024`.
- `run276-story-readers-checks-20261106.json`, SHA256
  `61fe02da5ea8c00adf8884b4789e16f8a8cb6643376aa7995969a4ad2990b8c0`.
- `run276-root-story-route-observation-20261106.json`, SHA256
  `f1429947c5e214b3f20ca565ef49629ab9b8d44d53864327569b0804d0feeb2e`.
- `run276-terminal-retained-audit-20261106.json`, SHA256
  `c1e6bdb864f06b53e8d3008591d0790f908cf4e07b05d9bba15838dd90a060f2`.
- Full launch: `ev_1dc4af6253efea848838a8bbd15a90b347d5aaa1f26176866bba997a5e80e81f`.
- Readback: `ev_a0f50f404fbd21480dbe828e39b08ba662058fd9dd70bd051d33d53c80b44bf3`.
- Return: `ev_71c4d27e652414f89d91b6bc57afe5b072228bf9545945a196aed59f68a984e6`.

The next attempt should inspect the view earlier, from the starting apron
after Thora's dialogue closes, then select visible ground toward a hostile
and return to Ido. Run236's retained E500 view exposes the right apron;
it does not prove onward passage or measured heading. The guards and full
normal-path objective remain unchanged.

### Run277: native turns and the right apron route

Run277 changes helper identities and reader pins only; numeric constants and
behavior match the tested run276 helpers. Its two staged no-listener checks
passed. The fresh grant explicitly allowed the same bounded Q/E actions,
starting with E250 after Thora's dialogue closed. The first two images lacked
some hall geometry; the second candidate hover showed the hall and Thora's
name before the single selection click. Target 21, interaction, step 1,
dialogue and dismissal were then observed normally.

Seven turn inputs were admitted. The first E250 did not visibly shift the
background; the following E500 moved the doorway left and exposed the right
apron. Later Q/E actions also changed the visible landmarks. All seven turn
intervals had zero new position reports and zero typed parser events, so this
does not exercise same-position rotation reports or establish yaw, continuous
stationarity or camera-only motion. Five subsequent 250ms movement checks
cleared calibration with measured XY changes; the first covered two consecutive
turns. The final turn remained uncalibrated because root released incomplete
without further movement.

Sixteen measured holds produced 260 reports and traversed the right apron,
covered carts and a different hall corner:

| Observation | Final reported XYZ |
| --- | --- |
| First post-turn calibration | `[-3163,-25660,-3743]` |
| Right apron onward | `[-2908,-26082,-3719]` |
| Cart clearance with readable third-person view | `[-3083,-26811,-3722]` |
| Far corner with readable near footing | `[-2906,-27040,-3738]` |
| Foliage obstructing footing | `[-2796,-27081,-3745]` |
| Final measured sidestep | `[-2743,-27074,-3747]` |

The requested holds total 7250ms and turns 2600ms; neither is native processing
time or a speed/angle measurement. The final view exposed forest downslope
through foliage, without establishing a connected route toward a hostile.
Root released incomplete with 78 input seconds remaining. No guard refused
an action, no movement remained pending, and neither deadline nor turn limit
had expired. There was no hostile selection, attack, death, step 2 or Ido/step 3.

Strict readback validates all 29 actions and remains incomplete; UI and outer
capture exited 13. Window67 returned at **2026-11-06T22:27:29.705028Z**, after
container absence, 79 immutable inputs, the original record, executable hashes
and console privacy were verified. Full capture verified owned cleanup and
retained 74 unverified process observations without attribution or intervention.
Root reviewed controller images, not all 67 retained images. Bind omitted one
rendered frame and readback four, of unknown sizes; their raw and normalized
terminal data remain complete. Full launch and resource return are untruncated.

Private evidence:

- `run277-root-story-route-observation-20261106.json`, SHA256
  `717fb09f410e4c14b5bef2b7d13f700985ed73d64659c6d6b6e8ebcf6f10f85c`.
- `run277-retained-turn-action-audit-20261106.json`, SHA256
  `f4ff8bee1e20e87ee7f55e305a15e08170ea8dbed4adf1f4c0bb8cfcd663f530`.
- `run277-retained-far-corner-findings-20261106.json`, SHA256
  `5241be21defed4628abf5bed1df87b9069dc83ccb94faf233cab8b82690e5a54`.
- First shifted view: `ev_e5888b594a0426a5891c6cae27d167066d31b2cc13e92a1b74338ff020970697`.
- Full launch: `ev_f601d05292aad3cbeccfb77271a67984323397d97ad72f88ef41f76e9b5eefce`.
- Readback: `ev_b43415bf9351c3a37110a9466ffb269ccbe5caa5b8e2ff342f3c4775c6be82cf`.
- Return: `ev_04614fefdc1aedf922f6ba738f32344ae958e97f145e07ccec161a1ccceb153c`.

The right-side route is now observed up to the far corner. Onward passage,
normal trophy/Ido completion and full retail accuracy remain unproved. A clear
view of the far-side ground is needed before entering the obscured section;
the final forest view does not authorize an inferred descent. Repeating the
same obscured movement/turn sequence would add little evidence. No production
change or deployment resulted from these diagnostics.

## Run278: Camera bindings and lodge threshold (2026-11-06)

The native Control Options list now supplies a concrete camera-input source.
One pre-Thora inspection opened F11, selected the mouse-shaped Controls category,
and scrolled down six times at three ticks each. The Camera section appears
below User Interface and above Screenshot/Map. Its displayed assignments are:

| Camera action | Primary binding | Secondary binding |
| --- | --- | --- |
| Free Camera | blank | blank |
| Zoom In | Page Up | blank |
| Zoom Out | Page Down | blank |
| Look Behind | blank | blank |
| Toggle Action Camera | blank | blank |
| Disable Action Camera | blank | blank |

These are displayed bindings, not observations of zoom behavior or distance.
No camera action was executed. Menu input targeted only opening, category
navigation, list scrolling and Escape; it targeted no binding or setting edit.
The retained images do not prove persisted preference bytes unchanged. A fresh
world image and explicit world-return attestation preceded story input.

The new controller required this single inspection before Thora and preserved
the original story-log baseline throughout it. Its strict comparator rejected
observed XYZ changes and new target, interaction, combat or story events, while
allowing benign regeneration/respawn and same-position/no position reports.
Seven controller and six reader test groups used the actual controller writer
with mocked native operations. Three permit-scope checks and two staged
no-listener configuration checks also passed. All previous input deadlines,
focus ownership, image freshness and cleanup guards remained in force.

The approved sandbox used `633fff8` with 102 story arguments. The publisher
deployed `cb8622a-q171` during preparation; its separate independent readback is
recorded in the backpack document. This run is not a native test of that build.

After the inspection, normal Thora selection and interaction produced step 1.
Escape and a fresh dialogue-closed confirmation preceded one E250 turn and six
measured movement holds. The initial image already contained the hall geometry.
The route approached the broad lodge doorway, producing 111 position reports:

| Hold | Final reported XYZ |
| --- | --- |
| W250 calibration | `[-3105,-25646,-3744]` |
| D250 alignment | `[-3159,-25757,-3738]` |
| W750 approach | `[-2906,-25883,-3729]` |
| W750 threshold approach | `[-2738,-25965,-3745]` |
| W500 toward interior | `[-2657,-26018,-3752]` |
| W250 threshold step | `[-2659,-26024,-3752]` |

The final hold added seven reports but only `[-2,-6,0]` net displacement. Paired
images retain the character at the threshold with the lit interior floor and
oven visible. They establish neither crossing nor an onward exit, and do not
diagnose a collision. Root released incomplete with 282 input seconds remaining;
no guard refused an action. There was no hostile selection, attack, death,
step 2 or Ido/step 3. Requested hold durations are not native processing times.

Strict readback validates all 22 actions, including ten menu actions, and remains
incomplete. UI and outer capture exited 13. Window68 returned at
**2026-11-06T23:00:29.441507Z** after container absence, 79 immutable inputs, the
original 154-byte record, executable hashes and console privacy were verified.
Owned cleanup passed; 60 unverified process observations were left untouched.
Root reviewed the first image and all 22 post-action images, not all 50 retained
images. Bind and readback each omitted one rendered frame of unknown size;
their raw and normalized output remain complete. Launch and return are
untruncated.

Private evidence under `/home/blizz/re/gw2/notes`:

- `run278-root-story-controls-observation-20261106.json`, SHA256
  `eb65cb0c440a97b4f0a431ee797ca709e65bb20c6448eefbb8b8311399c76d4e`.
- `run278-camera-controls-threshold-guide-20261106.json`, SHA256
  `2eeeebe860ced94205adb97868531edaf2b415857aef7b9f6d5b4ac30c446e7f`.
- Camera rows image SHA256
  `cfc47cc5f9ee4334fc3a6fbb1c188eabbf5b194b408d39d04a887776089ff014`.
- Full launch: `ev_a1cbaae14354afbaa4bdb6cea4e37e212750bb6a3b01721b93243495257f84f7`.
- Readback: `ev_a1de93ad4b61638327d18e00019489fd377a0fde8c181e28bb85a72e3b5ccd63`.
- Return: `ev_92c3aaec25cd274ec6b860e6cc1f47044fd6ad0386f0dd0504a42d10d2732d0f`.

The next bounded observation can use the displayed Page Down binding to assess
zoom response on a fresh run. Full trophy/Ido completion and retail accuracy
remain unproved; this diagnostic produced no production change or deployment.

## Run279: Page Down dispatch and partial apron route (2026-11-06)

Run279 used current `cb8622a-q171` and its 126 story arguments, with only
movement-log cadence changed for observation. The original 154-byte synthetic
character, pristine run145 prefix and production runner were preserved. Two
current-baseline and two staged configuration checks passed. Seven focused
controller groups, five actual-writer reader groups and three permit cases
covered the new bounded zoom action. Page Down uses the local X keysym `Next`;
both names have value `0xff56` in the local headers. Run278's displayed binding
is the client-side source for selecting that key.

Normal Thora selection, interaction and dialogue dismissal produced step 1.
Two zoom-out holds requested 250ms and 500ms. Both commands completed, but the
three paired images show no clear change in framing or scale. Their intervals
added no position report or typed parser event. This does not distinguish an
unconsumed input, a zoom limit or a transient response between images, and it
does not establish stationarity. Zoom counters advanced from zero to one to
two, and subsequent measured movement cleared the required calibration.

Four Q/E turns and seven successful movement holds traversed the apron to the
outer carts, producing 135 consecutive position reports. The last successful
W500 ended at `[-3083,-26855,-3722]`. After Q500, a D250 calibration added no
report; the single permitted passive check also added none. The controller
then stopped with `missing_translation_sample` and calibration still required.
Root released incomplete with 229 input seconds remaining. The view showed
near grass, carts and hall to the left, and foliage ahead-right; an onward
connection was not established. No hostile selection, attack, death, step 2
or Ido/step 3 occurred.

The 19 actions comprise five story actions, two zooms, four turns and eight
movement holds, including the final unresolved hold. Strict readback is valid
and incomplete; UI and outer capture exited 13. Window69 returned at
**2026-11-06T23:20:10.083166Z**, after container absence, 79 immutable inputs,
the original record, executable hashes and console privacy were verified.
Owned cleanup passed; 69 unverified process observations were left untouched.
Root reviewed the first image, all 19 post-action images and the final passive
image, not all 51 retained images. Bind/readback retain rendered-frame omissions;
all raw and normalized data frames remain present, including the complete
readback JSON. Launch and resource return are untruncated.

Retained command timings do not establish a controller sequencing defect.
The selected 250ms requests took 461–497ms including command and final key
release overhead; the selected 500ms requests took 748–749ms. The controller's
`native_input_sent` means command completion, not client consumption. Other
250ms movement intervals succeeded. These facts establish neither a generally
insufficient hold duration, low-frame-rate causation nor collision.

Private evidence under `/home/blizz/re/gw2/notes`:

- `run279-root-zoom-route-observation-20261106.json`, SHA256
  `820da787e7abae4bf8214967b2bdabc40b289a470b503f7a2463bcd2845c2bc1`.
- `run279-retained-input-terminal-audit-20261106.json`, SHA256
  `732b69690e1ab7eaec832331de90d435eee2d62266275010832dfe3001185070`.
- Three zoom images: `ev_13f6fc2613da6c3067a8e02e4fc3dff430c1ff7cf3b64c4cf3b142c8aba458e7`.
- Full launch: `ev_829f9936abbbdd1c1602aa3162b1ccd8869b05c8ac382d0550d33842d1d328ce`.
- Readback: `ev_1b5345299bb90794052a2d5626191aa2cf53c0cfce44fe34bb210650c737a964`.
- Return: `ev_29854da719c7645b1209bd676de917912c54b5c359f6df394c9b20f6e85f21e9`.

The next diagnostic dependency is sampled native input evidence. Run268's
guarded three-site observer previously captured baseline/input/post movement
vectors. Reusing it around one short calibration can test whether the sampled
generic movement path changes while a key is requested. It cannot establish
controlled-player identity, exact key-consumption time or collision. A new
scope must preserve the existing no-report terminal and cleanup guards; no
new observer was installed in run279. Full normal trophy/Ido completion and
retail accuracy remain unproved. This diagnostic made no production change.

## Run280: native vector sample and camp path (2026-11-06)

The run-local controller now offers one explicit `vector-hold` action: a
root-selected W/A/S/D hold of exactly 250ms. It reuses run268's unchanged
three-site observer and independent prefix validator. A complete baseline
sample must precede input; the worker rechecks the existing scene/input cutoff
after waiting for readiness. Complete host dispatch, baseline/input/post
coverage and disabled-hook cleanup are required before further input. Ordinary
movement comparisons, measured calibration and the one-passive/no-report stop
remain unchanged. There is no retry or extended hold on failure.

Seven controller test groups, six actual-writer reader groups covering 18
scenarios, eight composition checks, four permit cases and two staged
configuration checks passed. The source observer retains its 600-second arm,
10-second window, one-second pair, 64-callback and nine-sample bounds. Its
diagnostic GDB composition belongs only to this run; the exact inverse matches
the pinned production GDB source. This is not a byte-identical production GDB
run. The current `cb8622a-q171` server, 126 story arguments and original
synthetic character were preserved.

Normal Thora interaction produced step 1. After dialogue dismissal, two E500
turns exposed a different clear camp path. The single instrumented W250
produced 12 movement reports and three matched native sample triples:

| Host phase | Input vector | Computed/composed vector |
| --- | --- | --- |
| Baseline | `[0,0,0]` | `[0,0,0]` |
| Input | `[0,1,0]` | `[-0.841389,-0.540430,0]` |
| Post | `[0,0,0]` | `[0,0,0]` |

Source, thread/context and paired-frame guards passed. Nine callbacks spanned
an observer window from elapsed 268970ms to 270078ms. The terminal reason was
`marker_removed`, all hooks were disabled, the active marker was absent and
final readback found no later observer events. These are sampled generic
movement-context values, not controlled-player identity or exact native key
timestamps. Debugger callback spans do not establish frame rate or normal
unobserved execution timing.

The route then followed visible ground outside the camp and turned back toward
grass and tables. Six successful holds produced 125 contiguous reports:

| Hold | Final reported XYZ | New reports |
| --- | --- | --- |
| Instrumented W250 | `[-3299,-25664,-3741]` | 12 |
| W750 | `[-3476,-25779,-3720]` | 22 |
| W750 | `[-3662,-25903,-3683]` | 23 |
| W1000 | `[-3934,-26078,-3615]` | 30 |
| W250 after Q500 | `[-3928,-26159,-3606]` | 9 |
| W1000 | `[-3901,-26471,-3647]` | 29 |

At the final table, an A250 sidestep and the single passive check added no
position report or typed parser event. Their server-log prefixes were identical
at 14542 bytes/276 lines. The controller stopped with
`missing_translation_sample`; root released incomplete with 343 input seconds
remaining. The earlier observer was already stopped, so this run contains no
native-vector sample of that A input. Its host command completed in about
565ms, which does not establish client consumption or explain the silence.

The 15 actions comprised five story actions, three turns and seven movement
holds, including the final unresolved hold. No zoom, hostile selection, attack,
death, step 2 or Ido/step 3 occurred. The clearest path image shows a dirt strip
bending between grassy banks toward a brazier; passage beyond the bend and a
connection to configured trophy targets remain unproved. Final images show
a table ahead, a stool to the right and visible near-side grass, without
establishing collision or passage past the table.

Strict readback is valid; vector coverage is accepted while story completion
remains false. UI and outer capture exited 13. Window70 returned at
**2026-11-06T23:51:07.888575Z**, with container absence, 83 immutable inputs,
the original 154-byte record, executable hashes and privacy verified. Owned
cleanup passed; 53 unverified process observations were left untouched. Root
reviewed the first image, all 15 post-action images and the final passive image,
not all 41 retained images. Readback retains all 22128 raw/normalized bytes and
six data frames, but only four of six rendered frames, so its overall
truncation flag remains true. Bind also retains a rendered-coverage flag;
launch and resource return are untruncated.

Private evidence under `/home/blizz/re/gw2/notes`:

- `run280-root-vector-route-observation-20261106.json`, SHA256
  `5b8a2ac5abbec4919b5bb0a1b24e4594a9a22a9a4ee34e0f048bfff76975d620`.
- `run280-retained-input-terminal-audit-20261106.json`, SHA256
  `a27da618ee7f101b1ba288c94fd35be1c79eed9562e6d4be7eaf8d857ac51079`.
- `run280-retained-vector-lateral-audit-20261106.json`, SHA256
  `50de6de498a33b5cedf0ac21f0b0cd30d2e13fe0f357d65d575721c4aa65d932`.
- `run280-retained-camp-path-guide-20261106.json`, SHA256
  `f7f8f0a6abc7c0d9d0f9f5f4e178bd27eb323861d139124663b1c01dc4c0162b`.
- Instrumented W: `ev_768153e7d1cabee63317cad81f86393b9581ec85557e94e0247b29dfa813288e`.
- Readback: `ev_04d4980552162c37cc7cd0d8abfc0476c4374e6078b64dc028ff7819db991f08`.
- Return: `ev_92bd51f96a1c7d2f49606834cc5df757ec7104c577316d3c1296adf393355e18`.

The next discriminating observation should reserve the single instrumented
250ms pulse for a root-selected lateral key on freshly reviewed ground, before
the unchanged observer deadline. Successful W sampling does not diagnose the
later unsampled A input. No longer-hold fix, collision cause, full normal hunt
or retail accuracy is established. This diagnostic produced no live release.

### Retained lateral-binding check, 2026-11-07

The retained Controls evidence identifies A/D as Strafe Left/Right and Q/E
as Turn Left/Right. Run278 shows the lateral rows and E; the unobscured Q
row is retained from235. Both279/280 helpers dispatch the requested lower-case
key directly through keydown, bounded sleep and keyup, with cleanup keyup.
There is no supported key-remapping correction.

Run279's earlier A250 already produced seven reports, moving from
`[-3123,-26071,-3689]` to `[-3087,-26071,-3691]`; run275 D250 also produced
eleven reports. Those observations contradict a general unbound-lateral or
minimum-duration explanation for the later silent279D/280A steps. The failed
windows still lack native-vector coverage and do not establish collision.
The next single pulse remains assigned to an actual lateral step.

Private retained-only audit `run279-280-lateral-binding-audit-20261107.json`,
SHA256 `c1f1675b41e057f446337359796f813699d4096aef81bb54a4f0dbd83ce04159`,
pins the Controls captures, unchanged dispatch helpers and individual actions.
This audit launched no runtime and changed no input policy.
