# Build209026 Mesmer sword chain selection

Server628 supplies the live state used by the sword palette's predecessor
tests. This closes the setter-to-selector edge left open by
[the starter-skill note](209026-mesmer-starter-skill-requirements.md).
It establishes how to represent progression; it does not establish when a
server should advance, time out or reset the chain.

All new native observations are static reads of the frozen executable,
SHA256 `99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`.
No client, runtime packet, working archive or server policy was changed.

## Server628 and its two stored values

The registration pair at `0x142170a50` is descriptor
`0x1425d90f0`, handler `0x14125c230`. The descriptor explicitly contains
message628 followed by **Dword character index, Dword skill key, Byte
count**. This agrees with the existing extracted server schema in group
`table-0x14216e940`.

The complete handler:

1. Looks up the character from decoded message +2 and asserts `character`.
2. Obtains its skill bar through character virtual +`0xc8` and asserts
   `skillbar`.
3. Resolves decoded skill key +6 through content virtual +`0x230` and
   zero-extends the decoded count byte +`0xa`.
4. Calls `0x1411fb440(skillbar, resolvedSkillRef, count)`.

The setter compares both values with the current pair. When either differs,
it stores the skill pointer at skillbar +`0x1c8`, stores the count as a
Dword at +`0x1c0`, and tail-calls primary virtual +`0x1f8`, the established
skill-bar rebuild. An identical pair returns without rebuilding. The
setter neither increments the count nor schedules a timer. There is one
pair in this skill-bar object, not a separate pair per weapon slot.

The constructor `0x1411f4720` initializes both to zero: the Qword store at
`0x1411f493b` clears +`0x1bc` through +`0x1c3`, including the count; the
next store clears the skill pointer +`0x1c8`. This is a client construction
fact, not an inferred server reset packet or a gameplay default.

## Exact consumer binding

The constructor installs `0x142164f60` at skillbar +8. That interface has:

| Interface slot | Target | Result relative to the complete skill bar |
| --- | --- | --- |
| +`0xe0` | `0x1409495e0` | Reads interface +`0x1b8`, therefore Dword skillbar +`0x1c0` |
| +`0xe8` | `0x1411f78c0` | Copies interface +`0x1c0`, therefore skill pointer +`0x1c8`, to the output reference |

The normal palette resolver `0x141212d20` creates the previously identified
adapter `0x142169d08`. Its +`0xe0`/+`0xe8` methods forward to those same
slots of the skill-bar interface. These are secondary-interface slots;
the primary table's slots with the same offsets have different meanings.

`0x141213650`, including its continuation at `0x1412136ea`, visits the
selected row's entries in order, stride `0x70`, and returns the first
whose predicate `0x141212fe0` succeeds. The predicate's chain portion is
at `0x141213129`–`0x14121315d`:

- If entry +`0x30` is null, skip this portion.
- Otherwise require that pointer to equal the current skill reference
  returned by interface +`0xe8`.
- Then require the current count from interface +`0xe0` to be at least
  entry Dword +`0x38`, using an unsigned comparison.

All other predicate checks still apply, including availability, level,
context and specialization where the entry requests them. Server628 is
not a replacement for those inputs or the inventory-selected palette.

## Sword90, profession7, category3

The retained cached-content extraction already identified this ordered
row in file1282848. Its first entry is45230, with prerequisite43694 and
specialization59 conditions. The remaining entries are:

| Entry offset | Selected skill | Predecessor reference +`0x30` | Threshold +`0x38` |
| --- | --- | --- | --- |
| 5458360 | 10172 | 10171 | 1 |
| 5458472 | 10171 | 10170 | 1 |
| 5458584 | 10170 | null | 1, ignored by the chain test |

These three skills reside in cached file1282860 at offsets4032088,
4263688 and3965608 respectively. Each row requires level1. The existing
inventory/palette source maps category3 to the ordinary primary weapon
slot5; no new palette selection was inferred here.

Consequently, **assuming the earlier45230 entry is ineligible and the
other ordinary entry predicates pass**, the source-supported selection is:

| Current628 skill reference | Raw count | Selected sword skill |
| --- | --- | --- |
| 10170 | 1 through255 | 10171 |
| 10171 | 1 through255 | 10172 |
| Either predecessor | 0 | 10170 |
| 10172, or a reference matching neither predecessor | Any byte | 10170 |

The value sent is the predecessor reference, not the desired next skill.
For these rows, count2 is unnecessary to reach10172: the skill reference
changes and both thresholds are1. Count0 disables these two follow-ups
without clearing the retained reference. This proves a bounded way to
return the chooser to its base candidate; it does not recover the retail
reset message, timeout, interruption rule or server event that should
cause that return. In particular, null content lookup from wire key0 was
not needed or newly proved by this slice.

The exact E8/E9 query for `0x1411fb440` found the628 handler. The bounded
skill-bar implementation scan found these writes and the constructor
initialization. Neither result is a whole-client proof that no indirect
or alternate state mutation exists. Actual cast acceptance, damage,
completion timing, recharge and automatic repeated-input policy remain
separate from this selection contract.

## Raw codec and validation

`gw2_proto::skill_chain::SkillChainFrame209026` exposes `character_index`,
`skill_key` and `raw_count`, with complete-frame encode/decode methods.
Decoding commits input consumption only on success. The codec preserves
the complete Dword/Byte domains, including zero; it imposes no content
validity or progression policy and is not wired into the mission server.

`cargo test -p gw2-proto skill_chain::tests` passed3 focused tests covering
literal bytes against the extracted schema, consecutive frame boundaries,
all256 count bytes, every fixture truncation, wrong IDs and malformed
Dword overflow/continuation without input consumption. `git diff --check`
also passed. No client-runtime verification or server integration was performed.

## Retained evidence

New complete private bundle:
`/home/blizz/re/gw2/notes/209026-mesmer-sword-chain-20261106.json`, SHA256
`98872864fdf304f503042689991c446c45100f440a35a9a0e058bb5affd675a5`.
All12 captures exited0, were nontruncated and verified cleanup of their
owned process group. Some captures could not inspect unrelated processes'
environments; they do not establish host-wide process cleanup.

| Evidence | Scope |
| --- | --- |
| `ev_35ce9604543840bb0c2854037b141b6a17550166d11ac0744343c77c674b0c28`, `ev_b757f71588f6f17732ba60fe22306cc9639706f7db9518dd642f4a1bdbdf7a94` | Exact handler/descriptor registration and628 field list |
| `ev_7740570e6406655071e1a79e7d2864a4a31bdbc2ca818ec66e88c52ead96b7f1`, `ev_45e255e37564aa0a80b3d2180dc898d09300fad8ea53aaf25f7f96d1ae6f3007` | Complete handler and setter |
| `ev_b33cec11e8b1db43c7e58c8f22165ce1f99879bd6f613687eb6bf97cddb1a692`, `ev_3f0408617d796d1b05a2c312f3352bb7a6ab1603fc039cda963babecd5e0dabf` | Constructor, secondary table binding and count getter |
| `ev_a964f7fbcc48f20e20b978ce63bee6358d4b7118e811d85af89353c7b13f1f45`, `ev_d7a8504aa2656c640b143dcfb431c551ba6b0c5ce40e1de4f007a7c9c453fd03` | Bounded field-reference and exact direct-call searches, with stated coverage |
| `ev_1712b089919c3afbb562e63d4ecf977456c302a27860dcfd8b19041ec9fd9d6d`, `ev_e06e2ce39180516eef82eefcc6594d6bd16767043af64f6c01e2affe0de1cc7a` | Row selection, adapter creation and first-passing entry loop |

Reused evidence in the existing private
`209026-mesmer-starter-skill-requirements-20261105.json` bundle:
`ev_a06197aa4d3398ac0c2c71fa017e2c3f577b9ce615a8fc41d4d814db35fd4df6`
for exact palette entries,
`ev_bba6ac6e72f39d5b4dea317d73ffa82c94e75928fa789237a298e86d1acfbb56`
for the complete predicate, and
`ev_8ece6f4196501560d28b6b25be8bfa480afd154572a3f50dcc1b3aba29e156dc`
for adapter forwarding and the skill-reference getter.
