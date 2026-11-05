# Build 209026: item creation and equipment delivery

2026-11-05, hash-bound static REA inspection of the frozen executable and
archive. The client provides a concrete **create instance → equip instance →
select weapon set** path. This establishes framing and consumers, not retail
initial item state or successful combat. The implementation in
`crates/gw2-proto/src/item.rs` exposes each step independently and supplies
no default loadout.

## Identity and dispatch

Item-service getter `0x1413c71f0` obtains TLS context +0x178. Its primary
vtable is `0x14225ce50`; virtual +0x10 (`0x1413c6f20`) resolves a runtime
instance ID through the service's pointer array at +0x30, bounded by +0x3c.
It asserts that the ID is nonzero. This registry is distinct from content
data-key lookup and from archive file IDs.

Factory `0x1413c7720(context, instanceId, itemDataKey)` resolves the type-35
item definition, checks for an existing instance, grows the registry, and
constructs the subtype selected by item definition +0x2c. Kind0 constructs
armor (`0x1413c87e0`); kind24 constructs a weapon (`0x1413cb4f0`). The
result is stored under the supplied instance ID. A duplicate ID reaches
collision diagnostics; creating another instance is not a replace operation.

The content adapters establish the namespaces used below:

| Adapter | Content context virtual | Type | Meaning |
| --- | --- | ---: | --- |
| `0x1412e1ca0` | +0x178 | 35 | Item definition data key |
| `0x1412e1cb0` | +0x180 | 37 | Prefix definition data key |
| `0x1412e1ec0` | +0x240 | 66 | Skin definition data key |

Each adapter forwards to the content service's numeric-key lookup. An
upgrade item key is therefore not an item-service instance ID.

Armor960 and weapon971 belong to registration table `0x142260af0`,
registered in protocol0/group0 by `0x14021d7e0`. The extracted schema is
already present in `data/protocol/209026/server-tables.xml`. Server959
also reaches the factory but invokes only the generic initializer; it does
not supply the weapon or armor subtype fields documented here.

## Creation messages

The fields below are in wire order. Dword uses the existing bounded u32
varint; Qword is eight little-endian bytes. ArrayVarSmall has a byte count
and a descriptor maximum of four. Optional Guid has a 0/1 tag followed,
when present, by exactly 16 bytes in wire order. These primitive encodings
are shared with the existing build209026 codecs.

Armor960 descriptor `0x142606550`, handler `0x1417f4840`, callback
`0x1413c7460`: factory → item virtual +0x298 → `0x1413c8eb0`.

| Field | Encoding | Consumer |
| ---: | --- | --- |
| 0 | Dword | Runtime instance ID |
| 1 | Dword | Type-35 item definition key |
| 2 | Byte | Raw flags, stored at item +0x50 |
| 3 | Dword | Type-66 skin key, pointer stored at armor +0xe8 |
| 4 | ArrayFixed(4) Word | Colors copied in order to +0xe0/+0xe2/+0xe4/+0xe6 |
| 5 | Dword | Type-37 prefix key |
| 6 | Dword | Primary type-35 upgrade item key |
| 7 | Dword | Secondary type-35 upgrade item key |
| 8 | ArrayVarSmall(max4) Dword | Additional type-35 upgrade item keys |
| 9 | Optional Guid | Common item Guid |

Weapon971 descriptor `0x142606f90`, handler `0x1417f4990`, callback
`0x1413c75b0`: factory → item virtual +0x2d0 → `0x1413cb7a0`.

| Field | Encoding | Consumer |
| ---: | --- | --- |
| 0 | Dword | Runtime instance ID |
| 1 | Dword | Type-35 item definition key |
| 2 | Byte | Raw flags, stored at item +0x50 |
| 3 | Dword | Type-66 skin key, pointer stored at weapon +0xe0 |
| 4 | Dword | Type-37 prefix key |
| 5 | Dword | Primary type-35 upgrade item key |
| 6 | Dword | Secondary type-35 upgrade item key |
| 7 | Byte | Unnamed value copied to weapon +0xd8 |
| 8 | Dword | Unnamed value copied to weapon +0xf0 |
| 9 | Byte | Unnamed value copied to weapon +0xf4 |
| 10 | Qword | Unnamed value copied to weapon +0xe8 |
| 11 | ArrayVarSmall(max4) Dword | Additional type-35 upgrade item keys |
| 12 | Optional Guid | Common item Guid |

