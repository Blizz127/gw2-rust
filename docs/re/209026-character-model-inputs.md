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
whole-character +0xb00. The builder explicitly checks this pointer for
null and uses zero for its queried value when absent. Its component name
is not established here. It is therefore not an unconditional allocation
requirement of this inspected builder.

This connects the known **coreStats and inventory before finalization**
requirements to actual model construction. It adds no new required
packet for this path and does not establish valid appearance defaults,
all attachment dependencies, priority policy, asynchronous completion or
rendering readiness. The builder's later `0x141165d60` call and other
initialization branches remain follow-up work.

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
