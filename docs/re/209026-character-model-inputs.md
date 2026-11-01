# Build 209026: ordinary model initialization inputs

Static REA follow-up to [view finalization](209026-character-view-finalization.md).
No client or renderer was launched, and no packet fixture was sent.

## AvCharModel and initialization dispatch

The ordinary component at view +0x820 is identified by assertion source
`Game/AgentView/AvCharModel.cpp` at `0x14214e700`. Its vtable
`0x14214e458`, virtual +0x38, resolves to `0x14116a300`.
The [view initialization trace](209026-character-view-reveal.md) calls
this with the whole view and third argument zero.

The complete initializer stores the supplied view at model +0xc0 and
asserts nonnull. It obtains the character through view +0x08 virtual
+0x168 (`0x141107be0`, returns whole view +0xb38). Character flag bit 10
sets a model flag; bit 54 later selects one specialized loading branch.
The precise flag names remain unresolved.

Shared model construction `0x141151ab0` initializes mode fields +0x34
and +0x38 to **3**, model pointers +0x40/+0x48 to null and parent +0xc0
to null. The ordinary constructor also clears +0xe0/+0xe8. Initialization
can change +0x38 through the AvContext priority helper and model virtual
+0x1a0 (`0x1411696d0`). Its result depends on context policy and the
model's +0xe8 record. The exact mode names and resulting runtime value
are not inferred from the constructor default.

After resetting model state, the initializer chooses:

| Condition | Direct action |
| --- | --- |
| Model mode +0x38 == 2 | Set whole-view +0xb78 mask 0x80; skip the three direct loading helpers. |
| Otherwise, character flag bit 54 set | Call `0x141169890`. |
| Otherwise, view +0x08 virtual +0x1d8 returns a nonnull record | Call `0x141169ff0`. |
| Otherwise | Call `0x141169cf0`. |

The record getter reaches `0x1411087a0`: it first uses whole-view +0xb50
when nonnull; otherwise player identities return null, while other
identities can use character virtual +0x300. The base view constructor
initializes +0xb50 null. This describes the branch conditions, not a
guarantee that intervening setup cannot populate that field.

## CoreStats and inventory consumption

The initializer's `0x14116c240` calls model virtual +0x188
(`0x1411693e0`) to obtain three classification values. On its ordinary
player path, with no override record and its global predicate clear,
that callback dereferences coreStats through character +0x08 virtual
+0xb0 and reads sex (+0x48), race (+0x38), and profession (+0x28).
Other branches use fallback values 2/5/11. The context lookup and those
fallbacks do not validate a supplied appearance.

In the no-record loading helper `0x141169cf0`, modes 0 and 3 reach
`0x141166580` with the character and output locations for model +0x40
and +0xe0. This builder asserts player identity or the separate character
+0x438 predicate. It then dereferences coreStats and passes primary
virtual +0x30's **existing 45-byte appearance** to `0x1411be9b0`.
That is the same appearance conversion used by the earlier preview work,
with established [content requirements](209026-appearance-validation.md).

The builder calls `0x1412d84b0` with the converted description when its
optional final input is null, or `0x1412d8570` with the supplied record
otherwise. It expects a nonnull output model. These loading functions'
completion and rendering behavior are not exhausted here.

Later in the complete builder, character +0x08 virtual **+0xc8** resolves
to `0x1411d7f60`, which asserts **m_inventory** and returns whole-character
+0x3f0. The builder dereferences that inventory without a null guard and
iterates seven entries, using virtual +0x128, +0x370 and +0x130 before
conditionally adding attachment data. Missing individual item results
are checked; this does not establish that every empty inventory is valid.

A separate getter, character +0x08 virtual +0xf8 (`0x1411d88e0`), returns
whole-character +0xb00. The builder's initial loop explicitly checks this
pointer for null and uses zero for its queried value when absent. The
follow-up below identifies it as the wardrobe manager and finds a later
player-only assertion: the local null guard must not be generalized to
the complete builder's callees.

This connects the known **coreStats and inventory before finalization**
requirements to actual model construction. It does not establish valid
appearance defaults, all attachment dependencies, priority policy,
asynchronous completion or rendering readiness. The follow-up resolves
the later `0x141165d60` call; other initialization branches remain open.

## Follow-up: wardrobe is required by the later player helper

