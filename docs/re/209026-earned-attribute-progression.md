# Build 209026: explicit earned-attribute progression

The [starter baseline](209026-starter-base-attributes.md) populated the four
primary attributes, but the XP path still changed only earned level. Crossing
2,000 XP therefore sent level 2 while leaving earned attributes at 37. A
waypoint rejoin recreated that mismatch after restoring the earned level.

`--mission-base-attributes default-curve` now explicitly selects the frozen
[default primary-attribute curve](209026-default-primary-attribute-curve.md).
It starts earned/effective selectors 1–4 at 37, with the separate signed
column zero, just as the earlier fixed37 profile did. On earned-level changes
it sends four server187 updates after server188: level 2 receives 44, level 3
receives 51, and level 80 receives 1000. Level skips choose the reached row.
Rejoin restores the earned values after restoring XP and earned level.

This option is an unmodified profile model: no gear, account flag or other
modifier is inferred. The verified table contains all 101 source rows; the
player lookup accepts only levels 1–80. Existing numeric settings retain
their fixed values and do not acquire an inferred progression curve. Omitting
the setting preserves the previous population and XP output.

## Effective state remains a separate requirement

Server187 updates the earned column. The earned-level update does not establish
the policy selecting the second level or effective attributes, so this change
does not send189 on level-up or alter health. In this explicit profile the
second level and effective baseline remain at their initial level-one values.
That remaining limitation is not claimed to reproduce retail scaling. Skill
execution, localization keys and authored Great Hunt encounters also remain
incomplete. XP and heart state still reset when the server restarts.

### Source audit: health and healing-power baseline

A follow-up exact-source audit confirms that health388 and attribute6=0 are
consistent with the current unmodified level-one Mesmer. The baseline leaf
returns0 for selectors5 and above; prefix112 and the absent upgrades add no
attribute6. Earned-level growth therefore does not by itself require a healing
power increase. The missing progression input is the independently selected
second level, followed by matching effective attributes and health updates.

The complete101-row player-health LUT and retained Mesmer coefficient/default
primary curve give conditional health baselines388 at level1,476 at2,564 at3
and15922 at80. These are results for an explicitly selected second level,
not a recovered rule selecting it from earned level and map/sector. Server190
accepts the selected level;189 independently supplies effective attributes,
and256/259 supply health. Updating one without the others or without the heal
evaluator and character-bound carry would leave inconsistent state. How current
health changes when maximum health changes is also an unresolved server policy.

Private audit receipt: `mesmer-effective-health-progression-receipt-20261106.json`,
SHA-256 `d760bc10e3238f7e2361a518b8f9f726058417a05695da4c1dc9425051c708a3`.
Fresh LUT/evaluation REA:
`ev_d22bc5d33e224afc091dfef25a9f97e1aa822bb0218e172abcc0ffc3d78daa1a`,
complete raw/normalized output, exit0 and verified owned cleanup. Full bundle
`209026-mesmer-effective-health-progression-evidence-20261106.json`, SHA-256
`bdada5d1771e578b3859781c578ba20476387146e2f7097170822102135396f0`.
The earlier missing-cache attempt is retained; no archive fallback ran.

### Authored map and sector display levels

A targeted frozen-source follow-up recovers concrete display inputs from
already cached content. Type45 map bounds at +0xa8/+0xac are **1–15 for
map28** (1282830:3731704) and **1–1 for map379** (1282833:3863896).
Type63 sector +0x40 values are **200→5, 201→2, 202→7, 512→7**. All four
sector definitions reference map28; the tutorial layout shares200/512/201
but does not include202. This does not observe the current actor's sector.

The typed consumers establish display semantics. `WmmLabel.cpp` helper
`0x140978dd0` obtains coreStats through character interface +0xa8 and calls
its virtual0 getter, which reads earned +0xac with a separately gated80
override. It clamps that result to the map bounds. Leaf `0x1411c3100`
converts the level difference into a confidence enum; `0x1414bc2d0`, named
by `Confidence.cpp` and `currentConfidence`, converts the enum into an
ARGB color. The repeated label path applies that color to render objects.
The map-text branch at `0x140978b54` also formats the two bounds. Sector
branch `0x1409789f9..0x140978ae2` follows the typed layout references,
reads sector +0x40 and formats it into the sector-name label. Its exact
localized wording was not decoded; no scaling-cap meaning is assigned.

