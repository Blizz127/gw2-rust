# Build 209026: Norn content selection and navigation limits

Read-only REA process captures, 2026-11-05, following the
[Norn-first map audit](209026-human-norn-starter-maps.md). This establishes
manifest names and a content-selection path. It establishes no retail spawn
coordinate, NPC population, tutorial progression or area completion.

## Exact Norn manifest names

Both map **379** (The Great Hunt) and map **28** (Wayfarer Foothills) require
**`MapAlpineSnowline`** and **`RegionAlpine`**. The names use the already traced
prefix consumers `0x1413fe2a0` and `0x1413fe340`; see
[patcher](209026-patcher.md). A fresh extraction verified the type-45 keys,
container hashes, string-fixup membership and local string-table bounds.

| Map key | Container | Record offset | +0xb0 string/index | +0x110 string/index | +0xd8 string/index |
| ---: | ---: | ---: | --- | --- | --- |
| 28 | 1282830 | 3731704 | `AlpineSnowline` / 9557 | `Alpine` / 9317 | `7bzex` / 9558 |
| 379 | 1282833 | 3863896 | `AlpineSnowline` / 10759 | `Alpine` / 10713 | `uF5kn` / 11097 |

String references use **each record container's own** Main +0x70 table.
This differs from the shared base-content file-reference table. Executable
`0x1412e789e..0x1412e78a9` reads that container's +0x74 array pointer when
applying string fixups. An initial exploratory read incorrectly used the
first container's string table for map 379; its `sdy/c.EIxX7` result is
superseded and must not be used.

## Map content namespace selection

Loader `0x140951847..0x140951865` passes mapDef **+0xd8** to the content
service's virtual +0x18. Vtable `0x14220e160` resolves that slot to the
`0x1412e2fc0` thunk, then `0x1412e7580`, which initializes a secondary content
backend through `0x1412e7680`.

Namespace selector **`0x1412e6ab0`** searches for a matching name with domain
**7**. The matching branch at +0xb3 sets its namespace bit, repeatedly adds
namespaces whose parent is selected, then adds the original namespace's
immediate parent and grandparent. It does not subsequently expand descendants
of those ancestors. Record registration in `0x1412e78d0..0x1412e7903` and
`0x1412e7930..0x1412e7963` checks each index entry's namespace against the
selection bitmap.

The selector crosses the first PE unwind fragment boundary at
`0x1412e6b65`; stopping at that boundary would miss the selection logic.

| Map key | Domain-7 namespace | Namespace index | Parent / grandparent | Selected namespaces | Selected base records |
| ---: | --- | ---: | --- | ---: | ---: |
| 28 | `7bzex` | 26971 | 26551 / 26552 | 11 | 132 |
| 379 | `uF5kn` | 31145 | 31146 / 26552 | 6 | 40 |

A hash-checked census of all 32 frozen base containers applied this static
selection rule. Map 28 has type counts 41:2, 106:56, 108:12, 159:1, 174:2,
176:2, 257:56 and 268:1. Map 379 has 41:2, 106:19 and 257:19.
Neither selection contains type **57**, **306** or **307**. These are selected
base records, not a claim about every record visible at runtime, external
content loads, or client/server gameplay capabilities.

## Navigation fields are not spawn semantics

The previously established [navigation population](209026-quest-navigation-guard.md)
uses types 306/307 and resolves endpoint definitions through `0x1413f60f0`.
Constructor `0x1413fc9c0` stores a node definition at object +0x68 and installs
vtable `0x1422db0d0`. Its inspected accessors return:

- `0x14067a2a0`: the Dword at definition **+0x28**.
- `0x1413f66b0`: a pointer to definition **+0x2c**.
- `0x1413f66c0`: the Dword at definition **+0x38**.

Across 11,813 base type-307 records, +0x28 has only values 0:1,217, 1:7,817,
2:580 and 3:2,199. It is therefore not the Norn map key. Interpreting +0x2c
as three floats is useful for investigation, but no inspected caller selects
that vector as a starting spawn. No coordinate should be made a retail
default on this evidence.

The next source step is to trace the primary/global content backend's
type-306/307 visibility at population helper **0x1413f66e0**, then follow
node accessors' callers and character/map-entry position consumers. The
map +0xd8 namespace selection alone supplies neither a Norn navigation
graph nor a spawn choice. Runtime diagnostic placement must remain explicit.

## Evidence

The frozen executable SHA-256 is
`99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`.
Archive container hashes are in the linked map audit and the retained census.
All final captures exited 0 with verified owned-process cleanup; unrelated
inaccessible `/proc` entries were left untouched. No game client was started,
no runtime state was observed and no server behavior was changed by this audit.

Private manifest evidence:
`~/re/gw2/notes/209026-norn-manifest-names-20261105.json`.
Navigation evidence bundle:
`~/re/gw2/notes/209026-norn-navigation-evidence-20261105.json`.
Selected namespaces, record identities and all 32 source hashes:
`~/re/gw2/notes/209026-norn-navigation-selection-20261105.json`.

| REA evidence | Scope |
| --- | --- |
| `ev_5adf1bf815a336a6502b7896bd649a63f89849b83ee75f417b9a9dd41caeb612` | Both Norn map/region/content names, local strings and record identities. |
| `ev_7b2a5005cebc7ed93d30a36375a6c048bcd5ba63cfa66c4b6e655f9b0ac78141` | Map-loader +0xd8 call. |
| `ev_e854888ebd901e3a66b1adf4bed10d3eb1dde625e426880b146fc823e0e813c2` | Content-service vtable target. |
| `ev_7f22e516acee4dddf37463fdb7283cee4d4d9339f32876c5bd3442d36ac8de64` | Secondary backend initialization. |
| `ev_126f182c7304897e530d80ef1fb5b0b1704ac8ef4c63bea08c4fa6207c4001fd` | Local string fixups and namespace-gated registration. |
| `ev_f8e0185d4dc4e88b22be8ef30d16dbce44d12cc7870b0e0e66b3babbf0ab7132` | Complete selector continuation and bounded node accessors/constructor. |
| `ev_6aea7b6a5c5d7a0401ed72cbf1eea283beec0f981dc7648be2f634bb6ed67507` | All-base node +0x28 value census. |
| `ev_209aaa04ec1659905f78163e137feac13a922295c5683b6f25a9dd1697ae6f9e` | All-base census using the traced namespace selection. |

REA's process artifact identity identifies the helper interpreter. Input hash
assertions bind the selected archive/executable reads; imported helper source
is not automatically bound by that interpreter identity. The notes preserve
the scenario scripts and captured output for review.
