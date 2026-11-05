# Build 209026: 17-remap compatibility experiment

**Run 139 failed its feature smoke.** The first observed task selected the
mapped texture and loaded larger dimensions, but the client then displayed
a native crash dialog. The new assertion was **`Composite mask smaller
than source RGB texture`**, `GrTex2d.cpp(2169)`. The private overlay is not
releasable and was not deployed. Wrapper/UI exit zero does not override
the failed visual and native-assertion checks.

## Inputs and provenance

`run139-norn-creator-remap17` reused run 138's server cd4c3b6, production
runner63bc59a plus both passive observers, and the same UI actions/timing.
The only client-semantic input change was the synthetic CDN root and its
seventeen exact type-0 file-ID properties. The larger idle bound corrected
run 138's controller timeout: 360 seconds idle, 480 seconds total.

The values come from [authenticated cached build-208784 metadata](209026-cached-texture-remap-metadata.md),
whose 34 original/mapped record sizes and file-ID aliases match frozen
209026. They are a source-derived compatibility experiment, **not
authenticated build-209026 manifest metadata**. No adjacent-ID rule was used.

Private candidate: `~/re/gw2/notes/remap17-cdn-20261105/`.
Latest line: `209026 0 0 2130706700 276`.

| File | Bytes | SHA-256 |
| --- | --- | --- |
| 2130706700 | 276 | `eb7d6e6a09c4b5477e214cf3d552a792de72804f2e5b05df2680f067d6f7a281` |
| 2130706701 | 13,110,149 | `3846d11d2aa87f9bc7becf2c31353ee354c005547eb3fef3a876172f7f3cf53d` |
| 2130706702, 2130706703, 2130706704 | 112 each | `195226488309fb3558de1dce3e79b797e0b6d5246be5899d24f2c09d64fd23d5` |

All 819,350 existing record rows remained byte-identical. The other three
MFSTs remained wholly identical; root names/flags were preserved, with only
child IDs and the first manifest's size changed. Exactly seventeen property
rows were admitted. Verification:
`ev_1a77fda6935fe41d75b38566ca7b364b46b91c0e595445548abc6370b26caa6a`.
Receipt `remap17-cdn-20261105-receipt.json`, SHA-256
`64df2209634a39289aa1abae5bc52a983d46121a34f15fe77707d95832cc3e40`.

Two full archive-based generation attempts timed out cleanly at 50/240
seconds before producing manifests. Their evidence was retained. The
successful bounded helper reused the existing verified record array and
the repository's manifest writer without archive access. Five synthetic
checks passed. Helper SHA-256:
`45fb11f76c20b6d031abd94391804e2bf551190407629100193e66d28a64098d`.

The 1,839-entry run input manifest SHA-256 was
`f7206afa70ca85a47819c6b2ff1dce9738daceb07d753c4896d52feac29d8b4c`;
combined GDB SHA-256 remained
`a991cc97565d84f5ab9dd50cf34cc767c0de5285724171003b397ce2b7b4e400`.
Two configuration preflights, five shell syntax checks and both GDB Python
blocks passed. Before launch, the wrapper compared mapped working assets
140124, 140122 and 140522 against their exact frozen byte hashes.

## Native result and failed visual smoke

The observer retained one complete task generation:

| Field | Property-free run 138 | Run 139 |
| --- | --- | --- |
| Original ID | 142103 | 142103 |
| Selected load ID | 142103 | **142104** |
| Key/task flags | `0x19 / 0x420` | **`0x19 / 0x440`** |
| Incoming and post-call dimensions | 256×128 | **512×256** |
| Format/levels | 26 / 1 | 26 / 1 |

Run 139's transfer returned one, with rejection output zero, backing
present, ready bit set and flags `0x13036`. Its destructor retirement was
observed. No watched generations or frames remained, no cap was reached,
and there were no observer guard, read, parser or pairing failures. The
other sixteen original IDs were unobserved. This does not assign the load
to a specific scene owner or prove successful pixels/rendering.

The creator observer completed its initial snapshot handshake, then bound
owner 2 in scene 0. There was no scene-3 tuple, kind-9 dispatch/advanced
submission/return, late snapshot or `finished` event. Neither observer
reported completion. The final screenshot remained on the race-selection
screen with a native crash dialog; intended Norn appearance was not reached.
Final PNG SHA-256:
`6686d8e582e6912680c3aff11f1a0e30edeb16fefbc296c5d9d77da564ce76c3`.

The prefix crash log contains inherited historical assertions as well.
Root verified that its staged baseline matched the input manifest and
that the new file was an exact append. **Only the new suffix** contains
the mask-size assertion above. Delta/image/container evidence:
`ev_a89e901955ca6dd2282757891450ffadfb2de8b334c73716b17bb4b73c662081`.

This result establishes that supplying a real remap changes native file
selection and loaded dimensions. The assertion establishes a compositing
size mismatch. An incomplete set of companion-mask mappings is a concrete
hypothesis; the failing descriptor/mask identity is not yet observed.
The next source step must recover the complete authored source-channel
set rather than infer companion IDs or bypass the assertion.

## Cleanup and retained evidence

Wrapper and UI exited zero. The exact owned container was removed, both
executable hashes remained unchanged, the working archive retained its
identity and 1,124,232-entry index, and capture/console privacy checks passed.
Root independently confirmed shared-container absence. Window 18 was
returned at **2026-11-05 22:57:53 UTC**, with run 138 marked incomplete and
run 139 marked feature-failed. No production or saved-mode setting changed.

