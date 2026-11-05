# Build 209026: bounded Great Hunt species-link audit

Static REA archive/executable analysis, 2026-11-05, following the
[NPC character foundation](209026-npc-character-foundation.md).
**No exact tutorial species key was established by this slice.**
The searched graph exposes quest progression, sectors, layout and POIs;
none of the bounded links below selects an NPC. No NPC, placement,
stats, model or component defaults were chosen, and no code changed.

## Incoming references across the frozen base containers

The scan visited all 32 base content files, **1282830–1282861**,
checking each decoded container against its retained frozen SHA-256.
It enumerated the declared local/external relocation tables and matched
their exact destination container and content offset against:

| Target | Container:content offset |
| --- | --- |
| Map 379 | 1282833:3863896 |
| Quest 154 | 1282830:4145560 |
| Sector 201 | 1282846:3289648 |
| Sector 200 | 1282846:3233008 |
| Sector 202 | 1282846:3179240 |

There are **34 matching relocated pointer fields**: five to map 379,
one to quest 154, and 28 to the three sectors. None has a type-67
nearest indexed owner. The same container indexes contain 23,653
type-67 entries; the absence is not caused by omitting species records
from the scanned containers.

The five exact map references are:

| Source record | Source type/key | Relative field |
| --- | --- | --- |
| 1282830:4146200 | Inline goal, type 341 | +0x28 |
| 1282832:278080 | Type-150 indirection, declared value type 45 | +0x38 |
| 1282848:3333416 | Type 52, key 970 | +0x38 |
| 1282848:3361504 | Type 52, key 971 | +0x38 |
| 1282854:4424888 | Type-277 map layout definition | +0x48 |

Quest 154's incoming reference is type 68/key 2 at
1282830:4127192 +0x58. The sector references repeat the
[entry-geography inventory](209026-norn-entry-geography.md): 22
inline type-343 steps, three type-279 records, and three fields in
an indexed type-317 region.

The nearest preceding index record is an attribution aid, not proof
of the semantic owner of every trailing array. These results concern
declared relocated pointers to five exact records. They do not exclude
numeric IDs, GUIDs, deeper references, secondary content or separately
provided encounter/spawn state.

## Exact quest records and one further pointer hop

The parallel quest check covered nine roots: quest 154's fixed
0xd0-byte definition, associated type 68/key 2 through the next index
boundary, goal 1's 0x70-byte definition, and its six 0x60-byte steps.

Those intervals contain **22 relocated fields and 17 unique direct
targets**. Each direct target was then scanned through its next indexed
record boundary, a conservative bound that can include trailing arrays
or metadata. This produced **47 unique second-hop targets**.
Every target was classified against its container's index; neither an
exact target nor its containing indexed record was type 67.

Useful exact associated-record references are:

| Type 68/key 2 field | Target |
| --- | --- |
| +0x38 | 1282840:4865104, type 389 |
| +0x58 | 1282830:4145560, quest 154 |
| +0x68 | 1282833:4183760, type 53/key 3863 |
| +0x80 | 1282835:4241824, type 68/key 9 |
| +0xc8 | 1282857:6532048, type 74/key 2 |

Quest +0x68's array targets type-35 item keys
32130/32138/32135/32141/32136/32129/32132.
Quest +0x78's array targets type-57 quests 159
(1282830:4140272), 162 (1282830:4127496), and 155
(1282830:4157648). The step references remain the known sectors.
These are precise follow-up addresses, not NPC identities.

Private quest enumeration and complete Evidence:
`~/re/gw2/notes/209026-quest154-species-bounded-20261105.json`,
SHA-256
`779886683ddb6f8b524e904de4f3487b0cf4009bff2f265a46a2f934dc0e5a52`.
Its limitation explicitly excludes nonrelocated numeric IDs and deeper
graph paths.

## Map-linked records 970/971 are resurrection-shrine POIs

Content context vtable `0x14220e160`, slot +0x1e0, resolves to
`0x1412e1da0`, which passes numeric type **52** to lookup +0x70.
At `0x14126e599`, a caller obtains this content service, invokes
+0x1e0, and asserts the returned `poiDef`.
This binds the record type to POI definitions.

The branch at `0x1414433bd` compares definition +0x44 with **1**;
its failing assertion is
`pointOfInterestDef->GetType() == Content::POINT_OF_INTEREST_TYPE_RESURRECTION_SHRINE`.
Both map-linked records have +0x44 = 1 and +0x38 pointing to map 379:

| POI key | Record | Referenced +0x48 object |
| ---: | --- | --- |
| 970 | 1282848:3333416 | 1282848:3333504 |
| 971 | 1282848:3361504 | 1282848:3361592 |