**These paths do not select or update second level190.** The map clamp is
a confidence-color calculation, not a recovered downscaling formula.
The remaining edge is a rule choosing190 from earned level and current
map/sector, including any area offset or override, followed by matching189
and health policy. The explicit received190 setter is established; this
bounded audit does not prove that no other client helper exists. Neither
`min(earned, map maximum)`, the sector label value, nor second=earned is
justified as retail policy by these observations. Level-one health388
remains source-consistent with the current explicit level-one state.

Private receipt: `map-sector-level-display-receipt-20261106.json`, SHA-256
`45f351551016585ef96ae5ef0b0bfdf2b6b4106de05d789cc02ed4cb13a03ae2`.
Full12-capture REA bundle: `209026-map-sector-level-display-evidence-20261106.json`,
SHA-256 `e94a9b1567505a2286a0be1937140034a929b85f14f1c36902a334213fbc7d70`.
Exact cached rows are in `ev_9ada054279fa1118b7413e24f99b5432abd4152176bb37355e71e517b95eb102`;
typed label/color bodies in `ev_556a5e6b739127ccd3d924481815609b0301933a37438b4bd40bf7dd3cd093e3`,
`ev_d7f07429c1e0b881bb1e27bfc2737fd9dae43445e02339089a050f3234d34442`
and `ev_750104c1f92ec44db7ca3dc0986d85a2bae7cb784591981254eb2f813cbf1af3`.
All captures exited0 without truncation and with verified owned cleanup;
cache presence and hashes were checked with archive fallback disabled.

## Configuration and checks

The existing supervisor forwards `GW2_BASE_ATTRIBUTES=default-curve` to the
world role and its preflight. It changes no saved/prepared roster selection.
Candidate profile:
`~/re/gw2/notes/mission-profile-attribute-curve-20261106.conf`, SHA256
**d7e25b1da72a9354f7cf84c7d46ad893158fac3c24a1cda13a68d73a6a9ee3ef**.

Compared with the pinned cdn900 profile, this adds only that setting and removes
the retired `GW2_AUTOATTACK_SKILL=10170`, preserving all other values. The
baseline and earlier fixed37 candidate remain untouched. Exact paths and pins
are in `~/re/gw2/notes/attribute-curve-profile-20261106.json`.

- Two curve tests pass, and REA verified all 101 module constants against the
  pinned decoded source table. No new archive extraction was needed.
- Four focused base-attribute tests pass: explicit parsing, exact startup
  bytes, saved GUID selection, threshold crossing, saturation and fixed/omitted
  compatibility.
- All 39 mission tests pass. The isolated waypoint lifecycle checks ordinary
  kill XP, heart XP and restored earned values, with no effective-column
  updates and no repeated heart reward.
- All seven saved-population tests and the no-listener saved configuration
  test pass. The latter accepts fixed0/37/65535 and the explicit curve.
- Three focused supervisor tests pass for default behavior and prepared/saved
  forwarding. These use fake children and do not start game listeners.

These are offline server-output checks. No new shipped-client observation or
online deployment is claimed. The sole publisher's native owner question,
run145 hold and resource requirements remain unchanged.

## Checked cumulative artifact, not deployed

Source **9a8eb71** includes the fixed starter baseline, slot-release correction
and explicit skill-order composition changes. Its offline static-musl release
build passed in the isolated source tree with its own target directory.

- Binary: `/home/blizz/scratch/gw2-attribute-curve-20261106-target/x86_64-unknown-linux-musl/release/gw2-game-observe`.
- Size: **1582816 bytes**, static PIE.
- SHA256: **65a37f930a86b6e0820ae7c4cef7aa7affb472a321e8972ec05eff11b7c3e084**.
- Paired supervisor: `/home/blizz/scratch/gw2-attribute-curve-20261106-src/deploy/gw2-server.sh`.
- Supervisor SHA256: **c006b89524e011039a9b5bdb4df913d2505e82a9f44f7e69a31cd02b8735b6ad**.

At **2026-11-06T02:56:49.008725Z**, this exact binary passed four
`--no-capture --check-config` cases: prepared auth, prepared mission with the
curve, prepared mission with fixed37, and saved mission with the curve. Each
exited successfully without a listener. The saved case used an empty private
temporary store. Mission cases also retained the earlier explicit synthetic
damage/order test inputs; no retail timing claim follows.

The retired CLI rejection, two legacy-setting compatibility checks and shell
syntax passed. Eight fixture files, the run145 manifest and four schema input
pins remained unchanged. Private receipts and wrapper:

