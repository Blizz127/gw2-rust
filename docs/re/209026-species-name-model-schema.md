# Build 209026: species name and model schema

Static REA follow-up, 2026-11-05, to the
[NPC foundation](209026-npc-character-foundation.md) and
[bounded Great Hunt link audit](209026-great-hunt-species-link-audit.md).
The ordinary species-name path reads **a Dword text ID at type-67
record +0x194**. It does not pass an adjacent inline text key.
No sampled species has been matched to a tutorial NPC name.
No gameplay code, client state or encounter fixture was changed.

## Typed name consumer

Character primary vtable `0x142160be0` establishes:

| Slot | Target | Observed role |
| --- | --- | --- |
| +0x1c0 | `0x1411d71f0` | Calls +0x1c8, then applies additional monster formatting through `0x1411c37f0`. |
| +0x1c8 | `0x1411d7270` | Selects the underlying character display text. |
| +0x300 | `0x1411d87b0` | Returns the species pointer at character +0xa8. |

The named UI consumer `0x1405bf690` checks
`StrCmp(newName, m_character->GetPlayer()->GetName()) != 0`,
then obtains character virtual +0x1c0 and passes the result to
`0x141048020`. This ties the getter chain to the character
name-change display path. It does not identify every other name widget.

The complete +0x1c8 getter, `0x1411d7270..0x1411d7493`,
has separate custom-name, player, transformation and contextual branches.
Its ordinary nonplayer fallback:

1. Uses coreStats when present for a formatting selector, otherwise
   selector 2. A null coreStats pointer is guarded at this point.
2. Considers an existing character +0x140 text override and separate
   contextual display alternatives.
3. At `0x1411d7431`, calls species getter +0x300.
4. At `0x1411d7447`, reads **DWORD [species +0x194]**.
5. Copies that Dword into a stack temporary and passes its address to
   `0x1412e38e0`.

The player branch instead resolves the associated player and invokes
its virtual +0x88. It does not interpret the species name field as a
literal player name.

The two following wrappers are complete:

```text
0x1412e38e0: mov ecx, DWORD PTR [rcx]
0x1412e38e2: jmp 0x1410dbdd0
0x1410dbdd0: xor edx, edx
0x1410dbdd2: jmp 0x1410dbde0
```

They select the same no-argument text-ID encoding path established in
the [quest inline-key trace](209026-quest154-inline-key-followup.md).
The original species pointer and its adjacent fields are not passed to
this formatter. Thus this branch supplies no species-adjacent inline
key to the later text parser.

This is a boundary on this consumer, not proof that no other archive
or initialization path can supply dictionary keys. The custom-name
and transformation branches also remain distinct. A whole-client
claim that species names cannot be recovered would exceed this trace.

## Bounded archive samples

Only three type-67 records from already cached base container
**1282830** were sampled. Its decoded SHA-256 is
`e5d11e4fae8627d3efced749e7ab599b26abac59e502716aaf7d63ef509bb81f`.
All sampled intervals through +0x1a0 fit before their next indexed
record. These are schema examples, not chosen tutorial candidates.

| Species key | Content offset | Metadata name | +0x194 name text ID | Raw +0x190 |
| ---: | ---: | --- | ---: | ---: |
| 17992 | 228920 | `7r+rf.GIHgJ` | 727104 | 756116 |
| 18141 | 229800 | `7r+rf.6S4lO` | 783299 | 781163 |
| 18355 | 230448 | `7r+rf.4xEsW` | 781006 | 781852 |

For all three, +0x198 and +0x19c are zero. There are no declared
local, external or string fixups in +0x180 through +0x19f.
The raw +0x190 field has not been given a semantic name by this
investigation; it is excluded from the traced +0x194 call regardless
of its stored value.

Record +0x18/+0x20 point to metadata with a classified string fixup.
Resolving that string through this container's own Main +0x70 table
gives the opaque names above. Reading the metadata's table index
directly as UTF-16 does not yield a display name. None of these
metadata names establishes a match to Thora, Issormir or another
externally documented NPC.

The original three-record structural samples are retained in
`~/re/gw2/notes/209026-species-layout-samples-20261105.json`.
The separate schema bundle below independently confirms the +0x194
fields and adjacent zero values.

## Exact model references and copied flags

