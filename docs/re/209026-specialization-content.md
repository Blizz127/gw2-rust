# Build 209026: specialization catalog and trait membership

Offline archive census and static REA analysis, 2026-11-01, for the frozen
client identified in [mission transfer](209026-mission-transfer.md).
This supplies content evidence for the [640–643 message layouts](209026-specialization-messages.md),
without choosing a playable build or sending client messages.

## Complete base-container census

All 32 base content containers, file IDs 1282830..1282861, were scanned.
Their hashes accompany the private results. Both selected types use the
already decoded metadata tuple `[0, 20, 40, 24, 0]`: GUID at +0,
UID at +0x14, key at +0x28, name pointer at +0x18, tracking flag zero.

| Content | Records | Unique keys | Key range | Zero/duplicate keys |
| --- | --- | --- | --- | --- |
| Type 386 specialization | 81 | 81 | 1..81, contiguous | None |
| Type 77 trait | 2,425 | 2,425 | 1..2445, with gaps | None |

Every specialization's pointer array at +0x58, count +0x60, contains
**nine non-null references to type-77 traits**. All 729 references resolve.
Each specialization has three traits with +0x70 tier 1, three with tier 2,
and three with tier 3. That is 243 references per tier across the catalog.
This array contains choices; it is not the three selected keys supplied
for one specialization slot in server 642.

The generic content lookup `0x1412e6820` constructs a hash key from content
type and the input key's low 22 bits, searches up to two runtime sources,
and returns null if neither resolves. Its call to `0x1412e9680` checks the
high bits, but the inspected caller does not branch on that result. The
base census's lack of key zero does not establish that every runtime
source lacks it, or authorize using zero as a universal clearing convention.
The wire codec therefore preserves raw u32 keys and supplies no default.

## Discriminators used by Guardian

The Guardian consumer previously traced in
[profession initialization](209026-character-profession.md#specialization-dependent-byte-check)
compares specialization discriminator +0xc8 against 3 and 5; its fallback
is 1. The following shipped records have discriminator values 1..5, all
in file 1282840. The grouping does not assign display names or prove
profession membership independently of the profession-definition array.

| Specialization key | Discriminator | Flags +0x38 | Type-353 progress-reference key |
| --- | --- | --- | --- |
| 13 | 1 | 0 | 24 |
| 16 | 1 | 0 | 100 |
| 42 | 1 | 0 | 103 |
| 46 | 1 | 0 | 30 |
| 49 | 1 | 0 | 45 |
| 27 | 2 | 1 | 85 |
| 62 | 3 | 1 | 417 |
| 65 | 4 | 1 | 466 |
| 81 | 5 | 1 | 521 |

For a concrete membership example, specialization **13** has tier-1
choices **588/581/633**, tier-2 choices **580/584/1684**, and tier-3 choices
**585/586/589**. These are candidate content keys, not a selected or
unlocked fixture. The Guardian secondary-method GUID reference
`6804e96f7e8035418c057debff649834` resolves to trait key **2200** in file
1282858. Its gameplay name and role remain unassigned.

## Eligibility checks are separate from wire assignment

Specialization method `0x141224db0` checks a proposed major trait against
the active or pending specialization, requires trait +0x70 to equal
`major_trait_slot + 1`, calls the player's trait eligibility method, and
searches the specialization's +0x58/+0x60 pointer array for that exact
trait. It also checks character/runtime state. Thus matching a key and
tier is necessary in this path but is not a complete eligibility test.

Specialization assignment predicate `0x141224f00` checks character state,
effective-level/slot eligibility through `0x1411ce820`, and whether the
specialization is already present. Content flag +0x38 bit 0 restricts the
specialization to slot 2. It obtains the owned coreStats profession, resolves
the profession definition through character-context virtual 0, and searches
that definition's +0xb8 pointer array, count +0xc0, for membership. This
profession-definition membership has not yet been reconstructed offline.

The level helper uses three content-setting selectors at `0x142160978`
and compares the supplied level with a resolved threshold. Conditional
runtime paths can substitute zero for that threshold. This trace does not
assign fixed unlock levels. These client request predicates do not replace
the distinct effects and prerequisites of incoming 640–643 messages.

## Evidence

Private census outputs: `specializations-209026-{0-8,8-16,16-24,24-32}.json`
and `specialization-traits-209026-{0-27,27-54,54-81}.json`, under
`~/re/gw2/notes/`, mode 0600. Private cumulative bundle:
`~/re/gw2/notes/209026-specialization-content-1.json`
(1,119 records, 24,172,145 bytes).

| REA evidence | Observation |
| --- | --- |
| `ev_c38fac1ff2d7cd5aec5acd761bc955b4b0c7c27b1f95653590306c0e180b5664` | Generic key lookup and two-source search. |
| `ev_445c4adcdc0e4e7107778bf1af6bb4bf8d00a791e7ab9b672660780ffb355cb2` | Containers 0..7. |
| `ev_f4f2906293441de5b1777e27fd7ac51201555d0e34173054f56b9481807df543` | Containers 8..15. |
| `ev_2900f8a91e9616853d37888a2575f766a4b9b96fd7a02035e53691629370be1f` | Containers 16..23. |
| `ev_4478671278bf197df3271657561bbff88ba488d9a4adcb05a01d30b585228291` | Containers 24..31. |
| `ev_1db5bbf53d953b7d6b21a08c975790270be7cab693f6eb651a8f5958c53ed9c1` | Aggregate counts, duplicates/zero audit, selected records and high-bit predicate. |
| `ev_3e36ef04ae7eedc7f075fa26c414f16dc8378dee3464ec6562bc0fc2acae1e09` | Major-trait eligibility and selected-trait getter. |
| `ev_75e63824070b5c9176a860778d2c1ef82c22b5da5a376a40230b999267be8df6` | Specialization assignment predicate and profession membership. |
| `ev_a4468e001c536e16e6108077ca2836a97933f8d191672e564736725b2aa3c5b1` | Level/slot threshold lookup. |
| `ev_7dea17fc30440ca0ed031b93437e53d4ea0f4cb507acd5eac5c46818c5fc3810` | First 27 specialization trait arrays and tiers. |
| `ev_a18367ce4854c23dfe816fb701f4a8e430b3d9f1964b85a6bbed955bcd165978` | Middle 27 trait arrays and tiers. |
| `ev_5670817c23d1636d8df80fcfdbd12e33c313c9b35c49141b837f8e3657e2fe70` | Final 27 trait arrays and tiers. |