When the builder's optional final input is null, its tail at
`0x141166a10` calls `0x141165d60` with model +0xe0 and the character.
The ordinary `0x141169cf0` caller supplies whole-view +0xba8 as that
optional input; `0x141159290` initializes +0xba8 to null.

The helper spans `0x141165d60..0x141165f49`; the PE unwind entry at
`0x141165d60` ends at `0x141165e41`, inside the function, so inspecting
only that unwind range would omit live branches and its return.

For a player identity, it obtains character primary virtual +0xd8
(`0x1411d88c0`), asserts **transformationMgr**, and queries its virtual
+0x10 with `(0, 0)`. The getter returns the embedded character +0x548
object, not a new allocation requirement. The returned optional record
is checked for pointers at +0x58/+0x28.

It then obtains character +0x08 virtual +0xf8 and asserts
**wardrobeMgr** (`0x141adb358`) nonnull **before** testing that optional
transformation result. This is whole-character +0xb00. Thus the earlier
loop's null tolerance does not make wardrobe optional for this player
branch. If the transformation result is absent and wardrobe virtual
+0xc0 is nonzero, the helper returns; otherwise it uses inventory
entries 5 and 1, coreStats sex/race, and conditional model virtual
+0x70 calls. Individual missing inventory entries are handled here.

The [wardrobe allocation trace](209026-wardrobe-startup.md) already
establishes server 247/248 lazy allocation. A first all-zero 248 is a
candidate allocation with no changed-value notification. For a candidate
bootstrap that reaches ordinary player model creation during server 170,
wardrobe should therefore be prepared before that model-building path,
not deferred solely to the later server-431 prerequisite. This is a
conditional ordering refinement: loading policy and earlier callbacks
may select another path or allocate state, and no runtime sequence has
been validated.

Follow-up private bundle:
`~/re/gw2/notes/209026-character-model-wardrobe-1.json`
(1,574 records; 30,566,133 bytes).

| REA evidence | Observation |
| --- | --- |
| `ev_e3fe35d914e6ae8eefd395bda76dcf66ea6ce90b3d45d8caefda1d848859be64` | Player branch and unconditional wardrobe assertion before the transformation-result branch. |
| `ev_014d1df8e44875405b3db118b572fecfe156a023276f4fb24fa4f32239adccec` | Remaining helper through final return, inventory and coreStats use. |
| `ev_a8822f9ec1efcaf4b909daebaf4da18aa4fdc5c43fefef6d905c50376fb5a44e` | Exact wardrobeMgr assertion literal. |
| `ev_92255413eeac20e93a09441b53e5b345cda62089f96b5df43d5fc613b6f5449f` | Primary-character +0xd8 entry. |
| `ev_2b84ac6ca34cbb4af091d5ed93de5a6fdbbcff9363d1d1ad0f0ba54715e5d60d` | Embedded transformation getter and raw wardrobe pointer getter. |
| `ev_889e51710c75fecca914bfa4aefc2a008970786314a9e0516a123790735543fd` | Exact transformationMgr assertion literal. |

## Initial model mode and Queensdale policy

The complete priority insertion helper `0x141159620` appends a model
record and assigns its initial mode from its pre-insertion index:
index below manager +0x24 gets 0; otherwise index below +0x28 gets 1;
otherwise it gets 2. A nonnull character whose primary +0x358 predicate
returns false overrides this to 0. The helper stores and returns the mode.

For player identities, that predicate (`0x1411d9220`) returns the inverse
of mission predicate `0x14140f290`. The latter asserts a prepared map
definition and tests **mapDef +0x98 & 0x02000801 != 0**. It obtains the
definition through `0x141411100`, global context +0x1c8, mission +0x1e8,
the map-definition field already traced in mission transfer.

A hash-checked content read of Queensdale's type-45 record, data key 15,
found flags **0x50000200**, so the mask result is zero and the player
predicate is true. Queensdale therefore does **not** force mode 0 through
this override; the initial priority thresholds remain relevant.
The record is in archive file 1282830 at content offset 0x394838, source
SHA-256 `e5d11e4fae8627d3efced749e7ab599b26abac59e502716aaf7d63ef509bb81f`.

Model virtual +0x1a0 (`0x1411696d0`) then reads AvContext +0x478 bit 1
through `0x141110150`. If clear, it returns mode 0. If set, it retains
the requested mode except that mode 1 with no +0xe8 record becomes 0.
This resolves the initial transform of the priority result without
assuming a runtime setting or threshold value.