The common initializers (`0x1413c5ff0`, `0x1413c6370`) check the instance
ID against the constructed object, establish its item definition, copy the
flag byte to +0x50, and copy the Guid to +0x28 or use a fixed client value
when absent. Definition setter `0x1413c6510` asserts a nonnull definition
and stable identity. Missing required definitions are not valid substitutes
for a source-backed item.

The common byte is a **flag byte**, not an established level field.
Virtual +0x40 implementation `0x1413c48c0` returns zero when bit0x4 is
clear; otherwise it returns one plus whether bit0x2 is set. Predicate
`0x1413c58c0` also tests bit0x10. This does not establish all flag names,
their complete interaction, or fresh-character values. The API consequently
retains `raw_flags` without assigning binding or ownership meanings.

Weapon function `0x1413cb6d0` returns +0xe8 when +0xf4 is nonzero;
otherwise it obtains the effective skin and reads its +0x98 Qword, or
returns zero when no skin exists. This constrains the two fields' consumer
but does not establish their full meaning or retail initial values. The
API keeps `raw_d8`, `raw_f0`, `raw_f4`, and `raw_e8` explicit.

## Zero, defaults, and retained creation sources

Armor and weapon constructor prefix inputs come from subtype +0x38 and
+0x60 respectively. Upgrade initialization later resolves the message's
prefix key and **unconditionally replaces** the stored prefix pointer.
Sending prefix0 does not mean “keep the constructor prefix.” The examined
primary/secondary upgrade setters accept a null result and otherwise assert
the appropriate upgrade-item kind; an empty additional-upgrade array runs
no element initialization. These accepted representations do not prove
that a fresh retail character uses them.

Skin0 has a different result. Armor effective-skin getter `0x1413c8d50`
prefers +0xf0, then the message skin +0xe8, then default-skin virtual +0x88.
Weapon getter `0x1413c9510` prefers +0x90, then message skin +0xe0, then
virtual +0x88. Default-skin methods `0x1413c8d20` and `0x1413cb6a0`
obtain subtype +0x0 from the item definition. Thus the null numeric-key
lookup can reach a source definition's default skin; it does not preserve
a nonnull earlier override.

The retained Norn male Mesmer appearance and level1 template provide:

| Template slot | Item key | Kind | Default skin key | Constructor prefix key | Appearance colors, in row order |
| ---: | ---: | ---: | ---: | ---: | --- |
| 2 | 6465 | 0 | 13 | 112 | [117, 615, 480, 0] |
| 6 | 6467 | 0 | 18 | 112 | [117, 480, 117, 0] |
| 3 | 6464 | 0 | 53 | 112 | [117, 100, 615, 0] |
| 29 | 33355 | 24 | 3873 | 112 | Not an armor color row |

The [template reader](209026-creation-equipment.md) preserves those raw
equipment slots and item references. Armor definitions are in file1282841
at offsets1497064/1504936/1519368, with subtype offsets1497240/1505112/1519544.
Their skins are in file1282856 at offsets6230936/6259664/6248520.
Weapon33355 is in file1282834 at3014352, subtype3014528; its default skin
is file1282845 offset6985104. Prefix112 is file1282843 offset2968184.
Weapon subtype +0x58 also references palette90; its Mesmer rows and the
limits of that relationship are in the [palette source note](209026-mesmer-initial-palette-sources.md).

The [creation color trace](209026-creation-colors.md) establishes matrix
rows by equipment slot, with four ordered color selections. The retained
appearance extraction supplies the values above. This is archived creation
input; later preview color fallback and server overrides remain separate.
No instance IDs, common flags, optional Guid policy, additional upgrade
selection, or four unnamed weapon values are established by this table.

## Equipping and selecting a set

Server288 descriptor `0x1425e34a0`, handler `0x1412509d0`:
`Dword characterIndex, Dword itemInstanceId, Byte equipmentSlot`.
The handler resolves the character and its inventory, looks up the item
through item-service virtual +0x10, and calls `0x1411ec7b0`. That setter
asserts a nonnull item, slot below68, and an empty target slot. It stores
the pointer at inventory +0x160 + slot*8, assigns the item location, and
notifies listeners. Server300 is a bag-slot setter and server285 removes
a bag item; neither is this equipment operation.

