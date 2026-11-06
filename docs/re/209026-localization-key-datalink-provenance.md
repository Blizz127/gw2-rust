# Build209026: local localization keys originate in item DataLinks

The five local key-setter calls at `0x1413117b1..0x1413118a1` belong to
the **type2 item DataLink parser**. Their byte buffer is decoded from an
incoming Base64 UTF-16 string. One complete caller obtains that string
from the clipboard and recognizes the `[&` link prefix. This establishes
a concrete local import route for keys carried inside a link; it does not
recover a quest154 key table or make those keys derivable from a text ID.

This closes the outer-buffer gap in
[quest154 content](209026-quest154-content.md), without repeating the
[inline-key formatter trace](209026-quest154-inline-key-followup.md).
All source captures use frozen executable SHA256
`99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`.
No key was supplied, guessed or generated. No archive census, working
archive, live target, clipboard contents or server was accessed.

## Typed dispatch and decoded buffer

The five calls are inside complete parser `0x1413114e0..0x141311966`.
An exact direct E8/E9 query finds no caller because this entry is selected
through a table. The exact function pointer occurs at `0x1425f67b0`,
entry2 of table `0x1425f67a0`. The table's validated address reference is
`0x1413105fb` in complete dispatcher `0x141310580..0x14131069c`.

That dispatcher takes a string view with pointer +0 and count +8. It:

1. Rejects the empty string view and the one-word zero string.
2. Calls `0x140d97f90` with the UTF-16 input pointer and length sentinel
   `0xffffffff`, producing a temporary byte array.
3. Requires more than one decoded byte, then passes its current pointer
   by reference to complete leaf `0x141313d00`. The leaf consumes one
   byte and returns it if below17, otherwise sentinel17.
4. Calls `table[type](remaining_start, decoded_end)` at `0x14131060b`.
   Type2 therefore calls `0x1413114e0` with exact decoded-buffer bounds.
5. Releases the temporary buffer after the parser returns.

`0x140d97f90` sizes the output as three bytes per four source characters
and calls complete decoder `0x140d979f0..0x140d97b59`. Its named source is
`Arena/Services/Base64/Base64.cpp`, with assertion
`dstChars >= Base64DecodeSize(srcChars)`. The decoder reads UTF-16 units,
uses a byte alphabet table and combines four encoded values into up to
three output bytes. Thus this buffer is a decoded link payload, not an
archive record pointer or a decoded server748 message structure.

The parser constructs a link object with primary table `0x14223ac58`
and secondary table `0x14223ad20`. Its primary +`0x20` method
`0x141310110` forwards the retained definition to `0x1413cec80`, whose
source asserts `itemDef` in `Item/Const/ItemConst.cpp`. Nearby interface
assertions independently name `DataLink/Cli/DlCliInt.h`.

## Exactly which text IDs receive the optional keys