- Runtime: `ev_cee433b7653af690061801c4a48ac0ab1187fc1f504e25db17e79667c2fa4649`.
- Final safe audit: `ev_8de7b4993944e2e10d4c73d9590c201b73be0ce621f2138149c4b3d7c22eabb2`.
- Controller bundle: `~/re/gw2/notes/209026-norn-creator-run139-remap17-20261105.json`.
- Complete safe audit: `~/re/gw2/notes/209026-run139-texture-load-safe-audit-20261105.json`,
  SHA-256 `858d28fe068d5e0ef623d2d2c03f2c734a982266137acbccfd5387f862c41b62`.

## Run 140: twenty-remap follow-up also failed

`run140-norn-creator-remap20` added the three explicitly sourced companion
properties `142101→142102`, `142105→142106`, and `142125→142126`, using
synthetic root **2130706800**. This completed the retained authored
seven-channel cohorts for the seventeen original inputs; it did not establish
coverage of every optional variant or every runtime operation. The same
cached-build-208784 provenance boundary applies. See the
[primary-142103 cohort note](209026-composite-primary-142103-cohort.md).

The server remained cd4c3b6. The twenty-ID load observer was pinned to
SHA-256 `13f9e2a4751649eb04f1cd259fff9575a81cbd04652e24fdf0bea1933ddc7f5c`,
with forty generations, two per original ID and 192 output records. The
kind-9 observer remained
`70729da1925b2ae6e7761822fe5768d96a13d97b57938496430b7995657a76a1`.
Root launched once with 480-second total and 360-second idle bounds.

The final safe audit contains exactly these three native task generations:

| Generation | Original → selected ID | Observed lifecycle |
| --- | --- | --- |
| 1 | 142103 → 142104 | Selected; incoming and post-transfer 512×256, format 26, one level; transfer returned; destructor retirement observed |
| 2 | 142101 → 142102 | Selected; incoming and post-transfer 512×256, format 27, one level; transfer returned; no destructor retirement observed |
| 3 | 142105 → 142106 | Selected only; no decoded-transfer entry, return or destructor retirement observed |

All three selected tasks had key flags `0x19`, task flags `0x440` and task
state 2. Both sampled transfer calls returned one with rejection output
zero; their post-call fields had backing present, ready bit set and flags
`0x13036`. These are current native task observations, **not a binding to
the failing operation or a particular scene owner**. Original 142125 and
the other sixteen watched IDs were unobserved. Generations 2 and 3 remained
unretired in the captured observer lifecycle. No generation/output cap,
guard failure, parser rejection or pairing error was reported; no `finished`
record was observed.

The kind-9 observer bound owners 1 and 2, both in scene 0. Owner 1's initial
soak snapshot sampled fourteen materials, all with fields `+0x30/+0x50`
equal to 1, while its manager remained state 6, flags `0x1007`, texture
invalid mask `0x7fff` and cached graphics readiness false. There was no
scene-2/3 dispatch, paired advanced submission, appearance snapshot or
observer completion. Thus this run did not reach the intended appearance
comparison.

**The feature smoke failed again.** Root's final image shows the native
crash dialog over race selection. The exact append-only crash-log delta
contains a new `Composite mask smaller than source RGB texture` assertion,
`GrTex2d.cpp(2169)`, build 209026. The final safe client-log audit contains
one occurrence of that literal. Both primary and sampled mask transferred
at 512×256, so the previous explanation that this sampled mask remained at
its original smaller dimensions does not account for these records. The
actual failing pass and operands were still unobserved; equality of these
two task snapshots does not establish equality of the failing operands.

Wrapper and UI exited zero. Cleanup, executable-hash, archive-index and
privacy checks passed, and root independently verified shared-container
absence and returned window 19. **Wrapper/UI zero is not a feature pass.**
No overlay was deployed. No further runtime or archive reads were used to
write this outcome.

- Controller: `ev_914957ab9f4b07fb326b96dc6b7f96e74fff0c8c174bd1078d8728635c7d2f7b`.
- New crash delta/image: `ev_31a56cd7056ec2bcbbef740a08d31e0b23e3f343e2f01f30a7625c920cfb009d`.
- Final PNG SHA-256: `1b9a47a6592e0f2e2aec27ac3dc76e98d89601efce5f8f1a5fa533a6fe4aa417`.
- Final safe audit: `ev_e3f40ade01872c7a9df3afd8ee6319c80903eb983c6ca4f0e9d34ef6f0b62bf9`, complete and untruncated.
- Audited client log: 66,838 bytes, SHA-256 `075fe1cdc86038d3b373df4a825256cc510b3077c27e30efe0379d4038e55003`.
- Controller bundle: `~/re/gw2/notes/209026-norn-creator-run140-remap20-20261105.json`, SHA-256 `72b1642e728ec7c0dda5d1db07f0a5b61e56e61f9e7493bfdb53453785472e80`.
- Safe evidence bundle: `~/re/gw2/notes/209026-run140-texture-load-safe-audit-20261105.json`, SHA-256 `a3407804bc99e29918bbaaea24f4ec41d39bb9c262251d5894d64347e99469c4`.

The next bounded observation is the native failure branch's current compacted
pass, primary/mask dimensions and same-call source relations. Its
[source contract](209026-composite-mask-size-contract.md) does not require
guessing a missing remap or bypassing the assertion.