Server299 descriptor `0x1425bef80`, handler `0x141251070`:
`Dword characterIndex, Byte weaponSet`. Setter `0x141290dc0` stores the
selected set at inventory +0x46c and notifies listeners. It asserts that
the value is neither6 (`CHAR_WEAPON_SET_TOY`) nor11 (`CHAR_WEAPON_SETS`).
These assertions alone are not a complete policy accepting every other byte.

Primary and secondary slot getters `0x14128fdc0`/`0x14128fe70` use the
RVA tables at `0x14128fe40`/`0x14128fef0`:

| Set | Primary slot | Secondary slot |
| ---: | ---: | ---: |
| 0 | 24 | 24 |
| 1 | 25 | 25 |
| 2 | 26 | 26 |
| 3 | 27 | 28 |
| 4 | 29 | 30 |
| 5 | 31 | 32 |
| 11 | 68 | 68 |

Cases6..10 enter diagnostics. This proves that the template weapon's
slot29 is the primary slot of set4. Land/water names are not assigned here.
Set selection plus an equipped source weapon can feed the normal inventory
palette route, but this static trace does not prove the remainder of the
skill-bar availability, selection, or combat state.

## Codec contract and verification

`ArmorItemCreationFrame209026` and `WeaponItemCreationFrame209026` share
`ItemCommon209026` and `ItemUpgrades209026`; `EquipmentFrame209026` exposes
`Equip` and `SelectWeaponSet`. Every raw field is caller-supplied. The
codecs preserve the full wire integer domain, including values that would
violate client gameplay assertions. Callers must validate instance IDs,
definition kinds, character state, empty slots, and set selection before
sending. The codecs choose no loadout; the explicit observer option below
is separate from these raw wire views.

Decoders commit the input cursor only after a complete valid frame.
Creation encoders check the four-element bound before appending any bytes.
`cargo test -p gw2-proto item::` passes six focused tests covering literal
golden frames, independent extracted-schema decoding, stream boundaries,
all array lengths0..4, present/absent Guid, every truncation boundary,
wrong IDs, malformed Dwords, invalid count/tag bytes, and atomic failed
encoding. These are serialization checks, not a retail gameplay result.

## Explicit observer equipment batch

`gw2-game-observe --mission-equipment FILE` accepts an explicit binary batch
alongside `--mission-population`. It accepts only creation960/971 followed
by equip288 frames and, for a weapon batch, one final set299. It sends the
validated bytes once after client412 and the existing1036/431/83 startup,
before any optional quest initialization. Default bootstrap equipment remains
empty. Repeated readiness messages cannot recreate the item instances.

The parser requires unique nonzero instance IDs, nonzero item data keys,
already-created references, matching character indices, unique slots below68,
and exactly one equip for every created instance. Weapon-set values must be
0..5 and select a supplied weapon through the traced slot mapping. Armor-only
batches require no set selection. A maximum of4096 bytes,68 creations and
instance IDs1..4096 bounds this research fixture's resource use; these are
observer policies, not claimed retail ID allocation rules or client maxima.
Actual item-definition validity and all unestablished item fields remain
the caller's responsibility.

The observer suite and focused equipment checks pass, including armor-only
and mixed batches, duplicate/unknown instances, foreign character IDs,
duplicate/out-of-range slots, zero/oversized instance IDs, interleaved creation,
set selection, every truncated weapon fixture and readiness ordering. Two
pre-existing capture-dependent observer tests remain ignored. The optimized
musl observer builds, and six CLI rejection cases fail before binding.

The private Norn experiment is117 bytes: three armor960, one weapon971,
four equip288 and one set299. It uses the four source rows above, synthetic
instance IDs1001..1004, character1 and set4. Common raw flags and all four
unnamed weapon values are explicitly research zero; upgrade keys are absent
and the optional Guid is absent. This does not establish retail values for
those fields or the final template-selection policy.

Builder/source: `~/re/gw2/notes/norn-mesmer-equipment-fixture-20261105{,.rs}`.
Output: `~/re/gw2/notes/norn-mesmer-equipment-20261105/`.
Builder evidence:
`ev_648b5bee971a7450b120344b72b1c840d71444d0ecc4bd1bf97753320f73caad`.
Actual client observations belong in the [playback note](209026-norn-mesmer-playback.md).

## Private evidence

