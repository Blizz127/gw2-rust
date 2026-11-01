# Build 209026: player startup tail, bounded dependency follow-up

Static REA trace, 2026-11-01, on the frozen executable identified in
[mission transfer](209026-mission-transfer.md). This follows server 431's
[wardrobe/startup path](209026-wardrobe-startup.md). No client execution
or User Agreement interaction.

## Embedded player state

Context helper `0x1411b9680`, called with a null second argument after
431 sets context +0x208 bit 0, obtains the controlled player through
context virtual +0x70 (`0x140498900`, returns +0xa0). It returns if that
player is null or context virtual +0xe0 reports the bit clear.

The tail's inspected player getters resolve as follows. Player vtable is
`0x14215e9d8`; constructors are called by the previously traced player
constructor. These objects do not each require a separate allocation packet:

| Player slot | Getter | Result | Relevant inspected consumer |
| --- | --- | --- | --- |
| +0x08 | `0x1411bacd0` | Embedded +0x330, account object | Account +0x100, +0xf8, +0x1c0, +0x158. |
| +0x18 | `0x1411bb240` | Embedded +0x9760, progress object | Optional progress lookup by GUID/key. |
| +0x48 | `0x1411bb430` | Embedded +0x9ea8 | Virtual +0x20 reads its +0xa0 Dword. |
| +0x1b0 | `0x1411bace0` | Embedded +0x218, achievements | Four initial Dwords are summed by virtual +0x20. |
| +0x3e8 | `0x1411bb460` | Embedded +0xa090 | Virtual +0x60 reads its +0x40 Dword. |

Account constructor `0x1412367a0` installs primary vtable `0x14216d3f0`;
the source diagnostic at `0x14216d5f0` names `ChCliAccount.cpp`.
Its selected virtual methods are simple reads:

- +0x100: `0x14045de30`, returns Qword +0x70; the caller keeps its low
  Dword. Constructor value is zero.
- +0xf8: `0x14045de40`, returns Dword +0x68; initially zero.
- +0x1c0: `0x141237700`, returns whether Qword +0xc0 is nonzero;
  initially false.
- +0x158: `0x1412374e0`, reads +0xb8 but tail-calls `0x140d859a0`
  with EDX=0. That target returns EDX, so the inspected frozen method
  returns zero without a further pointer dependency.

The +0x9ea8 constructor `0x14120f4d0` installs `0x142169a78` and clears
+0xa0. Its +0x20 method `0x140282190` returns that Dword. The +0xa090
constructor `0x141209da0` installs `0x1421693a0` and clears +0x40;
virtual +0x60 `0x1402b6260` returns it. The tail clamps that latter value
to an unsigned minimum of 1 before calling `0x1409a4060`; its gameplay
meaning is not assigned here.

There is also a genuinely optional pointer at player +0x97b8. The tail
calls player virtual +0x338 (`0x1411bb2f0`) to test it before invoking
virtual +0x330 (`0x1411bb250`) and then object virtual +0x60. It skips
that branch when absent. This is distinct from 431's unguarded wardrobe
and reward-track manager dependencies.

## Achievement/content processing remains material

The tail calls `0x1411b8ba0` with 1 when its own second argument is null,
as in 431. This helper reads embedded player +0x218 through virtual
+0x20 (`0x141216240`), summing Dwords +8/+0xc/+0x10/+0x14 into context
+0x260. Constructor `0x1412158c0` clears all four. The source diagnostic
`0x14216a470` names **ChCliAchievement.cpp**.

This helper does more than read the zero fields: it enumerates content,
looks up GUID-selected achievement categories, and calls achievement
virtual +0x58 (`0x141216470`). That method explicitly handles a null
category by writing zero to supplied output fields. For a nonnull
permanent category, it dereferences category +0x60 without a null guard,
iterates its achievement pointers and calls further achievement methods.
The outer helper also asserts permanent category type and nonnull
permanent-category data in its selected content list.