The initializer's priority insertion itself is conditional: mode must
be 3 and agent primary virtual +0x678 must return zero. That getter is
`0x141027720`, reading whole-agent +0xe8 bit 6, which comes from server
37 field 8. When that bit is set, this initializer skips priority insertion
and leaves the existing mode for its later branch tests. This is an
observed input consequence, not a recommended flag or proof that its
other consumers permit a desired bootstrap configuration.

The priority manager also has a later update/sort path at `0x141159720`
and `0x141159da0`. Only its opening was inspected here; the initial mode
is not a promise that a model keeps that mode. No live graphics settings,
priority limits or model-load success were read or changed.

Private policy bundle: `~/re/gw2/notes/209026-character-model-policy-1.json`
(1,584 records; 30,646,119 bytes).

| REA evidence | Observation |
| --- | --- |
| `ev_d7f60381f7b62e76488148db9bcac8be265c4dd045542930e1b429e6537d0cc0` | Complete initial priority insertion and context bit getter. |
| `ev_2597a807034d6ea04f2dcd2cf79f4d34f43bc18cf3f6e09195ba316b5a42d608` | Model-parent and character-policy virtual entries. |
| `ev_75881778c3ea21888f7c16911616a746eac4aa9f35fdd5ff7adece0b26209695` | Parent getter and complete character predicate. |
| `ev_7a4ef421d96e7c2609e54036a778034dad129ed805a725965f6d8a9e8be3f75d` | Complete map-flag predicate. |
| `ev_0885cdb7ea511b139f71106dcb63481a0d304c6cf9a317967f3b65c111aa0395` | Current map-definition accessor. |
| `ev_495dc8d228b2c42d5e24f0b248a0748c7aba70af471f2d51ee81c54738f6dfd2` | Hash-checked Queensdale record and mask evaluation. |
| `ev_eadcccabc0268ad697574ab9ab6f708a41838bdec8ced8277d012e0fff648fab` | Agent +0x678 getter entry. |
| `ev_10b2ffc7131db18508110256625604f1e48b363497c81d96e8fad67663f90ccf` | Agent bit-6 getter and bounded later sorting path. |

## Threshold construction and fallback loading

Priority-manager constructor `0x1411595b0` initializes selectors +0x30
and +0x34 to 2, then calls `0x14115a2e0`. That setter uses these literal
arrays, indexed by the indicated selector:

| Array | Values at indices 0–4 | Consumer |
| --- | --- | --- |
| A | 30, 60, 100, 200, 300 | Selector +0x30 |
| B | 0, 30, 60, 100, 300 | Selector +0x34 |
| C | 10, 15, 30, 30, 30 | Selector +0x30 |

It writes manager +0x20 = C, +0x24 = min(B, A), +0x28 = A − +0x24,
and +0x2c = 0xffffffff. Thus the constructor produces +0x20 = 30,
+0x24 = 60 and +0x28 = 40. The initial insertion helper compares the
index directly against +0x24 and then +0x28; these are not cumulative
comparisons. With those constructor values, insertion indices 0–59
select mode 0 and later indices select mode 2 before the character and
context overrides. This is a conditional calculation, not a claim about
the settings present when a character arrives.

Setters `0x141159d80` and `0x141159d90` replace the respective selectors
and recalculate only when changed. Wrappers `0x141111920` and
`0x141111930` reach them at AvContext +0xc8. No index bounds checks occur
in these small setters; their upstream callers and the settings' names
remain unresolved. Later priority updates can change assigned modes.

The no-record loading helper's remaining body distinguishes a usable
current model from a replacement attempt. A nonnull model +0x40 whose
virtual +0x540(0) returns nonzero bypasses replacement. Otherwise a
nonnull supplied old-model pointer is restored, retaining the attempted
model at +0x48. With no old model, mode 3 skips the fallback call; other
modes reach `0x141168b00`.

That complete fallback helper (`0x141168b00`–`0x141168d29`) returns
immediately when AvContext +0x478 bit 1 is clear. It also returns when
the current model passes the same virtual +0x540 check. Otherwise it
obtains a view scale and transform, then tests model +0xe8. Only a
nonnull +0xe8 record reaches loading call `0x140cf7000`, using record
+0x28. A nonnull result becomes model +0x40, with the previous current
model retained at +0x48, and sets whole-view +0xb78 mask 0x80. Both an
absent record and a null load result instead reach model virtual +0x120
without installing a replacement here. That callback and the record's
population are separate dependencies; this helper alone does not ensure
that a view has a loaded model.