Executable SHA-256:
`99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`.
The helpers verify the executable hash before bounded disassembly; retained
archive helpers record container hashes and relocated references. Relevant
container SHA-256 values:

- 1282841: `62ef0c04651578ca4cf6a4322fe4cb792bd865b614069d639858986b13594e39`
- 1282856: `3c7e9a7c5cdbfc8a65b2fa54ce35935838154764258064c3bec55ab3783883de`
- 1282843: `ac91841287684a0a9262ec53b2e095498448276da6f0208d53421675024b75ea`
- 1282834: `fc03eee83e799747ebed6f13c5445c353e5bfd1198105b1bc0048c8359013232`
- 1282845: `81028af31ccfaf10571e166eb09d49d5c75ce1e6f0eea7656be1cc9c80ad841e`

Private bundle: `~/re/gw2/notes/209026-item-equipment-delivery-20261105.json`.
The following records retain the supporting captures; process captures
exited0 with owned process-group cleanup. No cold decompiler was started.

| REA evidence | Observation |
| --- | --- |
| `ev_4d484472ba31fa98e0078afd2aaeeb59c83fbd07464688a87c1e52f88ca060e3` | Equipment descriptors and handler identities. |
| `ev_90d711f94db938f849e9e5c779c5d1923fec2b73061db9d516936e20737a9105` | Instance lookup, context constructor, factory switch and complete288 handler. |
| `ev_81acfd5ed9ba62570cb002cc468fb84f3db6a0299a53e40f0482375b7346f1cb` | Factory entry and equip setter's store/listener route. |
| `ev_e0ee6e8f09caf96f3158bf509f70eaa2b3ef6cf14b033c783f1d29b1e7a5b5c2` | Creation table and descriptors. |
| `ev_55460e8007b53cd82ec96df445502840f4336fcae90d947b55ced0dfcfe811e9` | Subtype virtual initializers and nested Word/Dword/Guid descriptors. |
| `ev_b737a012ff03dc3c0c70947db890fbe8da1141c8655d2157cfd3481e02f51800` | Armor/weapon subtype initialization and960/971 handlers. |
| `ev_ee0fac4cfbc9bde79ca8c4d14f786dda2f33f9226745f1c07aa97c187817d5d8` | Prefix/upgrade setters, common initializer and stable item-definition identity. |
| `ev_77140012589b6e37c2ea7aacf8e409516659214ab10f61ad41fc71442338117f` | Type35/37/66 adapters and empty variable-array behavior. |
| `ev_59d9e69cad0d69a71ae910604250502d716b2de3cbd575a18910a4e4a7f8636f` | Creation registration protocol0/group0 arguments. |
| `ev_13315858a0fec75ca7dc5e6f2b0648c430460c9de9731f4952b3caceffc31aa5` | Four archived template items' default-skin and constructor-prefix references. |
| `ev_7575ebf6de1e19eaefeb6acfe8822eedb2b432838d5f94864db4194e22cf7e3e` | Common flag bits, default-skin methods and armor skin fallback. |
| `ev_426c82a8ffb613caeb8257aecbfffee5900322993d23012598e09dbbecf0060b` | Weapon skin fallback through virtual +0x88. |
| `ev_7708ad62dd266b8780bf3c7bf6c56173d81d9ac7b6c64216c2c03030cc0a8042` | Weapon +0xf4/+0xe8 consumer. |
| `ev_df0d0917b55f62cfc255143a796684f746d9729529d0d6277da0caa713210fad` | Complete active-set setter and named assertions. |
| `ev_71d333cc74d7a428c8faef27024eb4bd52a128684404a4843cb57313b776a988`, `ev_d771e87943460720da12b89d19ddb246327dfbb384ccdf466307b7f3e7eb8f07` | Primary/secondary set-to-slot tables, captured independently by the parent lane. |
| `ev_3599ddbcc79f5380c9e57039a1d1027876b2eb10fa6e5f71b1a562ec3127d55b` | Parent-lane retained Norn male Mesmer color extraction and source hashes. |

The color capture binds the appearance and colors file; the latter SHA-256
is `f3f02b91e8dd9685053392838cc2fe4e94bc7283ec190f442abbd70da205b578`.
Its private extraction directory is
`~/re/gw2/notes/norn-mesmer-appearance-20261105/`. Parent-lane evidence
is retained in that lane's bundle if absent from this session's export.
