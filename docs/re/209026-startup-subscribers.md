# Build 209026: bounded server-431 subscriber follow-up

Static REA follow-up to [wardrobe/startup](209026-wardrobe-startup.md).
The frozen client was not launched and the agreement hold remains intact.
This note covers selected subscribers, not a complete runtime event list.

## Registration and three embedded no-op callbacks

The character context at global-service +0x98 registers listeners through
virtual +0x1b8 (`0x1411b62e0`) into context +0x230. Server 431's initial
event calls their virtual +0x80. The previously traced AvContext callback
only clears three flags. Three additional interfaces installed in the
player constructor resolve +0x80 to the return stub **`0x140163990`**:

| Player embedded object | Constructor / registration site | Listener offset / table | +0x80 entry |
| --- | --- | --- | --- |
| +0x9700 | `0x1412094b0` / `0x141209529` | +8 / `0x1421691f8` | `0x142169278` |
| +0x9310 | `0x1412326e0` / `0x1412328ea` | +0x10 / `0x14216cfa8` | `0x14216d028` |
| +0x0618 | `0x1411e4f90`; later initialization `0x1411e68c0`, registration `0x1411e68e9` | +8 / `0x142162240` | `0x1421622c0` |

The player constructor calls the first two constructors at
`0x1411b9f99`/`0x1411b9f39`. It constructs +0x618 at `0x1411b9e83`
and invokes that object's later initialization at `0x1411ba0d0`.
These three callback bodies add no component prerequisite for this event.
Their constructors, other events and broader subsystem behavior are not
claimed to be no-ops.

## Conditional quest subscriber

Player +0x9820 is constructed by **`0x1411fd630`**, called at
`0x1411b9fc4`. Its source diagnostic names **ChCliQuest.cpp**. The
primary table is `0x142166030`; its +0x18 character-context interface
uses `0x1421661b0`. Listener +0x80 at `0x142166230` is the nontrivial
callback **`0x141200450`**.

The constructor clears quest +0x58, +0xf0, +0xf8, +0x110 and the
collection at +0xd0 (including count +0xe4). It initializes both 16-byte
values at +0x100 and +0x118 from static `0x1425be660`. Its complete body
does not register this interface with the character context.

Registration occurs conditionally in helper `0x141201540`: an already-set
+0xf0 mask 1 skips its initialization block; otherwise it performs owner
initialization and service subscriptions, and registers quest +0x18
when +0xf0 mask 8 is clear, then sets mask 8. Another path at
`0x14120291e` registers when quest refresh is waiting for the player's
owned character. Neither constructor state nor the callback address alone
proves that this listener is present at a particular server-431 event.

When invoked, `0x141200450`:

1. Asserts quest +0xf0 mask 8, unregisters this interface through context
   virtual +0x1c0, then clears that mask.
2. Calls quest helpers `0x141202810` and `0x141202bb0`.
3. Queries a global service from `0x141466a80`. If its returned object is
   nonnull and quest virtual +0x88 rejects it, it registers quest +0x20
   with another service, sets +0xf0 mask 0x10 and forwards a service
   result to a third service. Those global branches remain unexhausted.

The second helper has a useful bounded empty-state result:
`0x141202bb0` returns immediately when +0xe4 is nonzero or quest virtual
+0x70 is false. That slot resolves to **`0x1411ffd60`**, which returns
false for a null quest +0x58. Thus an unchanged constructor-null +0x58
skips this helper's deeper quest/content dereferences. This does not
prove the entire callback harmless: the preceding helper and subsequent
global-service branch still execute.

The first helper `0x141202810` calls **`0x141203060` before its own null
checks**, then compares the two 16-byte values with their static sentinel.
Its later owned-character/quest-content work is guarded. The leading
helper is followed below. No new mandatory network packet is inferred
from this partial callback trace.

## Leading quest helper with unchanged constructor state

`0x141203060` first calls predicate **`0x141202610`**. That predicate
queries global-service +0x1b0 virtual +0x88 through getter
`0x1413f6970`, then returns false if that query is false or quest +0x58
is null. A null quest therefore avoids its later owned-character and
quest-definition accesses, but does not avoid the global service query.

The caller invokes quest virtual +0x20 **even after the false predicate**.
That slot is `0x1411ff8e0`, forwarding to +0x18 (`0x1411ff3b0`) with
both optional output pointers null. For unchanged constructor state:

- Both position-like values +0x100/+0x118 equal the static sentinel at
  `0x1425be660`: three Float32 bit patterns `0x7f7fffff`, then a zero
  Dword. These are finite values, so their equality checks succeed.
- The getter queries character-context virtual +0x60 (`0x1411b4ef0`).
  It checks context +0x98, then, if present, calls that character's +8
  interface virtual +0. The ordinary interface table `0x142161250`
  resolves this to `0x140400100`, a direct read of interface +0x90
  (whole-character +0x98). It does not allocate or assert nonnull.
- A null result returns the sentinel. A nonnull result reaches the quest
  array count +0x74, which is initially zero. The constructor reserves
  capacity five but does not add entries. The zero count skips the
  per-quest selection loop and also returns the sentinel. The wrapper's
  null optional outputs skip their secondary calls.

With its predicate false, `0x141203060` then checks marker +0x128.
Constructor-null skips both marker allocation and removal and returns.
Back in `0x141202810`, the two unchanged sentinel values take its early
false-return branch before its later quest/owned-character work. The
second refresh helper's null-quest false result was established above.

This narrows **these two refresh helpers** for unchanged constructor
state, conditional on the first global service query returning normally.
It does not establish that the quest listener was registered or that the
callback's later global-service branch is empty. The latter getter
`0x141466a80` returns global-service +0x228, which the callback still
queries separately. Nonempty quest collections, existing markers and
changed position values retain broader behavior outside this empty-path
trace.

