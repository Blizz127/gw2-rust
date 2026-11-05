# Private candidate: complete historical MFST75 remap set

Prepared 2026-11-05. A separate private overlay adds all **26,322 explicit
type-0/four-byte remaps** from the pinned cached build-208784 MFST75 to the
unchanged run-137 synthetic manifest records. **Run 142 passed the targeted
creator-preview rendering smoke** with the complete visible male Norn Mesmer
and no new assertion. This is a tested preview fix using that historical
metadata set. It remains **not authenticated build-209026 metadata** and
was not deployed by this investigation.

The [compatibility audit](208784-mfst75-remap-compatibility.md) established
the eligible mapping set. [Run 141](209026-texture-remap17-experiment.md#run-141-actual-failing-dimensions-captured-input-ids-remain-unknown)
subsequently observed a ready/backed **512×256 primary versus 256×128 mask**
at compacted pass 1/count 3, flags `0x81`. All four current low/high holders
were absent, leaving both original IDs and scene attribution unknown.
Consequently this candidate does not attribute that failure to a particular
descriptor channel, optional `C+8` input, or one of the separately sampled
load tasks. Run-141 evidence:
`ev_b0188c8cd6295be802cccdf62bbf890070052eb5dce08497338739f40b407fd5`.

## Preparation and verification

The retained-input preflight matched **52,644 unique source/target cached
records** to exact synthetic **base ID, file ID, decoded size and flags**:
zero missing or differing rows. These are the referenced subset of the
first manifest's **819,347** records, not a new archive enumeration.
Preflight evidence:
`ev_12f3b51cb48d2290c73e49905947db7f8fbf75e0dee9429014c098bfba9b687f`;
private receipt `~/re/gw2/notes/all26322-remaps-synthetic-record-preflight-20261105.json`,
SHA-256 `6286936a5b477d47a77834f786d6a1e67e35c1ef654c44878c1ec84ea980b848`.

A separate private Rust helper reuses the existing manifest writer. Its
strict sorted TSV input is hash-pinned; each source must be an exact base
ID, while target membership uses one prebuilt base/file alias set. It
permits at most **26,325 MFST fixups**, retains the other parsing limits,
and reserves root **2130706900**. It refuses an existing output directory
or collisions with the reserved IDs. No production generator was changed.

**12 focused tests passed**, including 26,322 synthetic properties over
52,644 synthetic records, the full 26,325-fixup boundary, rejection above
that boundary, malformed/duplicate/unsorted/missing remaps, input hash
mismatch, exact record/root preservation, secondary-manifest preservation
and no-overwrite behavior. These are helper tests, not client observations.
Evidence: `ev_eafad9ff757b247203b668ca15d1c0ff3bbcaa8cebcf05efe411e3886dff3b1d`.

Generation and independent readback used only the pinned run-137 root and
four retained MFST files, plus the retained mapping inputs. They verified:

- Every record array, build and total decoded-size field remains identical.
- All 26,322 properties and table associations equal the explicit input;
  no extra property is emitted.
- The root differs only in child base/file IDs and sizes. Names, flags,
  order, empty extras, names' encoded bytes and all remaining bytes match.
- The other three MFST files remain whole-file identical. The prior 17/20
  helpers, receipts and candidate files retain their hashes.

No archive, client runtime, deployment or production-source change occurred
in this preparation. Generation/readback exited zero with nontruncated
output and verified owned-process cleanup:
`ev_5e31ac121ff33c4a5b117c2084ae3421dc092a99ee01b68b78147228f64a17b3`.
The separate run-142 validation is recorded below.

## Pinned private artifacts

Directory: `~/re/gw2/notes/remapall-cdn-20261105/`, mode 0700; payloads 0600.
Latest line: `209026 0 0 2130706900 276`.

| File | Bytes | SHA-256 |
| --- | ---: | --- |
| Root `2130706900` | 276 | `5ad07a3894511a40cd049bc70c3c9751a86b9a5b251e22e50e19e39f7d0ee974` |
| Main MFST `2130706901` | 13,872,994 | `51a00ecd63b91c4ff97257501603bee45f35a5a8656f4bb959f049acecc38c26` |
| Each of `2130706902`, `2130706903`, `2130706904` | 112 | `195226488309fb3558de1dce3e79b797e0b6d5246be5899d24f2c09d64fd23d5` |

The selected root names/flags remain `Launcher64:0`, `English:8`,
`MapAlpineSnowline:0`, `RegionAlpine:0`.

All following paths are under `~/re/gw2/notes/`; raw mappings and evidence
remain private, outside the repository.

| Artifact | SHA-256 |
| --- | --- |
| `remapall-cdn-20261105-receipt.json` | `3ff42843e4dc6bbb2608a7cf9c5454aa698317ba6e6685710edd498477841292` |
| `remapall-cdn-20261105-evidence.json` | `d1675842452ed9eac4b9b6a9f2636093d9435a84bde9ca9029d0318db2524ad1` |
| `remapall-cdn-20261105-properties.json` — every generated property read back | `ad44eff18b0cec38881ba08f39ef7b29f25d32c4b25ffda16bbd8d1a7bfe56c2` |
| `reuse-synthetic-manifest-remaps-all-20261105` — helper executable | `421d5418c368ac29b66c06192afbda739506c5ed50f58125a86d9d89fe2efda3` |
| `reuse-synthetic-manifest-remaps-all-20261105.rs` — helper source | `22d8ec28766ba91b097cb7f9d892c3f2d8353e3c6cc00934b85abc642b9b0623` |
| `compatible-mf75-remap-pairs-all-20261105.tsv` — exact sorted input | `c71728a59886427dbfb1eb340c2d425039923e741f0810fe33d35eb01448911b` |
| `reuse-synthetic-manifest-remaps-all-20261105-receipt.json` — build/tests and prior-artifact checks | `149de7d2406661b923ca64127e5d0daa19788eef91526edbb12eeb97faf15973` |

The generation receipt binds the exact run-137 inputs, copied tool/source,
canonical mapping provenance, command, readback comparisons and prior
candidate hashes. The helper validates canonical structure/build; this
external receipt supplies the selected input-artifact identity. Neither
layer authenticates the cached historical metadata as current metadata.

## Run 142: targeted creator-preview smoke passed

`run142-norn-creator-remapall` used the candidate above with the existing
server, production runner and bounded observers. Root launched once and
completed the wrapper in **222.07 seconds**, with wrapper/UI exit zero.
The final image independently shows a complete visible **male Norn Mesmer**
on creator page **4/10**: head, body, arms, legs and clothing render, with no
native dialog. The prefix crash log remained **byte-identical to its staged
baseline**, so no new assertion was appended. This passes the changed
preview-rendering feature; it does not complete the overall retail-accuracy
goal or validate character saving, relogging or gameplay.

The final pinned safe audit is stable and matches the intermediate snapshot:
130,875 bytes, SHA-256
`68730ede8f40fa71cdf8dbd752b77ad5ab00e37a14b5ff6478a33b97ddba5807`.
It contains **23 selected loads, 23 decoded-transfer entries, 23 returns and
23 destructor retirements**, covering fourteen watched original IDs:

| Observed original IDs | Actual selected IDs, respectively | Incoming and post-transfer dimensions |
| --- | --- | --- |
| 142103, 142101, 142105 | 142104, 142102, 142106 | 512×256 |
| 140521, 140515, 140517, 140519, 140523 | 140522, 140516, 140518, 140520, 140524 | 1024×1024 |
| 140123, 140121, 140115, 140117, 140119, 140125 | 140124, 140122, 140116, 140118, 140120, 140126 | 1024×512 |

All observed tasks selected with key/task flags `0x19/0x440`, state 2.
Every sampled transfer returned one with rejection output zero, one level,
backing present, ready bit set and texture flags `0x13036`. No observed
generation remained unretired. Nine originals reached their two-generation
observation cap; the global generation/output caps were not reached, and
no load-observer `finished` record appeared. These task records are not an
independent lifetime binding to a particular creator component.

The kind-9 observer paired owner 3's scene-2 component with original source
tuple `[140123,140121,140115,140117,140119,null,140125]`, descriptor masks
`0x800/0x800`, and three advanced submissions followed by the exact parent
return. Its sixteen-submission cap was not reached. The same current owner
later reached scene 3, manager state **7**, flags **7**, texture-invalid
mask **0**, finalized model and skeleton present. Graphics readiness and
animation readiness were both true. Late snapshots at 84.383 and 97.954
seconds retained that state and **12/12** current materials with fields
`+0x30/+0x50` equal to 1. The observer finished with
`late_followup_complete`, retaining its explicit partial-coverage label.

The failure-only observer remained armed without any qualified or
unqualified size-failure event; no terminal event is expected from that
observer unless it fires or rejects. The client log contained zero watched
mask-size assertion, transfer-rejection, ImgMip, assertion-failed or Python
traceback literals. Eight generic `error:` literals remain unclassified and
are not represented as a clean-error-log claim. No observer guard, schema,
pairing or lifecycle errors occurred. All raw safe-reader output was
retained; aggregate truncation affected duplicate terminal rendering only.

Cleanup, executable hashes, the 1,124,232-entry archive index and privacy
checks passed, and root independently confirmed container absence. Window
21 was returned at **2026-11-05 23:47:28.491814 UTC**. The artifact was handed
to the existing publisher for integration and its deployment workflow;
**it was not live at this handoff**. Existing runner-feed and saved-mode
holds remained intact. No new test, runtime or archive action was used to
write this outcome.

- Runtime: `ev_19c3fbc83ca079d877ce9aef00ecafd7d94a326cedc0761f543b1bd16be59fa0`.
- Independent cleanup/unchanged crash log: `ev_56781c000554e7ecdf5a2ad0bbfd1e2d1fd5ac9815b8b5491f939b0234ac2de1`.
- Final PNG: 988,162 bytes, SHA-256 `c2db498a76ade3341fc68b04047ffa8912dd14153014546eed6e12e802aee08b`.
- Final safe audit: `ev_d6b644a99f299d848352ac189f704e129ded6939c83d2c7d607b9514fa50ab99`.
- Final safe bundle: `~/re/gw2/notes/209026-run142-final-size-failure-v2-20261105.json`, SHA-256 `b93351305e54b2937cff34983603af97ece866d53049c82a70573321bc9eed1d`.
- Controller bundle: `~/re/gw2/notes/209026-norn-creator-run142-remapall-20261105.json`, 5,311,701 bytes, SHA-256 `286280ac11a9e774fb1fd40dfbc94b3b3d93c094b16bc6ee660e32ce6271ec1b`.