Achievement virtual +0x68 (`0x1412167d0`) and +0x70 (`0x141216920`)
derive a nonnegative remaining interval from +0x70/+0x78, +0x68 and
`0x1409cb4a0`; no additional component pointer is used in those two
methods. The category lists and final global callback are followed below;
general achievement evaluation and context notifications are not exhausted.
The evidence narrows allocation requirements; it does not establish an
empty player object as a complete usable startup state.

## Forced threshold refresh and the zero-point candidate

The tail passes 1 to `0x1411b8ba0` when its second argument is null,
as in 431. The achievement sum is stored at context +0x260. If the sum
matches the old value, the helper skips its first refresh block only
when that second argument is zero. Thus constructor-zero counters alone
do not bypass the forced startup work.

Context primary virtual +0x38 resolves through `0x14215e000` to
`0x1411b4b00`. When its cache count +0x4c is zero, it enumerates content
**type 3** and inserts the records into context +0x38, sorted ascending
by their unsigned Dword +0x2c. It copies those pointers to the caller's
array. No character component is allocated by this helper.

The refresh scans that sorted list for the first record whose +0x2c is
strictly greater than the current achievement sum. It uses that record's
optional +0x38 object; a null object skips the nested progress work.
Otherwise it visits at most 32 entries from object +0x28/+0x30, then
conditionally looks up progress using another referenced record's data
key and queries its virtual +0x38 with argument 8. This computes a mask
stored at context +0x2b8. These are bounded pointer/branch observations;
the gameplay meaning of every nested record is not inferred.

A hash-checked census of all 32 frozen base containers found **121 type-3
records**, 121 distinct thresholds and no zero thresholds. At a sum of
zero, the unique first record is data key **101**, threshold **100**, in
file 1282840 at content offset **0xe048**, SHA-256
`40368ecc24ac813b0c2f33ab00a1f800be915cac1f9456343052b6dd7b284d1a`.
Its +0x38 pointer is null. Consequently, for this content and zero sum,
the nested per-item progress-query loop is skipped and its computed mask
is zero. The largest nonnull sublist among the 121 records has 19 entries.
This does not skip the later category enumeration or prove the complete
startup tail ready.

## Fixed categories and capped child cache

The later refresh looks up three GUIDs through content virtual +0x68.
The table records their four little-endian Dwords in instruction order;
these are content identifiers, not account identifiers.

| GUID Dwords | Frozen record | Use |
| --- | --- | --- |
| `20508e27 40ffb20a 2b42e09f 95358149` | Type 1, key 97, file 1282845, offset `0x98d0` | Aggregate at context +0x284; timer at +0x288. |
| `ae3e6ce1 4bcd6ef1 3707d9b8 a2d22e7a` | Type 1, key 96, file 1282850, offset `0xa5b0` | Aggregate at context +0x28c; timer at +0x290. |
| `647d3b02 41f02302 886da388 cf556fbb` | Type 2, file 1282848, offset `0xb688` | Parent pointer filter for the capped child cache. |

The first two records have +0x58 tag **1**, with nonnull +0x60 data
at `0x9968` and `0xa648` respectively. Their data +0x20 selectors are
0 and 1. Achievement virtual +0x140 (`0x141216bc0`) pairs that data
pointer with the manager's +0x58 or +0x5c scalar. Virtual +0x138
(`0x141215ef0`) uses iterator `0x141215100` to find the first achievement
with +0x3c mask 4. Both frozen data objects have **zero counts at +0x08
and +0x38**, and null arrays at +0x00 and +0x30. The iterator therefore
returns null regardless of the scalar, without entering either list.

For tag 1, aggregate virtual +0x58 returns directly at `0x1412167ae`
when that search returns null. It **does not write its output** on this
branch. The two startup call sites initialize their output Dword to zero
before calling, so the values compared/stored at context +0x284/+0x28c
are zero. This differs from the explicit zero-output handling of a null
category and must not be generalized to arbitrary uninitialized callers.
The timer calls still run separately.

The third GUID resolves uniquely, but **none of the 486 type-1 records**
in the 32 frozen base containers has a resolved +0x50 pointer to it.
Thus its selected child list is empty. This conclusion relies on the
nonnull resolved parent; a missing parent would instead compare against
null +0x50 pointers and is not equivalent to skipping enumeration.