The no-record helper later repeats the wardrobe/inventory helper
`0x141165d60` when model +0xe0 is nonnull and view +0x08 virtual +0x1f0
returns zero. This is an additional conditional call site, consistent
with the earlier wardrobe prerequisite; it does not establish that the
initial builder is the only consumer.

Private bundle: `~/re/gw2/notes/209026-character-model-fallback-1.json`
(1,592 records; 30,739,795 bytes).

| REA evidence | Observation |
| --- | --- |
| `ev_55fb738f0a38a53ccbf3221776680e1db3c32edb5a12622715a818b46e2e3d19` | Three threshold-setter relative-transfer candidates, validated in the following bodies. |
| `ev_729528fab12fb16d497aad6c956bf2fb53821556e166e5dad876c9404e76bd50` | Constructor defaults, selector setters, fallback gates and load call. |
| `ev_35bc7aaf55cb9fce51f70fae3e15ac5be109d15c16ac9834d01031931fdaf5fa` | Complete threshold calculation and remaining fallback body. |
| `ev_e974eee90406425c15be91aa42268d91edac27099d41aecbaeddf84d05337c8c` | Relative-transfer candidates for the selector setters. |
| `ev_2ab5a4b7ef9d105049ba95763f6398f8f40ff996a554e14bc845ef8f317ceed7` | AvContext wrappers and no-record helper's replacement decisions. |
| `ev_504ed77c77d67fc1111b7fade9c26e22a607077b9f6f13d0e3fdced34933b6f6` | Remaining no-record helper, including second wardrobe/inventory call. |

## Evidence

Private bundle: `~/re/gw2/notes/209026-character-model-inputs-1.json`
(1,567 records; 30,523,023 bytes), frozen executable SHA-256
`99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`.

| REA evidence | Observation |
| --- | --- |
| `ev_9cdcc78dc19d772b1ab74b7c2eb6e36ac8344f3bb6a959d6984ed907b29e2be2` | Model +0x38 initialization entry. |
| `ev_417e937d0f9eeaf04fa552ebe7766ecd31db0dd86255a1bd2fbf9d44e4465d4b` | Complete initializer and branch map. |
| `ev_80e673d0b2f5ce003ca47c27568bef4111ca94b55fc5372284b391d1aaa48ef8` | View getters and character flag getter entries. |
| `ev_8ff74ad31e9f88546fe011617497b1bb158fd581d9bbcf718fbfaa9b7b01f353` | Character/record getter wrappers, classification call and flag getters. |
| `ev_205435a6d7db535b473f3f18a0e2aa62e52a6797a3b98966e3d7a580d2c26e42` | Classification and record getter virtual entries. |
| `ev_59a305752e229bfd5a6286bf514e8a445eb7b0eb336ff55c8a52c8b988474037` | Complete classification, record getter and loading-helper opening. |
| `ev_fb21b1f2b5ceffc12a977e2a5306b49fc130eb56f103c2e03f3857a8a1f88e74` | Builder first half, coreStats appearance conversion and load calls. |
| `ev_752f937013e3f115c38e5680116f9d308b3587fcc2f96c787a846b1e0b39b919` | Remaining builder, inventory loop and mode-policy helper. |
| `ev_5d4ca2ea797517f2c0414c6086d617aff1b316601c96dbf3f3da0bbefbede916` | AvCharModel assertion source. |
| `ev_59b6a46588ee59f9b27fd1679f628949275a3ae7478158f2c7940da7a6e4db35` | Base/ordinary constructors and mode/pointer initialization. |
| `ev_8ffb4dc41fdb14ce91093eb9bafa65cd41d029545408bb978a8cbc8fe114ebf6` | Mode-policy and appearance scalar getter entries. |
| `ev_886823f1ae5e9f01dc3edb1a40fa76919ea346856b057501bb2c5c8e9ff83b9b` | Inventory and optional-component getter entries. |
| `ev_319b045422e9abc13cde0e562534443264f022ddc9119b7437128868a5f3c4fc` | Optional component returns whole-character +0xb00. |
| `ev_7c811c2d2ef7d1d78012de80b6dd76e1b865626f895eec08e0ab519edcb9effb` | Complete inventory getter and initial view override record cleared. |