These are not species definitions. An all-32-container incoming
relocation search found **zero pointers to either POI record**.
Tutorial type-283 layout **1282852:4461360** has null/zero
collections at +0x40/count +0x48 and +0x70/count +0x78, which
the inspected layout consumer uses for POI-bearing records.
Its populated collections are +0x50 (52 type-286 records) and
+0x80 (three type-289 sector polygons).

No authored POI coordinates were established. The POI definitions'
small +0x48 objects contain no identified location, and their numeric
bytes are not being reinterpreted as a spawn. These records do not
replace the explicitly diagnostic placement already documented in the
[sector polygon note](209026-norn-sector-polygons.md).

## Remaining source boundary and useful next step

A separate header-only inventory of shared map file **187611**
confirmed its frozen decoded SHA-256:
`2dc9c92929362a10cdb22b5350957eac764ccc1d53c8a97b01c5798223211bbc`.
It contains 20 chunks, including `msn\0` version 2 with a 200-byte
payload. Chunk names and sizes alone do not establish whether any
unparsed chunk encodes NPC references. No claim of archive-wide
absence follows from this inventory.

The official community wiki provides useful external name locators:
Thora Griffonbane, Ido the Tanner, Eir Stegalkin, Vicious Dire Boar,
Owl Griffon Sire, Alpine Bull Minotaur, Ice Wurm Hatchling and Issormir.
These names are supplementary current documentation, not a join to a
frozen type-67 key. [The Great Hunt](https://wiki.guildwars2.com/wiki/The_Great_Hunt)

The next bounded source step is the **type-67 name consumer/schema**:
determine whether its archived name retains an inline text key, then
use the external names to locate candidate species. Each match must
retain the actual species key, content address, model references and
flags. Only then can the NPC model/component audit be specialized.
Retail placement and encounter ownership remain separate questions.

## Evidence

Frozen executable SHA-256:
`99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`.

Private bundle:
`~/re/gw2/notes/209026-great-hunt-species-link-audit-1.json`.
It preserves the 32 source hashes, exact incoming-reference inventory,
13 successful REA captures, command manifests and limitations.
This is static source inspection; the REA process runner's authority
label does not imply game execution or observed NPC behavior.

| REA evidence | Observation |
| --- | --- |
| `ev_5e8263fb4f2ddf93ccbcbb83c44e4e64aaddfae86465da85b42266da07285abf` | Incoming references, base containers 0–7. |
| `ev_f458d0f87425561a79abb6a1ae548bf476f6e64057c82b50a0717f2cac4ec713` | Containers 8–15. |
| `ev_c017cd858badb4503682d5f56f47eb32438a867c79839b79e735fbb284b8c828` | Containers 16–23; POI references. |
| `ev_bea30bc2a902ff21ae69d28863ae5d7cfcd7d2b4478953cd4c16161e18079506` | Containers 24–31; layout/sector references. |
| `ev_4fcf67b73b50435fb2d70f5bc412a16406538988810e4f2520d2877c17301df3` | Quest agent: exact nine roots and direct targets. |
| `ev_184478562931158ba57bca75a1ffe5c357cebed1b55f78d4d5ddc5f1cf67ff4e` | Quest agent: further relocation enumeration. |
| `ev_15d3b643ed1ad700312caa50c51f4da7e01ef10f318f09b11faac51e6ec8f034` | Quest agent: complete second-hop target classification. |
| `ev_863412689ddc838dc0a8bacee4dce7c704a5905c7eea06da6984723b77222e9e` | Exact POI and map-indirection records. |
| `ev_99edeb938f450330280c5fe7a73343afe527a141c60af76795abd679c2ae222e` | Numeric type-52 adapter. |
| `ev_a3200dc2ca885971cba6e1ca3df9b7ac6656c6c14c3c9adf5cfa07503578dac3` | Content vtable +0x1e0 and POI assertions. |
| `ev_e0afbf03928ebd47aec4bb4ca902a0e5959fbd240a592e3a8db918602d66f3e8` | Mapped assertion addresses and caller candidates. |
| `ev_9d322b6b6e8f6fdee845110e924971e0dba04901653e166f0e52a82c3c7677bc` | Named POI lookup and resurrection-shrine subtype branch. |
| `ev_da0772567c37477efa33670993779ac1a1e13827c4aa1935feebf22091b27a3e` | No incoming POI pointers, containers 0–15. |
| `ev_3f9074a1a8b34f5f2ab16c7d1cbdf1510031ae2e3c261430bf63248b0cb3efdb` | No incoming POI pointers, containers 16–31. |
| `ev_ed8200ac4a13f8f41a088b885f630666de4a838059cc2f16c6c7b67dc0167c80` | Tutorial layout collection counts and child types. |
| `ev_ae415310c93a98b798cc0896146c84d1df035fe9e87121b5c716319dd8445d77` | Shared map archive chunk inventory only. |