For a nonempty list, the helper sorts ascending by unsigned +0x40 and
caps the count at three. Context +0x298 stores eight-byte cache entries
(data key +0x28, aggregate), with pointer +0x2a0 and count +0x2ac.
Count/key changes rebuild the cache through aggregate virtual +0x58.
When keys match, it asserts permanent-category tag 0 and nonnull +0x60
data, then checks the first achievement with mask 4 and updates the
cached value from query virtual +0x50's output +0x18. With the frozen
empty selection, neither per-category query loop runs; a previously
nonempty cache can still be cleared and signal a change.

Source SHA-256 values:

- 1282845: `81028af31ccfaf10571e166eb09d49d5c75ce1e6f0eea7656be1cc9c80ad841e`.
- 1282850: `7c4267d7cba4fc5878a66512f207b53f4abb69f86e3a49102d9339475b79e4e4`.
- 1282848: `176ae80621ebfe33adc0bd58bbce865b3d582e402890b1f624a80d0af5793576`.

Validation refinement: the shared analysis helper previously compared
expected hashes only for files listed in the map-content inventory.
It now merges the prior appearance and numeric-content inventories,
rejects conflicting hashes, requires all 32 expected hashes, and checks
every opened container. The category census above passed that stricter
check. The 121-record threshold census was also rerun successfully and
produced the same result/evidence ID; REA does not bind external helper
source bytes to that ID. Failed preliminary helper assertions are retained
in the bundle and are not successful census results.

Private bundle: `~/re/gw2/notes/209026-player-startup-categories-1.json`.
This is static content/branch evidence, not a client startup smoke.

| REA evidence | Observation |
| --- | --- |
| `ev_b17d81d83f26c55be35a164797274f9361e36138e4415f95853f1a3468cb15b4` | GUIDs, caller-initialized outputs, timer calls and parent filtering. |
| `ev_04d3a4550c0cc568180ff731b2c2f66cdcf552f2d3b2e2a1c835d0bd1087619b` | Sort, cap, cache comparisons, assertions and rebuild queries. |
| `ev_4f6b0bfd0e9456805362c9779442a3258219f3c3b80c0609355793c3dc8b42fc` | Cache cleanup and aggregate tag-1 early-return branch. |
| `ev_18ff06d2556f001daf52073dd1e767cdc01b259f1c309216c33ef300b60f90f1` | Complete category census with all 32 expected hashes enforced. |
| `ev_921041401293c0edab05f6f74299fca6f5f42838f0d681be9e22bb055f3546db` | Achievement method-table entries. |
| `ev_82af73bed1e79a04f868f400c9aada1474db7a1b133c1848346d276100be7f86` | First mask-4 search, aggregate output/return distinction and constructor scalars. |
| `ev_f3d0c39148eab1557181d5533c9ff2d9e7c3560328263c06eb0a24d9f5d026e8` | Complete selector and iterator, including both empty-list exits. |
| `ev_57cec4f42dfd7dbe203cd6dc69fc22b13aa92b903c206233f80aebdd097bab00` | Both fixed categories' empty list data and valid selectors. |

## Progress-backed fields and required definitions

The complete ordinary tail after the achievement helper queries six GUIDs
through progress-manager virtual +0x20. The manager vtable is
`0x14216c780`; this slot is **`0x1412319c0`**, distinct from its named
lookup +0x10 (`0x141231af0`). The GUID method first resolves content
virtual +0x68, asserts a nonnull definition, then forwards definition
+0x28 to the sorted runtime-object lookup +0x28 (`0x141231a20`). That
lookup returns null when absent and does not allocate. Thus the progress
**objects** are optional here; their content **definitions** are required.

All six definitions resolve uniquely in the frozen base content. They
are type 53, each with channel mask **0x100** (channel 8), matching the
literal channel argument used by the tail.

