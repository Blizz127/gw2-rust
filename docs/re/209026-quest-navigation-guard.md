# Build 209026: quest refresh navigation guard

Static REA follow-up, 2026-11-01, to the
[startup subscriber trace](209026-startup-subscribers.md). The previously
unresolved service query is a navigation-link count check. It does not
introduce a network request or allocate in the queried method. Runtime
service state remains unobserved because the client is stopped at the
agreement hold.

## Service and predicate

Factory registration `0x1413fa350` binds slot **0x36** to
`0x1413fa380`, a jump to **`0x1413f5c80`**. The factory allocates 0x188
bytes, installs primary vtable **`0x1422db110`** and a +8 interface
vtable **`0x1422db1f0`**. Getter `0x1413f6970` reads thread service
context **+0x1b0**. Source-path strings identify `MlCliContext.cpp` in
`Game/MapLayout/Cli`.

Primary virtual **+0x88**, table entry `0x1422db198`, resolves to
**`0x1413f66d0`**. Its complete body returns whether Dword **+0x104** is
nonzero. It has no calls or stores. The factory's zero Qword write to
+0x100 clears both the capacity and this count in the collection at +0xf0.

The quest predicate `0x141202610` calls that virtual before testing quest
+0x58. Either a false service result or a null quest field returns false.
Thus, under the previously documented unchanged quest constructor state,
its false result does not depend on the navigation count remaining zero.
This does not remove the later, separate conditional store-query branch.

## Population and startup subscription

`0x1413f6250` registers the service's +8 interface with thread context
+0x98, virtual +0x1b8, then tail-calls **`0x1413f66e0`**. The +8
interface's startup virtual +0x80, entry `0x1422db270`, is the already
identified no-op **`0x140163990`**. Registration timing relative to the
player callback is not established by this static subscription site.

The population helper counts content types **306 (0x132)** and
**307 (0x133)** through content service +0xe0, virtual +0x40. It reserves
capacity and iterates type 306 through virtual +0x48. For each definition,
it skips a null pointer at +0x28, a null pointer at +0x30, or identical
endpoint pointers. Otherwise it resolves each endpoint through
`0x1413f60f0`, constructs a link through `0x1413fc8d0`, appends a 40-byte
entry to the +0xf0 collection, and increments its +0x14 count (whole
+0x104). Definition +0x40 selects endpoint attachment; values outside
0..2 reach the inspected invalid-case path.

The final count assertions explicitly name **m_navigationLinks** and
**m_navigationNodes**. They compare whole +0x104 with the type-306 count
and whole +0x124 with the type-307 count. This establishes the queried
field's navigation-link meaning, rather than inferring it only from its
position in the object. Endpoint helpers and all other lifecycle writers
are outside this note's scope.

A hash-checked census of all 32 frozen base containers finds **17,213
type-306 records** and **11,813 type-307 records**. Twenty-nine containers
have these records and each has at least one type-306 record with two
nonnull, distinct resolved endpoints. File/hash/count details are in the
REA evidence. The census is of base content, not a reconstruction of the
runtime registry's visibility, initialization order, or any map-specific
content. It disproves an assumption that the frozen base data contains no
navigation definitions; it does not prove a particular runtime count.

## Evidence

Private bundle: `~/re/gw2/notes/209026-quest-navigation-guard-1.json`
(1,809 records). The executable and all 32 base-container reads were
hash checked. External helper source is not bound by the REA artifact
identity. Registration and branch candidates were validated by bounded
disassembly; unrelated neighboring functions are not part of the claims.
No source behavior, runtime state, agreement, account or cache changed.

| REA evidence | Observation |
| --- | --- |
| `ev_e0fa63fc939f776e9b70d977c9bac44294c89db3a44fde4267ea477a3a8d1b4b` / `ev_3e8929a5eb1628f0da1b6a3399a0b4595b938ca7c7eca99df52cfbb59c15a30b` | Slot-0x36 registration candidate and validated function. |
| `ev_b3f906939b0e9116258a4d622b968c488705d787416128746a70673cb82a8952` | Factory thunk and quest guard call order. |
| `ev_c9e43cf7a8a0cf9fbf247b5955d54d92e4d4708b440d3cbba99da802539d7c98` | Factory allocation/tables and context getter. |
| `ev_d2d486ad016a1bdfe0249448cb2e01994733b9b59e62283b54d73c54c3bff1ee` / `ev_35cb25284ebee7cafbc22ab3029603c90da0ccbc4424d8b7948d0019b0e95bff` | +0x88 slot, complete count predicate, factory clearing and population entry. |
| `ev_97d93378735194726eaf4095de0c2b8f06140a01e746629869cbddb5be263867` | Endpoint guards, link append/count increment and count assertions. |
| `ev_4806e417427e69b550884caeea8f2d6a3f460f540396d39df544e8c1ad73b472` | Map-layout source path and navigation assertion strings. |
| `ev_cfb3705d0ad891466cdc04eb045ea6c91e2a7b76a9c50dffc73cde34b169c6f9` / `ev_7713525dd884b6cfd5a597d564b489f5eda20bcc50467421f04bff3161b98c70` | Population caller candidate and validated subscription/tail call. |
| `ev_162e28bf85c7e69e2faeedfe580bafb54b0f5eae04b929036e90896d0d9b497a` | Startup interface callback is the no-op. |
| `ev_a4696aaf3ddafa7062d24c8ba68d1ecb2ae82ba727682d7a32c2ec16ed1b02d2` | All-32-container navigation census and endpoint predicates. |
| `ev_1745d5f61ea67fb91c109377ffb0418b716a1fcada40a5a076aeed53c960ce7b` | Subscription function unwind boundary. |
