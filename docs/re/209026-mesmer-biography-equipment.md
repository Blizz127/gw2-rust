# Build 209026: Mesmer biography equipment

2026-11-05, bounded read-only REA analysis of the frozen executable and
archive. For **Norn, profession7, level1**, the selected Mesmer biography
answer supplies one additional armor item beyond creation template210:

| Submitted answer byte | Answer array index | Item data key | Default skin data key | Prefix data key | Equipment slot |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 187 | 0 | 6551 | 2058 | 112 | 5 |
| 188 | 1 | 6552 | 2059 | 112 | 5 |
| 189 | 2 | 6553 | 2060 | 112 | 5 |

These are references selected by the client creation preview, not observed
retail inventory messages. The mapping supports deriving a saved-character
candidate loadout from actual request415 answers. It does not supply item
instance IDs, unresolved creation-message flags, upgrades, or server-side
ownership rules. No client was launched and no protocol code was changed
for this investigation.

## Submitted answer to selected choice

The [creation-answer record](209026-creation-answers.md) is type260 in
archive file1282831, content offset3991224. Its question at offset3994376
has index188 and the exact internal name `Profession Mesmer`. The question
and its three answers have applicability flags0x11f. The answer records,
in array order, are at offsets4006488/4006552/4006616 and hold values
187/188/189 at +0x14. The field is read as a Dword by the selection
consumer; these three values fit the request's byte vector. Question
index188 and answer byte188 are separate values.

Creation context vtable `0x14217dca8` resolves virtual +0x10 to
`0x14129d670`. It finds a question by its internal name at question +0x10,
returns the question through an output pointer, and dispatches virtual
+8 (`0x14129d7f0`) to find its selected answer. That function:

1. Checks question applicability through `0x1411c5710`, with the current
   profession and race.
2. Walks the saved creation-data answer bytes at +0x40/count +0x4c.
3. Walks the question's +0x20/count +0x28 answer array, stride0x40,
   checks each answer's applicability, and compares its +0x14 Dword
   to the zero-extended saved byte.
4. Returns the first matching answer pointer, or null.

This connects the selection to the same answer vector forwarded by the
[request415 sender](209026-creation-answers.md). It is not inferred from
answer ordering in a screenshot. Global byte uniqueness is unnecessary:
the lookup has already selected the profession question and checks race
and profession applicability.

`0x14129e840` gets the profession through virtual +0x80. For profession7,
its switch selects the literal `Profession Mesmer` at `0x14217e138`.
It calls virtual +0x10 and finds the returned answer pointer in the
question's array, yielding its zero-based index. A missing answer falls
back to index0 in this preview helper. That fallback does **not** prove a
server creation policy for missing or contradictory answers.

## Choice to content reference

The [equipment preview](209026-creation-equipment.md) performs a
profession-specific override after the base template when its input flag
bit2 is clear. At `0x14129b91d`, it calls context virtual +0x70 with
mode0 and slot0x44. The vtable target is `0x14129e5c0`; its mode0 branch
obtains the selected profession answer index through `0x14129e840`.
It supplies race, profession, level, and this index to `0x1411c34d0`.
The mode0 branch ignores the passed slot when constructing the name.

`0x1411c34d0` adds one to the answer index and formats:

```text
Gw2.Common.Configuration.Level %d %s %s Madlib %d Item
```

The race/profession helpers `0x1411c6740` / `0x1411c5f40` return `Norn`
and `Mesmer` for race3/profession7. It requests content type150 through
content-service virtual +0x50. If no record is found, it decrements the
level and retries until zero; this lower-level fallback belongs to this
named override selector, not the separate template selector. All three
level1 names resolve directly in the frozen archive.

The returned type150 record must have +0x28 equal to35. The helper
`0x1412e3ba0(record+0x30)` also checks the embedded tag35 and returns
the relocated item pointer at record +0x38. The three selected answer
records themselves contain no direct item-reference relocation.

The archive's content flags2 activate name mangling in `0x1412e6650`.
It calls `0x1412e4df0`, which splits at the final period and shortens
the namespace and leaf separately through `0x141481860`. That helper
hashes the UTF-16LE bytes through the SHA-256 routine `0x141578e20`,
folds the digest with the shown 64-bit multiply/xor loop, Base64-encodes
the little-endian result, and retains five characters. The loop operates
on successive shifted Dword values; substituting ordinary bytewise FNV
would not reproduce it. The two short strings are joined by a period.
The content lookup then uses `0x14130c150` for its name hash and also
compares the string. Matching only an unmangled hash is insufficient.