| Key | Data type | Context destination | GUID Dwords |
| --- | --- | --- | --- |
| 576 | 3, MAXIMUM | +0x268 float | `282b59dc 492fc9aa 3e044da7 d04cd7e6` |
| 411 | 0, BIT | +0x270 mask 1 | `0d596930 47b6b53d c050a1aa 6cac5566` |
| 412 | 1, BIT_MASK | +0x270 mask 2 | `08f1c132 493aa169 cb974dae a1a85fa8` |
| 413 | 0, BIT | +0x270 mask 4 | `c905490a 418dbcb2 79b33688 522116df` |
| 409 | 2, COUNTER | +0x274 Dword | `3b295693 4be18309 17bfa692 fb7cdd34` |
| 410 | 1, BIT_MASK | +0x264 Dword, low byte of query result | `3abce8a9 43fa812a bfb31e9d 913b381b` |

Key 576 is in file 1282855 at `0x40cfd0`; the other five are in file
1282833 at `0x3eb580`, `0x3eb3b8`, `0x3f1e60`, `0x3edb48`, and
`0x3eced0` respectively. Source SHA-256 values:

- 1282855: `9f9226f71e3871066237d560e95b62ef33c847818a7bd78cea586507f492cfa6`.
- 1282833: `5ab847328dd307407d9701711890f08282f9c6f3639d24bb9684ea46006e26ef`.

Absent objects yield zero for the scalar fields. The key-576 conversion
divides by the float literal 100 before its clamping sequence; absent
progress therefore produces zero. The three +0x270 bits are queried
only while each bit is clear and are only set by a nonzero result; a
missing object does not clear an already-set bit.

The tail compares old/new values through conversion helpers and ORs
change bits into context +0x278: masks 2, 4, 8, 0x10, 0x20, 0x40 and 1
for +0x268, account-backed +0x26c, the three +0x270 bits, +0x274 and
+0x264 respectively. These bits are also serialized state, as traced below; their updates
do not by themselves establish an immediate listener dispatch.

Even absent key-409 runtime progress does not skip conversion helper
`0x1411c6380`. That helper requires GUID Dwords
`1d10f932 4e7db5e8 f93d2882 4269f3d8`. It resolves to **type 0, key 283**,
file 1282853, offset `0x7cb8`, SHA-256
`33ca7ae0d8e93f4fbfbe966e3e639a80e9e7ba6dcef710819cbb86e231ee0603`.
Its +0x78/+0x80 array has six 32-byte entries with unsigned thresholds
**1, 1000, 3000, 10000, 50000, 250000**. The helper returns the index of
the first threshold strictly greater than its input, or count minus one
when none is greater. Input zero therefore returns index zero. The
content lookup and nonnull assertion occur for both old/new conversions.
No gameplay name is assigned from these numbers alone.

The final preferences callback is controlled by whether a queried runtime
object equals the helper's second argument, rather than solely by changed
scalar values. In the 431 path that argument is null: an absent first
progress object already makes the callback condition true. Other queried
objects can also set it. This explains why the callback can run with
unchanged zero progress. It remains the scheduling method described
below, not an additional character-component allocation.

Private bundle: `~/re/gw2/notes/209026-player-startup-progress-1.json`.
All content reads used the strengthened 32-container hash checks.
No progress messages were transmitted and no client smoke was run.

| REA evidence | Observation |
| --- | --- |
| `ev_4d766caef991bb5698d9c3ff279239ad5908386a17886385d7bcbb7aa028e907` | Tail GUIDs, null guards, channel arguments and flag updates. |
| `ev_bd8acb14f7ab87f291c6a44b2016cd2ba43a2ccda15c41b633b349fc554cbe27` | Remaining scalar writes, final callback condition and converter entry. |
| `ev_6d8b4ece7e7ea7e438af4015c62cdc4f38e8dd0de0ecc9fb70499ed3be8eaabb` | Boolean conversion and threshold comparison/return. |
| `ev_bbadc18ae94324625a600ab3c388632a1776e4cdbf9b8d3eb759f5f0fca653c9` | Six progress definitions and required threshold content census. |
| `ev_7d59af670a444c9c9b25844aae70f7b6dc6a3ca3145edce26984e2644f5c75a8` | Complete nonallocating sorted object lookup. |
| `ev_aef4a9634be7ac79cb12c53592fcbf2fc156c284a1d8acbeb599cc683ae29281` | Progress-manager table entries. |
| `ev_4a4584f17c3f18c98be9b1e5685fe9f8b9c0edc77d520c60aedaa2844de1ceaa` | Complete GUID lookup and required-definition assertion. |
| `ev_b4c5aca40c616a96a4bca4d284fdcc1c5e33edd4f9708ca4d6cf7ee21fe16bd5` | Float literal backing the key-576 conversion. |

