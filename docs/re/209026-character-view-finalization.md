# Build 209026: finalization creates the ordinary character view

Static REA follow-up to [view reveal](209026-character-view-reveal.md).
No runtime launch or replay was performed. The existing agreement hold
was not changed.

## Finalization subscription

The static object at `0x1425ba680` has primary vtable `0x142142340`.
Its +0x10 listener uses `0x1421424d8`; both the initial data and the
vtable-restoration code at `0x14190e7d0` confirm these bases. Assertion
source `0x1421429b8` identifies `Game/AgentView/AvContext.cpp`.

AvContext method `0x14110f680` registers its +0x10 interface with the
character context through virtual +0x1b8 at `0x14110f870`. The character
context's constructor-established table `0x14215dfc8` resolves this slot
to `0x1411b62e0`, which inserts the listener into **context +0x230**.

The [previously traced server-170 finalizer](209026-character-finalization-dependencies.md)
calls `0x1411b61f0`. That dispatcher passes the character to context
+0x230 listeners through thunk `0x140255560`, virtual **+0x08**.
For AvContext +0x10, the entry is `0x1421424e0`, resolving to
`0x141110570`. This callback asserts a nonnull character, checks
`0x141111ad0`, and, when true, calls ordinary view factory `0x14110f150`
with `(whole AvContext, character, 0)`.

The prior view note established that this factory constructs the view,
registers its per-character listener, then synchronously calls its
initialization method and allocates the component used by reveal.
This resolves one substantive effect of the finalizer's context event.
It also makes finalization-before-scene-entry a supported candidate
ordering for an eligible fresh character. It is not proof of all model
dependencies, asset readiness, event scheduling or a playable bootstrap.

## Player eligibility and server 428's low flag bit

The complete `0x141111ad0` predicate behaves as follows:

1. Call character +0x08 interface virtual +0x68. Its target,
   `0x1411d9be0`, tests character identity high nibble == 0x3.
   When false, the predicate returns true immediately.
2. For a player identity, obtain the associated player through character
   +0x08 virtual +0xe0 (`0x1411d84a0`); assert it is nonnull.
3. Call player primary virtual +0x170. Player constructor `0x1411b9cc0`
   installs vtable `0x14215e9d8`; the slot resolves to `0x1411bb660`,
   which returns **player +0x100 != 3**. A true result rejects creation.
4. Otherwise call the existing-view lookup chain beginning at
   `0x141107840`; creation is admitted only when the resulting view
   pointer is absent. This chain obtains the agent identity, looks it up
   through `0x141111e90` / `0x14115d280`, then queries returned virtual
   +0x38. The underlying map implementation is not exhausted here.

Server 428's constructor writes state **2** when its final flag bit 0 is
set and **3** when clear. Therefore, for the candidate identity
0x30000001, setting that bit prevents ordinary view creation through this
finalization callback. Clearing it admits this state check, subject to
the remaining prerequisites. This is a concrete consequence of the
[spectating state trace](209026-player-spectating.md), not a name for the
numeric states or authorization to send a bootstrap fixture.

## Other factory callers remain distinct

The same AvContext listener's +0x28 entry is `0x141110620`. Its nonzero
R8 branch tails into `0x14110f380`; the zero branch calls the factory with
third argument 1 only if a view lookup is absent. The [server-585 follow-up](209026-character-view-suppression.md) resolves
this event to a finalized-character flag setter and view deletion/recreation.

The factory also has a caller at `0x14111179b`, within `0x141111730`.
It belongs to AvContext +0x20's vtable `0x142142658`, slot +0x118, and
checks the supplied player against character-context virtual +0x70.
It obtains that player's finalized controlled character through virtual
+0x78, then applies the same eligibility predicate. The corresponding
player-event producer remains to be traced. These extra paths do not
justify omitting finalization from the fresh-character sequence.

## Evidence

Private bundle: `~/re/gw2/notes/209026-character-view-finalization-1.json`
(1,537 records; 30,190,947 bytes), frozen executable SHA-256
`99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`.

| REA evidence | Observation |
| --- | --- |
| `ev_24b6d5bee76fe7f148509a49e2c24bf668d4b6fa3aad5b70b69d391c96d7cae7` | Three factory-calling callbacks' table entries. |
| `ev_fd9dadda04a838d0c2b3140e7eb3949389ef10b5607efb5ee705200d8593db1d` | Initial AvContext interface vtables. |
| `ev_0127ddd8fc630f6b360de8f556c139049ef7baaa2cec036e0f2bac939b98ba10` | Independent vtable restoration and interface offsets. |
| `ev_6965ea811f6d0e615b9eeffb4b38ab75bf75dcd4765f9ddda56e3c64cfc4096b` | Complete three callbacks and eligibility predicate. |
| `ev_0e545435963612683803b2bf11b78491230af2a9632fea3e796904b463b74707` | Character-context subscription slot. |
| `ev_c76e7120429b49f2f30a6cf964dcb0a802adea18a3251ec42d48a9e2dba9ce50` | Actual subscription, context collection insertion and finalization dispatcher. |
| `ev_14b50a962a0cf6e913b9b3fce08efd0f4d467f4cc7ac6cc23f8be97c07dca7e6` | Player-identity test, associated-player lookup and constructor-installed player table. |
| `ev_f87b7f73c7198c2f4a55db780ce370e039d6ec0ea8de8f8275110b6206dc3c59` | Player predicate/getter and listener entries. |
| `ev_d45fd2261dcee4a85581da00ff840f9d0cce2b40c360604a5cd8d5ed86d61c4b` | Complete state predicate, virtual +8 thunk and view lookup wrapper. |
| `ev_77c3b0286a202a9e58a3abfa48defce6d10374ae037305ca299c885d59dd2a4e` | Server-428 constructor state-2/state-3 selection. |
| `ev_a3c848e8cb4700c92ddfc527c52c2cbb431b3552435b0a99c2d91ec867b61669` | Identity lookup wrapper and finalized controlled-character getter. |
| `ev_7da8ff9ae75764bc2d8a7794dcc3f4735e6fc016b322f807141dacf35e9ccaeb` | AvContext assertion source. |