The source-reading helper reproduced that transformation, checked both
the shortened string and hash, and scanned all32 base containers for
type150 matches. It found exactly one record per requested name:

| Choice suffix | Shortened name | Name hash | Type150 file / offset | Item35 file / offset | Skin66 file / offset |
| ---: | --- | --- | --- | --- | --- |
| 1 | `ZnY31.+kHUZ` | 0x466ceaed | 1282832 / 220880 | 1282841 / 1493232 | 1282853 / 4164448 |
| 2 | `ZnY31.1coa7` | 0x2ff3a038 | 1282832 / 285592 | 1282841 / 1512160 | 1282853 / 4165696 |
| 3 | `ZnY31.Dbz8n` | 0xa37c75ef | 1282832 / 236552 | 1282841 / 1495152 | 1282853 / 4160984 |

All three item +0x2c kinds are0. Their armor subrecords are respectively
at file1282841 offsets1493408/1512336/1495328. Each has category3 at +8,
default-skin reference at +0, and prefix reference at +0x38. The common
prefix is type37 key112, file1282843 offset2968184. Armor +0x68 is1
and the skin discriminator +0x80 is0 in all three, satisfying the inspected
armor-preview branches.

## Equipment slot and colors

The override armor branch reads item +0x30, passes armor +8 to
`0x1411bf460`, and invokes the same preview helper `0x14129b0c0` used
by base-template armor. Jump-table entry3 at `0x1411bf4dc` targets
`0x1411bf4a7`, which returns **equipment slot5**. This supplies the
slot independently of any localized mask name or summary-record order.

The [creation color matrix](209026-creation-colors.md) uses four values
per equipment slot. Request415 sends rows2 through7, so this additional
item uses **row5: final-array indexes12,13,14,15**, in that order.
For a saved character, those submitted selections are the appropriate
input; the other three armor rows must not be reused for the mask.
The retained Norn male/profession7 archived default has row5
`[117, 615, 117, 0]`. That is source default input, not a replacement for
the user's submitted colors or a complete later color-fallback policy.
Request415 fields are Dwords while armor960 color fields are Words:
unsupported values above65535 must not be silently truncated.

The archived appearance extraction is key57, file1282852 record113352,
string slot119600/index346. Its appearance SHA-256 is
`b3b957c919a12d3b030b3ed4cc75c0117b2fb564c5859b18e2d97b98af7f9c4f`;
its59-row color file SHA-256 is
`f3f02b91e8dd9685053392838cc2fe4e94bc7283ec190f442abbd70da205b578`.

Prefix112 still has empty primary/alternate attribute arrays and a null
effect reference. Under the explicit no-upgrade assumptions in the
[starter health profile](209026-mesmer-starter-equipment-health.md),
adding this item contributes no prefix attributes and does not change
the sourced unmodified health388. This makes no claim about armor values,
damage reduction, or unexamined server modifiers.

## Saved-character policy boundary

A concrete candidate implementation can retain all raw request415
answers, identify exactly one applicable answer from the Mesmer question,
resolve the corresponding item/skin/prefix references above, and add that
item at slot5 using the submitted row5 colors. A missing or ambiguous
selection should remain an explicit unsupported input, rather than being
silently replaced by the preview's first-choice fallback. This is a
recommended emulator validation rule, not an observed client rejection.

The exact archive mapping in this note is scoped to Norn/profession7/
level1. Sex is absent from this override lookup's arguments. Other races,
professions, levels, race-story overrides, localized names, inventory
instance creation parameters and summary-equipment ordering remain outside
this extraction. The base template and selected biography addition are
separate source inputs; neither alone establishes the entire retail
starting inventory.

## Observer equipment helper