## Change-mask storage and reset path

A bounded scan of `0x1411b0000..0x1411d0000` for displacement +0x278
located serialization, restoration and explicit clearing in addition to
the startup writes. This scan is not an exhaustive whole-program caller
or alias analysis.

`0x1411b7810` packs context +0x278 into a Dword at record offset **0x0f**.
The same record includes the startup scalars, category values and at
most three child-cache entries; unused child slots are zero-filled.
It passes the record to the supplied object's virtual +0xa0 with tag
**`0x73726c70` (`plrs`)** and descriptor `0x14217d560`.
`0x1411b7b30` obtains that tag via supplied-object +0x58 and restores
record +0x0f directly to context +0x278. Thus this flag word is part of
stored player state, not merely a transient notification parameter.
The storage object's final I/O implementation and other consumers are
not established by this bounded trace.

If the supplied storage object or its `plrs` result is null, restoration
tail-calls reset helper **`0x1411b7aa0`**. That helper zeroes context
+0x260 through +0x290, frees/resets the child cache at +0x298, clears
+0x2b8/+0x2bc/+0x2c0/+0x2c8/+0x2cc and sets +0x2c4 to **1**. It has
no player-component getter or allocation call. These are observed reset
values, not proof that this branch was taken in a live startup.

The independent clear method **`0x1411b4520`**, at context primary
virtual +0x118 (`0x14215e118`), writes zero to +0x278 and tail-calls the
shared preferences scheduler with `(0, 0)`. It does not dispatch a
character listener in its body. Direct-call candidates for packing and
restoration were validated at `0x1411b6eed` and `0x1411b7031`; their
wrappers first require selector zero and adjust the interface pointer
by -0x10 before passing the context to these helpers.

Private bundle: `~/re/gw2/notes/209026-player-startup-state-1.json`.
Only executable instructions were inspected. No saved-state, preferences
or credential-cache file was opened, and none of these routines was run.

| REA evidence | Observation |
| --- | --- |
| `ev_bf97c45d9eaa079cf9d94a2bdf6b9ecaa72e2a6a198d8fdc6e4b3c07d36265f3` | Bounded field-use census. |
| `ev_54370a7856d0ff0a395cd2d4364ceb5d7a98e311c720e9d1ac2af78bad51025f` | Unwind boundaries used to select deeper reads; split regions are not complete functions. |
| `ev_42a352eb9c2c172bb53d354e14302f39db062ff033843348f8743ae6c9128e23` | Packed field offsets, restoration, full reset and full explicit clear method. |
| `ev_2cea704fe8f23c7922b324f86e7df77ac37cb64ccca30cdefdf4b351b6aa8fbd` | Packing call/tag, zero fill and restoration's null-to-reset branch. |
| `ev_1bec9f7ba93f297a94fe0267fbcffa538ee800e3568aa98ac692d4876be2a0d6` | Clear-method context table entry. |
| `ev_a0132759adec20eceb5417e7dc65bf4b256235f6bd3437ae0d78cf250ea753f1` | Direct-call candidates, subsequently validated. |
| `ev_206792577019d29998b81fd5480983405a14a20423b79a0b79cecbf32208b1e6` | Packing/restoration wrapper call sites and selector checks. |

## Shared preferences callback

Both `0x1411b8ba0` and `0x1411b9680` can finish with
`0x14027e6d0()->virtual +0x60(0, 0)`. The getter returns static object
`0x142558400`, whose initial primary vtable is `0x141929860`; +0x60
resolves to `0x14027e7e0`. Its assertion source names
**Game/Pref/PrContext.cpp**, and its selector assertion names
**EPrefLocation**.

