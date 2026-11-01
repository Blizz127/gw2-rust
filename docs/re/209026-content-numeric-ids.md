# Build 209026: numeric content IDs

Static REA analysis, 2026-11-01, of the frozen executable identified in
[client-build.md](../client-build.md), plus all 32 base content containers.
No client replay occurred. This corrects the earlier use of “key” for the
wire values in [specialization messages](209026-specialization-messages.md).

## Two separate indexes

Content-context vtable `0x14220e160` exposes two numeric lookups:

| Virtual | Adapter / lookup | Backend table | Record field selected by metadata |
| --- | --- | --- | --- |
| +0x58 | `0x1412e1b30` / `0x1412e6820` | +0xd0 | TypeInfo +4: `uid_offset` |
| +0x70 | `0x1412e1be0` / `0x1412e68e0` | +0xb8 | TypeInfo +8: `key_offset` |

Index builder `0x1412e8060..0x1412e840c` reads 17-byte TypeInfo records.
Its +0 member supplies the GUID index at backend +0xa0. For numeric
indexes, metadata offset `0xffffffff` omits the field; a zero record value
is skipped. Both indexes combine `(type << 22) | (value & 0x3fffff)`.
The lookups search up to two backends and resolve packed handles through
`0x1412e6df0`: high eight bits select the container, low 24 bits the
16-byte entry whose +4 offset addresses the container's content bytes.
The builder and lookups call `0x1412e9680` before masking; callers do not
branch on its return. Arbitrary out-of-range values are not validated here.

For the six types below, UID is at record +0x14 and data key at +0x28.
They are distinct namespaces, even when their ranges coincide. Skipping
zero in this builder is observed; safe gameplay clearing still depends
on the consuming handler and listeners.

## Message mapping

| Messages | Content type | Lookup | Wire value |
| --- | --- | --- | --- |
| 640, specialization part of 642 | 386 | +0x58 | UID at +0x14 |
| 641, trait part of 642 | 77 | +0x58 | UID at +0x14 |
| 612 | 65 | +0x238 adapter `0x1412e1eb0` → +0x70 | Data key at +0x28 |
| 605, 609, 613 | 64 | +0x230 adapter `0x1412e1e90` → +0x70 | Data key at +0x28 |
| 565 | 353 | +0x70 | Data key at +0x28 |
| 247 | 23 | +0x70 | Data key at +0x28, carried as a Word |

Skill availability additionally indexes its bitset with the resolved
skill's data key. Existing palette/availability codec names remain valid.
Reward-track and wardrobe raw ID names also remain valid. The specialization
codec must use `specialization_uid(s)` and `trait_uid(s)`, while catalog
readers must expose UID separately from their existing data-key fields.
Wire encoding is unchanged. No earlier specialization fixture was replayed.

## Complete base census

Private helper `inspect-numeric-content-ids.py` checked containers
1282830–1282861 in four bounded groups, retaining source hashes and record
locations. Both fields are nonzero and unique within each type below.

| Type | Records | UID range | Data-key range | Records with equal values |
| --- | ---: | --- | --- | ---: |
| 23 | 155 | 2–173 | 1–155 | 0 |
| 64 | 81,049 | 1–83,197 | 1–81,357 | 1 |
| 65 | 7,769 | 1–8,092 | 1–7,777 | 1 |
| 77 | 2,425 | 1–2,425 | 1–2,445 | 1 |
| 353 | 529 | 1–530 | 1–529 | 1 |
| 386 | 81 | 1–81 | 1–81 | 9 |

The existing [profession membership census](209026-specialization-content.md)
uses data keys and remains a valid description of pointer membership.
It does not directly supply specialization wire UIDs. In its Guardian
membership order, the mapping is:

| Data key | Wire UID |
| ---: | ---: |
| 42 | 57 |
| 16 | 19 |
| 13 | 15 |
| 49 | 69 |
| 46 | 64 |
| 27 | 36 |
| 62 | 76 |
| 65 | 32 |
| 81 | 77 |

Specialization data key 13 (UID 15) has tier-one trait data keys
588/581/633 with UIDs 2373/967/153; tier two 580/584/1684 maps to
604/1706/155; tier three 585/586/589 maps to 904/1215/1505.
The previously identified Guardian GUID target, trait data key 2200,
has UID 1036. These mappings establish identity, not unlocks or eligibility.

## Implementation and checks

The specialization codec now names its raw fields `specialization_uid(s)`
and `trait_uid(s)`. The bounded specialization/trait catalog views expose
both `uid` and `key`. The checker rejects zero/duplicate UIDs and prints
Guardian membership as `(data key, wire UID)` pairs.

Three specialization framing tests and three profession-content tests pass;
the reader fixture uses deliberately unequal UID/key values. The library
and `gw2-profession-check` builds pass. The checker ran through REA against
all 32 frozen containers and reproduced the Guardian table above, with
81 specialization UIDs and 2,425 trait UIDs and all previous membership,
tier and progress-reference checks passing. This is an offline content
check, not runtime validation.

## Evidence

Private bundle: `~/re/gw2/notes/209026-content-numeric-ids-1.json`.
Private census: `numeric-content-ids-209026-{0-8,8-16,16-24,24-32}.json`
in the same directory. All analysis used REA; no account data was involved.

| REA evidence | Observation |
| --- | --- |
| `ev_1270c3fcf4015daee63d67f02600054da0c41d9b12daeabece03e0bbc576cd45` | UID lookup and packed-handle resolver. |
| `ev_5cb5d88e83a6e8fdff8b8d3f6c83658af34ab1a64d983cf3df0ffcc66b7f9704` | Complete index builder, field offsets and zero skipping. |
| `ev_ff20a6f658cc202493621f6c1cac4eeb946b6716d1071ea73cf03cb6fb3a312f` | Containers 0–7. |
| `ev_d423304ad536951c7c96e1e0ab33e4c9bb84349a18d5e61b2f003cbe1bf31fa3` | Containers 8–15. |
| `ev_7aa1251cd2b99fab543599e5cb11fd65f3df2b4989ca1c1e811c52e11261ab7c` | Containers 16–23; skill/palette adapters select +0x70. |
| `ev_6855b69967542ba4e8c7f2a55c398566e75e091be8ef1f6c485100bebc127a7b` | Containers 24–31. |
| `ev_5686d8eade4e97e7e9e7a26e1f380e14094e4103569a6b65b9898879634aeef8` | Aggregate ranges, uniqueness and Guardian mappings. |
| `ev_8c529ba5cbe3c8cab301db5c01bb1913ff3a34cadc6fce7ebbfdfbf28ad7e2ab` | Rust reader/checker smoke across all 32 containers. |