`EquipmentFixture::with_mesmer_biography` now adds the selected item to an
explicit equipment batch. `mesmer_mask_choice` requires exactly one
occurrence of187/188/189, rejecting absent, repeated and competing choices.
The method rejects an occupied slot5, a full68-item batch or an allocation
beyond the observer's instance-ID limit4096. Its new instance is one above
the maximum existing ID. The creation precedes every equip; its equip
precedes the final weapon-set selection, or ends an armor-only batch.
Every existing frame's bytes remain unchanged.

Flags0, no Guid, no upgrade items and initial zero colors are explicit
observer research choices. The caller must apply `with_creation_colors`
afterward for actual request415 row5. Eight focused equipment tests pass,
including all three mappings, rejection cases, both insertion orders,
original-byte preservation, boundary allocations,67-to68 items, and row5
recoloring with rejection of values above65535. These tests validate the
observer batch; they do not add runtime evidence for retail creation.

## Evidence

Executable SHA-256:
`99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`.
All archive reads checked the retained source hashes:

- 1282831: `f9af9a85b3de5bd09c1840d058bb61442e414cdfeb0f99720f8896e2d6ccba56`
- 1282832: `63bd50638f1386c9d92ecdecf9c9260e02500d1d9ac616e33883577033c387de`
- 1282841: `62ef0c04651578ca4cf6a4322fe4cb792bd865b614069d639858986b13594e39`
- 1282843: `ac91841287684a0a9262ec53b2e095498448276da6f0208d53421675024b75ea`
- 1282853: `33ca7ae0d8e93f4fbfbe966e3e639a80e9e7ba6dcef710819cbb86e231ee0603`

Private bundle:
`~/re/gw2/notes/209026-mesmer-biography-equipment-20261105.json`.
Private source-reading helpers: `mesmer-biography-hash-items-20261105.py`
and `mesmer-biography-final-check-20261105.py`. No raw archive content
has been added to the repository.

| REA evidence | Observation |
| --- | --- |
| `ev_755c93296ae9d55c8a4368cca6f347b670c6be9d053c1834ef2e6c7455aa4cc0` | Applicable Norn/Mesmer questions, answer values and relocation checks. |
| `ev_9706720f429b6225da7080cbcec3903a635dc5839d25f7fea434aefb552f500c` | Preview override, getter argument flow and selected-answer ordinal helper. |
| `ev_0defc4e08bbf014406ce5dbba0447dad0e8f971ff1b7f0268bfa84e84c4b0c58` | Question-name lookup, Mesmer switch branch and typed item-reference accessor. |
| `ev_893294b3cff5cb4981692299360ac129098e0fd98747363cd8288495d9618968` | Saved-byte selection, name formatting, level retry and item-tag checks. |
| `ev_db4c657186169060df6e25961332280c405a1785cc7e67b7dcbb3efbd16f70e3` | Exact literals, vtable entries and name-hash table. |
| `ev_5ef869d5a88f9f4a36d65928c0657d137e66ecd00e24bbcb9741d370c453f734` | Flags2 name transformation and final hash/string lookup. |
| `ev_ca385978d82dffd88441d1d992e358613373d1a49281e4256b7dcdd6645b5a45` | Namespace/leaf name mangling. |
| `ev_9e957da2b7963124e6fe096390eab016bb077e760823b301bb21527f4e6fe8a4` | SHA input, exact multiply/xor fold and output truncation. |
| `ev_a453a9ecc1fdb172a86bd71b5ae7d8904ef4ebcffe1171b4e1d9081177d99f29` | SHA-256 initializer and standard Base64 alphabet. |
| `ev_54eb9e7dbd19554b127ddce150aaab90d3f510dde86bfb2164808167244189c8` | Content-name hash routine. |
| `ev_6a97fa3eb499eeaaf592e8d892722a88582ba904cae261078e7f9225acc57328` | Unique matching names across32 containers and exact item/skin/prefix references. |
| `ev_7b93542104875d01706a14ed7ad0913ab9007b3f0915be50d5918c18053bba22` | Armor-category mapper code. |
| `ev_028f930e0126cb0ede2dc2321721307042dc43e8d91f9b46ee757983f4bb2689` | All three answer mappings, category3 branch, preview guards, prefix rows and archived row5 colors. |

The cited captures completed with exit0, retained complete process output,
and verified cleanup of their owned process groups. Earlier failed name
probes and timed-out archive scans remain in the private bundle as
investigation history; none supplies the positive mapping above.
