# Build 209026: fallback model record selection

Static REA follow-up to [ordinary model inputs](209026-character-model-inputs.md).
No client was launched and no graphics setting or packet was changed.

## Record population and selection

AvContext initialization `0x14110f680` enumerates content **type 146
(0x92)** through content-service virtual +0x48. For each returned record,
it passes record +0x28 to `0x140d04df0`, then appends the record to the
array at whole-context +0x458 (data +0x460, count +0x46c). This happens
before the character-context subscription previously traced in that
initializer. The load/preparation call's success is not checked here;
its behavior is not exhausted by this trace.

Model helper `0x14116c240` obtains sex, race and profession through model
virtual +0x188, forms the respective `1 << value` masks, and calls
`0x14110f590`. That lookup returns the first array record for which all
three tests succeed:

| Record field | Required intersection |
| --- | --- |
| +0x30 | Race mask |
| +0x34 | Sex mask |
| +0x38 | Profession mask |

An empty list or no match returns null. The model stores that result at
+0xe8. It also derives scale +0xf0 through `0x1411674c0` using race;
that helper's scale table is not part of this trace. The ordinary
classification branch uses the existing coreStats values. Alternate
classification values and mode overrides remain as described in the
model-input note.

## Frozen content coverage

A hash-checked census of all 32 frozen content files 1282830–1282861
found nine type-146 records, all in file **1282855**, source SHA-256
`9f9226f71e3871066237d560e95b62ef33c847818a7bd78cea586507f492cfa6`.
Type metadata has no data-key field (`0xffffffff`); content offsets
identify these records. All nine sex masks are **0x3**.

| Content offset | Race mask | Profession mask | Raw +0x28 relocation value |
| --- | --- | --- | --- |
| 0x3cd8b8 | 0x1 | 0x38 | 0x1273 |
| 0x3cd918 | 0x2 | 0x38 | 0x131e |
| 0x3cd978 | 0x1c | 0x1c0 | 0x1520 |
| 0x3cd9d8 | 0x2 | 0x206 | 0x131c |
| 0x3cda38 | 0x1 | 0x206 | 0x1271 |
| 0x3cda98 | 0x1c | 0x206 | 0x151f |
| 0x3cdaf8 | 0x2 | 0x1c0 | 0x131d |
| 0x3cdb58 | 0x1c | 0x38 | 0x1521 |
| 0x3cdbb8 | 0x1 | 0x1c0 | 0x1272 |

Every combination of race 0–4, sex 0–1 and profession 1–9 has exactly
one match: **90/90**, with zero missing or ambiguous matches. Candidate
Human race 2 / Guardian profession 1 selects offset **0x3cda98** for
either sex. This establishes content coverage if the context enumeration
and classification have completed; it does not prove model loading or
renderer readiness.

The +0x28 locations are present in the file-relocation-slot list. Their
raw values above are **not resolved model file IDs**. An attempted decode
against this content file's local file-reference array failed its index
assertion; a bounded follow-up found that array empty. The reference's
resolution source is now established by the [file-fixup follow-up](209026-content-file-references.md): the base loader selects the first
container's shared table, yielding Human/Guardian model file 527140.
The raw values in the table above remain relocation indexes, not file IDs.

## Callback when replacement loading produces no model

The fallback helper's final model virtual +0x120 resolves through entry
`0x14214e578` to `0x14116c140`. It returns immediately if model +0x40 is
null, and also returns if model virtual +0x210 is false. That predicate
is `0x14116a690`: true when model +0x48 is nonnull or mode +0x38 equals 1.

On the remaining path the callback obtains a character color value (or
uses 0xffffffff), converts three bytes to float components with alpha 1,
and passes them to the existing model's virtual +0x278. It contains no
replacement model-loading call. Thus the absent-record/null-load path
cannot rely on this callback to create a missing model.

## Evidence

Private bundle: `~/re/gw2/notes/209026-character-model-records-1.json`
(1,604 records; 30,847,263 bytes), frozen executable SHA-256
`99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`.

| REA evidence | Observation |
| --- | --- |
| `ev_238c6a3003fb46ea681fb99b7db5deb377f120d1d719818bb9b53991bb02453a` | Fallback callback virtual entry. |
| `ev_a59d207b8bcecd9534e78c86b563b1ee4e6c6137e39751fd8a927ed804365909` | Bounded +0xe8-store census locating record population. |
| `ev_4bbfca4dfff3bb922d40e926ccf03abe2c191fcf685681473893c7386ca0fe2d` | Complete callback, record assignment and mask lookup. |
| `ev_c324c030c0cf9b1bc44ec749482a8b170252e24e91978aea74a3b772cc2c1fd0` | Classification outputs and ordinary coreStats path. |
| `ev_f30549290594db617a547968fbc56730d078136884b5e86ce8b2d38551684e48` | Context array population site. |
| `ev_e1e3e1a54acd72e36b7200d1f7b88487a9c0b46da68cc937c9a0d9ea82726f23` | Model +0x210 virtual entry. |
| `ev_d0498ca9db6a27b8f24dfdbfaaa8617a8d2cba9ac49cc096129cf7a2846ac2ea` | Type-146 enumeration and predicate body. |
| `ev_1b26231114304e2fe0f7a0a5edb5ec52f5a8ee2905e41fcadb3d41b32500e51a` | Hash-checked content census, file-slot membership and 90-case coverage. |
| `ev_a2fc70cc7075f50a423e9aacb0fc360951fd8d3b6486946987cc1c7268481c92` | Failed local file-reference-array decode; failure is in captured stderr. |
| `ev_ed17b4c2eca09901afeba9e9459feeaf917e00f623ac4bcede653c74908373c8` | Follow-up: local file-reference count is zero. |
