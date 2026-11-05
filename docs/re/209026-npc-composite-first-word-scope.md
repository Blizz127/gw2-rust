# Build 209026: NPC composite record first-word investigation

The first Qword of server 377's optional composite record remains
**unresolved** after this bounded source pass. The typed ordinary model
path selects the established profile/preset words at `+8/+0x10`; this pass
does not establish a valid first-word default or complete appearance input.

This source-only REA investigation used frozen executable SHA-256
`99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`.
No archive, runtime, packet, helper or target state was accessed or changed.
The existing [Eir appearance](209026-eir-alternate-appearance.md) and
[registered preset](209026-eir-preset-assets.md) facts remain intact.

## Established typed path

The server setter `1411e0950` stores the three supplied Qwords in order in
a 24-byte record at character `+0x110`. It asserts that the character is
not finalized and the record is absent. It performs no per-word validation.
The constructor-established character primary table `142160be0`, slot
`+0x1d0`, points to `141096ac0`, whose whole body returns `[RCX+0x110]`.

The complete ordinary model selector `141166a80` obtains the character
through the established view interface, calls that character slot at
`141166b27`, and keeps the record in RBP. Its normal composite branch reads
record `+0x10` at `141166c10` and `+8` at `141166c14`. The alternate token
branch also takes the profile from record `+8`. Before composite creation,
RBP is reused for the species definition. The record pointer and record
`+0` do not flow into the creation call in this complete function.

The three exact direct callers of this selector are `141168dd5`,
`14116a0d3`, and `14116d06b`, in character-model and mount-model paths.
Their inspected call regions pass the existing view and explicit model
arguments; they do not introduce a first-word use in this selection path.
The view-definition selector `1411087a0` and override predicate
`1411098f0` use the established override species pointer and character
`+0x258` token branch, respectively. Neither supplies a first-word link.

## Rejected type coincidences

The tiny getter `141096ac0` is shared code, so an exact function reference
does not by itself establish a character consumer. The two direct calls
found by the bounded exact-target query, `141352e7e` and `141354b03`, obtain
their object through `141347df0`. Their source is `EfCliEffectModel.cpp`;
the named assertion is **`renderTargetManager`**. They then call
`141359870`/`141359770`. These are not evidence that character composite
Qword 0 is a render-target pointer or identifier.

Likewise, four other references to the exact literal `compositeData`
(`14040c895`, `14040d563`, `140410df6`, `140412cb1`) concern character
appearance **component descriptors** returned by `1412d9da0`. They read
the descriptor type byte at `+8` and carry the assertion
`compositeData->type`. Their type and layout differ from the three-Qword
character record; their token, dye and component behavior cannot be
assigned to its first word.

## Input boundary

For species 3172, profile `0x0000ac0b4a6749ee` (`nornfemale`) and preset
`0x0000000001249650` (`preir`) remain sourced for Qwords 1 and 2. The zero
auxiliary value in the species array has still not been linked to Qword 0.
No name, content-key, seed, pointer or zero-default meaning is assigned.
No complete server-377 fixture follows from this pass.

This is a bounded negative result, not proof that the first word is unused
throughout the client. Exact direct-call/pointer references, the known
typed model selector and its callers, nearby constructor-bound character
getters, and the exact assertion references were checked. Arbitrary
whole-image field-offset or virtual-slot scans were not performed. A
further positive result requires another demonstrated character-record
consumer or captured provenance for the word; apparent offset or literal
matches must retain their actual types.

## Evidence

Private complete bundle:
`/home/blizz/re/gw2/notes/209026-npc-composite-first-word-scope-20261105.json`.

- `ev_e4069e571003a7cb25c049401312309dcf889f80ac9acd4c7178c310478a5a2c`:
  complete setter and ordinary model selector.
- `ev_b403abdf728619cfdef9dbc22ea05e149dbdad548ef901289dd841f4d54b75a8`:
  exact primary-table bindings read as data.
- `ev_32c899da64771f42ff3401347fd1dcb5a042391516fa6240565f1ed0c3d3fb71`:
  exact eight-byte getter at `141096ac0`; preceding partial instruction
  bytes in the window are not attributed to it.
- `ev_67bb76e59ca782ae2a4563306030fd253d0619816520d526a8dfb66d221914d1`
  and `ev_59cd10c730e4005ba8140c39e144a3293410212d1a53a3a3501185c52a9de498`:
  exact getter references and the unrelated effect-object callers.
- `ev_c4f6927dcd1d081196c997686e0ecd9098c1de232f6793a39be4451e5978c738`
  and `ev_b6bf519eb7e193912ce4635ec20c5bb4a8056d6eaa57d9fcf356f851f4e6e979`:
  exact assertion references and component-descriptor type distinction.
- `ev_fcd227f02f75e7810cc758cf48db007ed32c45f50451e23078b06b55104c895d`
  and `ev_0f3957dd8f65c4a806b5175bbc0219ae3405e6018b47692d0f91c414b8e630af`:
  exact selector callers and their bounded call regions.
- `ev_7e31cd22825d37954146e097015de7e424a3c469a145d854168e60088903cb56`:
  view species selector and override predicate.

Source captures exited zero with nontruncated raw output and verified
owned reader-process cleanup. Missing `.pdata` entries for leaf functions
were not treated as missing code. Incidental jump-table bytes were not
interpreted as instructions. Unrelated unreadable process environments
remain separate capture limitations.

## Exact packet-to-record copy follow-up

A later bounded pass followed only the demonstrated server-377 handler and
setter, without a field-offset sweep or shared-getter search. At
`141253440`, parsed optional-field-3 pointer `message+0x10` is loaded into
RDX. If present, `141253454` loads its first Qword unchanged into RDX;
`141253457` calls `1411e0950`. The setter preserves this value in RSI at
`1411e0971` and stores it at record `+0` at **`1411e0a13`**. Between those
points it checks character finalization/record absence and allocates the
24-byte record. It does not interpret, validate, or derive the first word.

The handler's complete continuation through `141253537` uses the separate
optional-field-4 pointer at `message+0x18`, restores registers and returns
one. It introduces no further optional-field-3 or first-word use. Thus the
newly checked path ends at an opaque network-to-record copy; it adds no
demonstrated semantic consumer or authored producer beyond the earlier
typed model-selection boundary. **Qword 0 remains unresolved**, and no
default or complete Eir packet follows. The existing profile/preset proof
is unchanged. No fixture, runtime or archive work was performed.

Fresh source evidence: handler/setter
`ev_b10b0fec7e02cc2c2c0efd248d325ce772222a5f631e6f385b6d93429268dfbb`
and aligned handler tail
`ev_8df7d355196b0ac8283d06778d07b65dc3a6796d7bc8cfded25f56fbd1b254af`.
The tail capture completes the partial last instruction of the first handler
window. Complete private export:
`~/re/gw2/notes/209026-npc-composite-first-word-copy-boundary-20261105.json`,
SHA-256 `d12b0f3429c436974539ed78aeb062ffd7b3d592aaf738f73d1e447985aa62b8`.