- `~/re/gw2/notes/attribute-curve-build-inputs-20261106.json`.
- `~/re/gw2/notes/attribute-curve-build-checks-20261106.json`.
- `~/re/gw2/notes/check-attribute-curve-build-20261106.py`.

Pair the binary and supervisor with the candidate profile above. If the
publisher's current profile differs from the recorded baseline, apply only
the stated setting deltas and preserve other work and roster mode. Previous
tested artifacts remain intact; this candidate is cumulative. Last verified
live remains **82d8d04-cdn900**, deployed **2026-11-05T23:56:31Z** at
`/home/blizz/gw2-rust-82d8d04-cdn900`, with rollback retained. No release,
resource-lease acquisition or native prompt answer occurred here.

## Independent port onto live world-handover source

Commit **e78b02857d519a740862ef93aec1311479388dd9** ports only the baseline
and earned-curve increment onto76af245. It preserves multi-heart/story work,
GUID/destination carry and map28 arrival. Fifteen focused Rust tests and
three supervisor tests passed, including multi-heart level gains and
character-bound carry with earned-attribute restoration. The isolated musl
release and four prepared/saved auth/world preflights passed; the latter
kept the target listen addresses occupied to prove no listener was created.

Artifact:
`/home/blizz/scratch/gw2-earned-attributes-76af245-20261106-target/x86_64-unknown-linux-musl/release/gw2-game-observe`,
1732176 bytes, SHA-256
`3233c1d1e3e488fb338dfceb4154cbcb487e672b8c65239b291a5cfcab2fb0db`.
The only profile delta is `GW2_BASE_ATTRIBUTES=default-curve`; retain all
latest publisher values, including explicitly empty legacy autoattack,
17-byte skills, damage30, echo disabled and saved mode. Do not substitute
the earlier whole profile recorded above. The paired supervisor is in the
commit. Effective-level/health scaling and restart persistence remain absent.

Root reviewed the production diff, checked the commit/artifact and preflight
receipts, and submitted the concrete ready handoff through the existing
native publisher inbox. This is not a deployment receipt. Evidence:
`earned-attributes-76af245-20261106-checks/profile-preflight.json` and
`earned-attributes-native-publisher-handoff-20261106.json` under private notes.
There is no added client-run or routine approval gate.

At06:48:23Z the existing publisher source includes e78b028 under0570b950.
Its shared queue records the lane-B attribute smoke, while the latest release
receipt remains76af245. Integration is not deployment. The bounded watch
ended without invoking a live verifier or claiming a newer kit live.
Aggregate verification receipt:
`earned-attributes-76af245-20261106-checks/verification-receipt.json`, SHA-256
`b87f43a0088bf37f7184e071bcc4da156624acfc9396d3b0c1e78759d8759e37`.
Publication watch: `earned-attributes-release-watch-20261106.json`, SHA-256
`4539fba7eeae5b2990091ce8d470b7ce3aa07493d9daded46e2d6829973a1b36`.

## Verified live release 0763c47

The sole publisher shipped this increment separately at
**2026-11-06T06:54:42Z**, kit `/home/blizz/gw2-rust-0763c47`, rollback
`/home/blizz/gw2-rust-76af245`. The later discovery branch remains separate.
Root independently verified the running release at **06:55:45Z**:

- Both complete kit manifests and expected game/manifest pins pass.
- Four listeners belong to their expected processes; zero humans/bots.
- The game process uses `--mission-base-attributes default-curve`.
- The17-byte skills, damage30, absence of autoattack/echo, map28 and14 heart
  NPCs remain effective. Existing heart data hash also matches.
- Saved-norn mode, one0700 record store and unchanged activation backup
  pass. Character contents were not read; no configuration was modified.

Game SHA-256:
`b3478950101539115d46f006c33df786a7e178cbb00d22b1fe2d74a6e1d2f551`.
Manifest SHA-256:
`bbbc55b6f3cd251ff2fa430f5877c64e8eb10365b0663f123ce386ed49156043`.
Receipt: `209026-live-0763c47-readback-20261106.json`, REA
`ev_f9a09a24c6752a1266d164ba78b67afcad5cd56b358104cffabe95a836241852`.
Read-only verifier exited0. Root reported the result to the existing native
publisher and coordinator and marked the e78 handoff published. No duplicate
release or launcher publication was requested. This release does not complete
second-level/effective-attribute scaling, health growth or restart persistence.
