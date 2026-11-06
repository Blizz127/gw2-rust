# Build 209026: repeat-attack order admission correction

**Superseded request mapping:** the later
[input lifecycle trace](209026-skill-input-action-lifecycle.md) proves that
0x29 is reached by deactivation/end paths. The
[slot-release correction](209026-skill-slot-release-correction.md) removes
the synthesized cast/hit. The c862325 artifact below is retained history,
not the latest releasable build. The separate admission-mask finding remains
valid for actual incoming orders.

2026-11-06. The existing repeat-attack fixture sent server34/typed0x78 with
the request's outer flags copied unchanged. The frozen client's repeat
producer sends flags3; the inspected controlled-empty-slot receiver requires
mask0x10 before it will construct that order. This was the same omission
already corrected in the separate SkillBegin echo diagnostic.

The RepeatStart response now ORs0x10 into the request flags while preserving
the remaining flags, agent, remote tick, behavior value, skill-bar slot,
configured skill and selected target. This corrects that admission gate.
It does not establish retail repetition, chain progression, cast duration,
effect timing, correlation policy or completion/release sequencing.

## Retained client evidence

The [echo preconditions](209026-skill-order-echo-preconditions.md) retain the
complete incoming0x78 path: at0x14127f393, mask0x10 admits the factory branch.
With it clear,0x14127f3a7 checks controlled state and the empty-slot branch
returns at0x14127f429. This common path does not select different admission
rules according to whether the initiating request used packed0x23 or0x29.
Mask0x10 also bypasses a conditional earlier bound-agent filter, as already
documented there; this remains an explicit research response choice rather
than a recovered universal retail flag byte.

Complete repeat producer0x1412848f0..0x141284a3c allocates a new behavior
container and writes two packed bytes,0x29 and the low byte of its slot
argument. The dispatch at0x141284993 supplies outer flags3. It subsequently
updates local player/skill-bar state, but that does not bypass the common
incoming0x78 gate above. REA
`ev_45911bb23b1facbe58ab9961c5869a4295ada34918852b561604e1be281c22f2`
is retained in the private
`~/re/gw2/notes/209026-skill-cancel-request-correlation-20261106.json`,
SHA-256 `f2627d2758c9f22f9ce82448452db18acd1137aa9e7e21c6aacfb3f7b1eb3a7d`.
The frozen executable identity and surrounding correlation limits are in
the [cancellation note](209026-skill-cancel-request-correlation.md).

## Regression and verification scope

The new regression first failed on the original branch: the decoded server34
header retained3 where the required admitted response was0x13. After the fix,
all three focused auto-attack tests pass. The final regression sends complete
client23 frames through the mission parser, responds to the decoded0x29,
and decodes the complete server34 response. It checks native flags3, zero,
an already-admitted byte and unrelated high/track bits. It also checks the
other header fields, exact packed0x78 fields, complete frame boundary and
unchanged hostile health when fixture damage is disabled. Existing target
selection and damage behavior tests continue to pass.

Command:
`cargo test -p gw2-net --bin gw2-game-observe mission::tests::auto_attack -- --test-threads=1`.

## Built artifact, not deployed

The isolated source worktree is
`/home/blizz/scratch/gw2-repeat-admission-20261106-src`, commit **c862325**.
Its dedicated target directory avoids reuse with the active working tree.
`cargo build --offline -p gw2-net --bin gw2-game-observe --release --target x86_64-unknown-linux-musl`
passes and produces a static-PIE executable:

- Path: `/home/blizz/scratch/gw2-repeat-admission-20261106-target/x86_64-unknown-linux-musl/release/gw2-game-observe`.
- Size: **1,582,024 bytes**.
- SHA-256: **834c723f8d38139f7b60cd10700629e415fe333fc0c08db684880a8c7ed8e439**.

The first isolated build lacked the four git-ignored extracted schema inputs.
Exact copies from the already-tested root tree were then supplied privately,
with hashes verified before the successful build. Their receipt is
`~/re/gw2/notes/repeat-admission-build-inputs-20261106.json`. No source or
schema substitution was inferred from a different client version.

At **2026-11-06 01:41:33 UTC**, that exact binary passed two
`--no-capture --check-config` startup checks: prepared auth and prepared
Norn mission inputs. The mission check enables the corrected repeat option
with a synthetic hostile argument and damage amount; it checks argument/
fixture construction, not an authored NPC or rendered attack. Both checks
report no listener started. Eight existing fixture input files were verified
against the pinned run145 manifest and remained unchanged. The run145 script,
staged binary, manifest and NOT-LAUNCH-READY state were not replaced.

Check wrapper and retained receipt:
`~/re/gw2/notes/check-repeat-admission-build-20261106.py` and
`~/re/gw2/notes/repeat-admission-build-checks-20261106.json`.
No native runtime, working archive, shared container, saved-character store,
publisher process, feed or online service was accessed by these checks.

The existing1000ms duration remains an explicitly labeled diagnostic
placeholder. This correction does not derive a weapon's cast duration or
change gameplay defaults. The standalone run145 Ether Feast observation
remains pinned to its previously staged binary and unlaunched. The saved
roster native question, resource lease and sole-publisher boundaries remain
unchanged; this note is not a deployment or successful-client-cast receipt.