After the type byte, the item parser reads one raw byte and one Dword.
The Dword's low24 bits select an **item definition** through content
context +`0xe0`, virtual +`0x178`. Its high bits select optional fields.
The established content adapters map +`0x178` to type35 and +`0x240` to
type66 skin definitions; see
[item-content namespaces](209026-item-equipment-delivery.md#identity-and-dispatch).

Bit31 adds a skin key resolved through +`0x240`, with item/skin subtype
compatibility checks. Bits30 and29 add two further type35 item keys,
called A and B below. The parser uses kind23 bodies of those optional
items for one additional text ID. None of these lookups is a quest,
goal or step-definition lookup.

| Flag | Setter call | Text ID derived from resolved content |
| --- | --- | --- |
| 28 | `0x1413117b1` | Main item +`0x80` |
| 27 | `0x1413117f2` | Main item +`0x84` |
| 26 | `0x141311829` | Kind23 body +8 of A when available, otherwise B when available |
| 25 | `0x141311865` | Optional item A +`0x80`, when A resolved |
| 24 | `0x1413118a1` | Optional item B +`0x80`, when B resolved |

Each enabled branch requires a nonzero derived text ID, obtains the next
bounded eight payload bytes, and calls `0x1410dc7a0(text_id, key)` only
when that supplied Qword is nonzero. The Qword comes from the decoded
input; the item definitions supply the IDs, not a computed key. The
consumer requires the final cursor to equal the buffer end before
constructing a successful link object. Dictionary insertions occur before
that final check, so a later parse failure is not proof that no earlier
key was imported.

This table deliberately does not assign new name/description/prefix enum
labels to the content fields. The receiver and exact offsets establish
the relevant boundary without those labels.

## A concrete outer origin: pasted link text

The exact direct query for `0x141310580` finds six instruction-validated
call sites. Two complete bodies, `0x1411e4260` and `0x1411e5500`, check
the first UTF-16 words for `[` and `&`, then pass the suffix starting four
bytes later to the Base64 dispatcher. The latter receives its text through
an incoming object; that object's upstream origin is not assigned here.

The former has a stronger, demonstrated origin: it allocates a text buffer
and calls `0x140da23c0(maxChars=200, buffer)` before inspecting the prefix.
That callee's source is
`Arena/Services/Clipboard/Windows/Clipboard.cpp`; its inspected entry
asserts `maxChars`, `text`, and `!text->Count()` before the clipboard API
calls. No clipboard API was executed by this investigation. This establishes
that local clipboard text can reach the item-link parser and optional
dictionary insertion, conditional on a valid link and its supplied keys.

Other direct sites occur at `0x140439559`, `0x1411e86c3`,
`0x1411e99f5` and `0x1414a108d`. Their presence is not labeled network
provenance without their own upstream binding. The local parser may be
reached from more than one source; the clipboard route does not prove all
input originates there.

## Quest154 implication and remaining boundary

This route does **not** supply recovered keys for text IDs239267 or
34076..34082. It accepts caller-provided item-link keys associated with
text IDs derived from the linked item definitions. No inspected edge
connects quest154's quest/goal/step records to this type2 payload, and no
matching authored item or valid keyed link was identified. A coincidental
text-ID match across other content is not ruled out by this bounded slice;
no broad search was performed to claim otherwise.

The usable correction to the earlier uncertainty is that the five local
setter callers are an import path, not evidence of an embedded default
quest-key source. Their existence cannot supply missing quest keys from
the IDs alone. The already established dictionary748 and formatted inline
key mechanisms remain valid, but each still requires an exact, independently
proven key source. Authored quest diagnostic strings do not carry one on
the inspected formatter path. The blank-objective dependency is therefore
still unresolved, with this candidate source now concretely classified.

## Evidence

Private twelve-capture bundle:
`/home/blizz/re/gw2/notes/209026-localization-key-datalink-provenance-20261106.json`,
SHA256 `2648ac93c19affdc79c5d4f34828f20baac11e9d3b74bfb7c665417f7d5631c9`.
All captures exited0, report nontruncated output and verified owned-process
cleanup. Some include unrelated unreadable `/proc` entries, which are not
claimed as a machine-wide process audit. The initial `.pdata` helper could
not identify two leaf functions; their complete aligned bytes were captured
separately. Cut adjacent instructions are not evidence for those bodies.

| Evidence | Scope |
| --- | --- |
| `ev_c7aa6bd4ff3914e82054b9364625b595e3a3f80cf5a8034eb58ae51b802221e3` | Complete item parser, exact input/key/ID flows and link construction |
| `ev_f1525671887dba963bec65ada40e82b7e7aa974cb1afc86feddf923321e51f26`, `ev_4a25befc425bb505472f6db0abf8d39cceac5225eded43dab2387dd9a038e17f` | Exact pointer/table binding and validated table-base reference |
| `ev_c1ebcccc74ba84db05941bcbd94863b9d57a4542ab32a6119826c25e2c2605f7`, `ev_cb1e23e8a7d62dbbe8ea74d122eb7c5e9e866e4b1f220a5791a3db5fa88a5143` | Complete dispatcher and complete type-byte leaf |
| `ev_d42aa3de0d4adf4b8e734cc4eaaa610fbbb30ffe3811eee291d6ca8d23376470`, `ev_a2251f2c0c2677aebcf707a043178ef2660d6d40b14b15caaa48daf040387617` | Base64 wrapper/complete decoder and complete two prefix-checking callers |
| `ev_e02f9a4f2066fd7771a9b0cf10500f41761943974d5cccac7a709ffe99ba81dd`, `ev_31427f6f759252f4459c75665fd4828ce3463153020da91ef668db8c641061b5` | Named DataLink/item interface and clipboard source entry |
| `ev_3c264b827dcbfd852c7b5927d799def5f041580ef4ffb912387160a8eda85e86` | Six exact direct dispatcher callers; indirect callers remain outside coverage |