Private bundle: `~/re/gw2/notes/209026-quest-startup-empty-1.json`.
All observations are static; no quest messages or runtime calls were run.

| REA evidence | Observation |
| --- | --- |
| `ev_0b0eb6eba9245637bc64c8baecf4014beb5d2e72aafd7940509460f6c7c625a3` | Complete predicate and leading helper's false/no-marker branch. |
| `ev_dfa6b1c3f83bfa4863c9e823dddae66c8e80341e7688e79ee5403873d04d6e1c` | Position-wrapper method entry. |
| `ev_bc36bfd63ea71157a4003f53bcaeb6f81360e1229ca2c83e0612d55e5dbb84e7` | Two global-service getters. |
| `ev_a7eaee2f0b1460ef3f0a78f07fd85a2e02b33460ddb414b5aa63cad34fb992b8` | Complete +0x20 wrapper. |
| `ev_8ba7ed75cc13eb0b675d3aa304fd80199ffeab51c6c2be629047aa6c93ce1346` | Underlying +0x18 entry. |
| `ev_d2040c49c6c532690483829e9ef1e6157593a659c26fddf56e77f1036d7c1567` | Sentinel comparisons, context getter and zero-count exit. |
| `ev_32c3923c212734e14caece44723c644f5aebd44ed04e24fefb6835e198633397` | Optional-output guards and sentinel returns. |
| `ev_cafbaaa7d2e9968160bfe6d9579e3c8b03a89b78898de76e784740ce45bb429f` | Context +0x60 entry and finite sentinel bits. |
| `ev_bb2280b7270ae318c5fd0289239af7d050e25592c2ac0544d64b8794f9bc203d` | Complete nullable context getter. |
| `ev_2c944a8412f66fc4db91641a5af4ae58b35593e8b01990de018853e1097b1856` | Ordinary character interface +0 entry. |
| `ev_7057d962e2d5e1e81c940db3c2e4c4627cf9783bc7d2e7f4e2ad09edb300c51b` | Direct character field getter. |

## Coverage and evidence

A raw whole-text displacement scan found candidate indirect +0x1b8 calls;
that displacement is shared by unrelated interfaces. A narrower scan of
`0x141100000..0x141300000` validated instruction boundaries for 36 such
calls and retained 15 whose preceding 28 disassembly lines contained a
+0x98 memory operand. Those retained contexts show the global-service
getter and character-context use, but this heuristic can miss register-
indirect calls, aliases and other code regions. It is an aid to selecting
reads, not a complete subscriber census. Remaining retained candidates
include sites around `0x1411031e3`, `0x1411726be`, `0x1411dfd98`,
`0x141267c04`, `0x1412a2fc6` and `0x1412c3d0c`.

Private bundle: `~/re/gw2/notes/209026-startup-subscribers-1.json`.
No source implementation changed and no runtime smoke was performed.

| REA evidence | Observation |
| --- | --- |
| `ev_e9bc158aa7c09ed5f78a939a48a044be2857510b8ca18cfb04442c1987ab7e8e` | Raw whole-text +0x1b8 call candidates. |
| `ev_7352430ea9e17174d948a22a14f8aafe0f284539d0c93026e2be2669a4454cc8` | Bounded validated call contexts and heuristic counts. |
| `ev_180fa93595cd2ab8405d347fd18f6ff85cf261dbce3871a3c61f5aa2f825d902` | Registration and +0x9700 constructor table. |
| `ev_1e37dfa7272e9f145bc64c0b4a92ee6cc92f0f5cd5b868dcc10e1017d351759d` | +0x9310 listener table installation. |
| `ev_bacb6437fe5bf8223663d146a574a7a64a01bc5eb02399cedc79720b6df82e03` | First no-op entry. |
| `ev_c03ca07cadb9993b593bb267d7078b9de16ad5481641797245ebba70137e63fa` | Second no-op entry. |
| `ev_d1bf4387a11ffdad73fca8fa639b8421eb66a87923e834331eb1913f52d1a8bb` | Third no-op entry. |
| `ev_1a7a8b8115a2478a65eb5c49d70a92217bf6f6c7263b377946bb2dba068890c9` | Player-constructor call sites and embedded offsets. |
| `ev_12e199149f782b348c62c8156218bfc3534129d6d575a0181a3e7ff444c786b4` | Earlier constructor sites and quest table/null pointer initialization. |
| `ev_6ed6effcaf1c136a83b126e95746df0a350441e356cc65c930d98b62b35051e6` | +0x618 listener table and return-stub body. |
| `ev_ec6ef4e704506566058da307f143d440eee73e1e4fb892ce079d48767c52e267` | Quest callback slot. |
| `ev_223b90f0227d91e919be80c40b136cac4426680351348b22b76a947924d8e6b4` | Complete quest callback body. |
| `ev_916174a076d207808c6597229ffb79566c16d67573477fbcd15912f44676b7dc` | Quest source diagnostic. |
| `ev_d841a69a847e8ea7c51fd39fba90fca1cab970dd1fec040f2a3e215858504eff` | Refresh guards and leading helper call. |
| `ev_8f992bce1a6d3c54e55c8904e6ea6dd7e9d5f2c1e907ee53c5954def4afdada1` | Quest +0x70 entry. |
| `ev_639c507326ba903aa2d6be5e79c609663c2742862156ab42d2e9e2676c11672b` | Remaining constructor, leading-helper entry and +0x70 null branch. |
| `ev_26c64ed53c27a231cacd5c286b7d829c9c385a74ef52d3d48f9cf3b2b6f96bbb` | Null-quest false return, refresh early-return epilogue and conditional registration. |