The [ordinary model trace](209026-npc-character-foundation.md)
establishes species +0xf0 as the direct model-loading branch's
file input, with +0x120 as another input.

All three sample records declare **file fixups**, not content-pointer
relocations, at +0xe0 and +0xf0. The established
[content file-reference decoder](209026-content-registry.md)
uses base container zero's file table, at Main +0x1c:

| Field | File-table index | Encoded words | Decoded file ID | Observed archive header |
| --- | ---: | --- | ---: | --- |
| +0xe0 | 5555 | 19185, 256, 0 | 18930 | PF / AMSP |
| +0xf0 | 5554 | 19198, 256, 0 | 18943 | PF / MODL |

File 18930 is 27,996 bytes, SHA-256
`571ebbacd1d44695abaa1868ef85d34e9a8ada8ffa0aea190ee73c648dce29f6`.
File 18943 is 137,760 bytes, SHA-256
`1bbbd07584481d2d0e637078440f1909b050fc9a0067a1022e7b9755f3cc1e0a`.
Both were extracted and hashed through REA. The +0xe0 field's gameplay
role is not assigned here. The sample +0x120 values are all zero.
The MODL header proves an available model artifact, not its rendered
appearance or tutorial identity.

Constructor helper `0x14122bbc0`, called with the species pointer
and character +0x260, copies **species +0x100 directly to character
+0x264**. This resolves the source of the fallback flag word used by
the previously traced target-selection rejection method
`0x1411d9a30`.

The +0x100 values are 0x00800000 for species 17992 and 0x20800000
for 18141/18355. All three clear mask 0x08, but that passes only
one observed flag test. It does not prove target admission, model
readiness or valid component state. The other target-context,
transformation, distance, view and health gates still apply.

## Practical next lookup

The typed catalog fields are now known: species key at +0x28,
name text ID at +0x194, model file fixup at +0xf0, additional
model input +0x120, and the selection fallback flag source +0x100.
A subsequent catalog can retain these without assigning English names.

To match the wiki's tutorial names, first obtain a supported resolution
of these text IDs or another independently verified species identifier.
This slice found no inline-key shortcut in the ordinary name path and
does not choose a key based on file sharing, record proximity, raw
numeric coincidence or an opaque metadata name. Retail encounter
membership and placement remain separate from a successful model match.

## Evidence

Frozen executable SHA-256:
`99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`.

Private bundle:
`~/re/gw2/notes/209026-species-name-model-schema-1.json`
contains six complete successful REA captures and their commands,
output, identity checks and limitations. The three additional sample
captures reside in the linked private sample bundle. No live game
behavior was observed by this analysis.

| REA evidence | Observation |
| --- | --- |
| `ev_dd6df5d911b8476a52a0e43d60bf08d57578f811409fd13d7f922f97ec89ba0f` | Name/species table entries and named UI assertion. |
| `ev_5089bf262251cd5dbb9b3b6190015fc545d951a2ed1fd9bab54dc255c3b1731d` | Complete UI name-change consumer. |
| `ev_8f0cc4f68e85cf3c58829d52c9f412623e439689444976536d2af2cc59cd067c` | Complete +0x1c0/+0x1c8 getters and Dword-only text wrappers. |
| `ev_6dd368e648e8d911297398eb6ac55cbb808a3bd5c69dcdde42376e6d3fd714a7` | Three archived +0x194 fields, adjacent values and absence of fixups. |
| `ev_c069f2e654c1f46b12ccd02a3e6a7515c216c61daa1d13d8bea3a4c23897ef50` | Species +0x100 to character +0x264 copy. |
| `ev_fa1a70cffed09cd90860fbb2bb613d1fa48ee80c6fe126a7e68f3054462d43d4` | Exact file fixups, decoded IDs, model inputs, flag words and extracted artifact hashes. |
| `ev_c8280308877c6d7a28661cc64d53f2c7a2577a11b84130afa819e7155fe160af` | Parallel sample: three record prefixes and relocation classifications. |
| `ev_d7288b33462e8cc6b283c2173b0987047a49391423ebc7816332301ab0b7e935` | Parallel sample: metadata and array pointee bytes. |
| `ev_7f95058c7630407254eb22f7b03f7d7af28fe4028c074819044a997500e40403` | Parallel sample: correct string-fixup resolution to opaque metadata names. |