The complete callback checks object +0x90 bit 4, then the selected
location's enable condition. For argument 0, that is +0x90 bit 1. When
admitted, it compares the existing timestamp against the selected delay
(argument 0 selects literal 15000), cancels/schedules a callback through
`0x141069f40`/`0x141069fa0`, updates the timestamp and tracks a pending
bit at +0x94. It does not dereference another character component.
The eventual scheduled callback and persistence behavior remain outside
this trace; no preferences or cache files were read or written here.

Private bundle: `~/re/gw2/notes/209026-player-startup-thresholds-1.json`.

| REA evidence | Observation |
| --- | --- |
| `ev_c3d7b099a0d106027ea0fdb2484d787b736d9af5f451a74131af3b526bb20b75` | Bounded call census used to select reads, not a complete control-flow proof. |
| `ev_240e92779abd4a1d09f1443546d44f4b1fadab690b7d8c3ff8378342f1f5c496` | Tail callback arguments and conditional dispatch. |
| `ev_bb8c11bc81b016416333a585abc0f23ee211dae2185ec23c7a6e733acab449fd` | Forced-refresh argument/branch and static preferences getter. |
| `ev_62ca150f7db338bb1b0a8c9461dea6e87c88a08f1b24a40c455de5c7bbeaaf0c` | Static object's initial vtable. |
| `ev_0170be5e828a48bf2e601f76945db3a717733c008bcb2a696599a6e552ac3501` | Preferences +0x60 entry. |
| `ev_6f790a3b2e613fb4cc381813d2a6d25df68bd62c4f3e7c388e419e1f256ddfb9` | Complete scheduling callback and threshold selection/progress branches. |
| `ev_27ac1b9f549595bc9ea6d979bb4cb33921b321115f5b2a1d52bd898e4de8d70d` | Preferences subsystem and selector assertion literals. |
| `ev_1c3d047b12d3bd469194d3c59ec9478b4bfc977ddd3b48fbadece70e0b67cb66` | Context +0x38 entry. |
| `ev_51ad9eabe5fe13f1ed7a4752ffce8798f230cb5116f6a984c7fbd56ff8bdd131` | Complete type-3 list construction, sorting and copy. |
| `ev_9cb216855bb3227d1979ee9fba2d632aa290b18313309a8ba795f5f49b791c17` | Hash-checked threshold census and zero-point record. |

## Evidence

Private cumulative bundle:
`~/re/gw2/notes/209026-player-startup-tail-1.json`.
Bounded call-only listings were used to choose deeper reads; their
omitted instructions are not treated as a complete function trace.

| REA evidence | Observation |
| --- | --- |
| `ev_88f246512c93f9014c3a94eae3fe8544e880ab1e2b6a3f55356a24ede3d061f0` | Tail calls/guards, controlled-player getter and achievement helper entry. |
| `ev_64744b2135adbe97e612ea40adc663fa032936ac5f55de9333fdf820b04630f6` | Embedded getters, optional pointer predicate and bounded achievement call listing. |
| `ev_347dcdbe6a826d9f538a14a5fe71542b05f2127b142c575127387699486d8470` | Achievement source/sum, category queries and content assertions. |
| `ev_65950bc990c711e28556cf9a0b1bddc3408b8d7f2dcae410be9deace305a80af` | Constructors and selected method-table entries. |
| `ev_423262bdfefa6a057d1023c6b02f61547c633c8914d1ffbf1071984414d823fc` | Category null branch, timer methods and scalar getter. |
| `ev_dcf5b25ceb2c1ff85f455fd4b43a16d3337c06527a45185fcb46dd1067ea97f0` | Account vtable/initial values and category-output epilogue. |
| `ev_76816d2791756f3edbeb7e5840200ac30953db7f6a2c9e604c48bd7df0fa2edc` | Account method slots and content iteration call sites. |
| `ev_c4fccc474f54c62654173d3b3866b0bc9788bb58174a8f8738c0c0b48297b31c` | Account methods, source diagnostic and embedded scalar initialization. |
| `ev_ed326da96216d3b69dbd11cc2d15d37bbad82ce92581d4afbe1caccf5db4aaed` | Constant-return target, +0x9ea8 initialization and permanent-category dereference. |
