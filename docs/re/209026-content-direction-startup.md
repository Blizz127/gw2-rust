# Build 209026: content-direction startup and quest report

Static REA trace, 2026-11-01, following the remaining
[startup subscribers](209026-startup-subscribers.md). A content-direction
listener requires the quest manager to have been reported. **Server 538**
is a one-player-index initialization message that establishes that bit.
This is a dependency for the inspected callback, not a complete validated
bootstrap. The client and agreement hold remain untouched.

## Callback registration and direct behavior

Constructor **`0x141267710`** installs primary vtable `0x142172128` and
context-interface vtable **`0x142172268`** at whole object +0x60. At
`0x141267c04`, it registers that interface with thread service context
+0x98, virtual +0x1b8. The interface's startup +0x80 entry at
`0x1421722e8` is **`0x14126fab0`**. A diagnostic source path names
**ChCliContentDirection.cpp**.

The callback adjusts the interface pointer by -0x60 and:

1. Calls configuration refresh `0x14126ebf0` (bounded below).
2. Clears whole-object flags +0x160 masks 4 and 8.
3. If mask 1 is clear, obtains the current player through context +0x70,
   asserts it exists, gets player virtual +0x3c8, and subscribes its +0x48
   listener through that result's +0x60 when nonnull. It then sets mask 1.
4. If mask 2 is clear, subscribes its +0x50 listener through the service
   returned by `0x1414443a0`, virtual +0x60, then sets mask 2.
5. Registers its +0x90 listener through `0x1409720c0`, virtual +0xd8.
6. If whole +0x1f8 is null, obtains/asserts the current player, calls
   player virtual +0x358, asserts the result, subscribes its +0x80
   listener through that result's +0xb8, then caches it in +0x1f8.
7. Tail-calls `0x141023870` on whole +8 with arguments `(1, 1000, 0)`.
   The scheduler's eventual callback is outside this trace.

The constructor clears +0x160 and +0x1f8, so their subscription branches
are reached if that state remains unchanged. Other registration bodies,
listener notifications and the full configuration path remain open.

## Quest manager is required to be reported

Player vtable `0x14215e9d8` resolves +0x3c8 to `0x1411bb420`, a direct
return of embedded player **+0x9df8**. Its constructor is called at
`0x1411ba02b` as `0x14123e0c0`; its listener implementation is not yet
traced here.

Player +0x358 resolves to **`0x1411bb310`**. This calls player +0x168
and asserts the result before returning embedded player **+0x9820**, the
quest manager. The assertion at `0x14215ee88` says
**`IsQuestMgrReported()`**. The predicate is **`0x1411bb650`**, returning
player Dword +0x9910 mask 1, exactly quest +0xf0 mask 1. This is a
quest-report check, not an ownership check.

Quest virtual +0xb8 is **`0x141200130`**. It independently asserts that
same initialized bit, a nonnull listener and absence of a duplicate,
then inserts the listener into the list rooted at quest +0x28. Its
complete inspected body does not immediately call the listener.

The resulting ordering constraint is: report the current player's quest
manager **before server 431 reaches this registered startup callback**.
It need not contain an active quest to satisfy the report bit. The prior
empty-quest refresh guards still apply only while their fields retain the
stated empty state; this does not bypass the conditional store query.

## Server 538

Registration pair **`0x1421704c0`** contains descriptor
**`0x1425d04b0`** and handler **`0x141258c30`**. The terminated descriptor
has Header **538**, followed by exactly one **Dword player index**.
The established wire encodings are little-endian u16 for the header and
unsigned varint for the Dword. Example player 300: `1a 02 ac 02`.

The complete handler reads the decoded index at packet +2, resolves it
through context virtual +0x118, asserts a player exists, and calls
**`0x141201540`** on embedded player +0x9820. It returns 1.
The initializer returns immediately if quest +0xf0 mask 1 is already set.
Otherwise it sets that bit, invokes player helper `0x1411bbb10`, registers
quest +8 with another service when mask 2 is clear, then registers quest
+0x18 with the character context when mask 8 is clear and sets those bits.

`0x1411bbb10` supplies player list +0x90 and trampoline `0x14026e8b0`
to `0x14026dce0`; the trampoline calls listener virtual +0xf8. That
notification's full listener set and dispatch implementation are not
exhausted here. Consequently this note does not claim that the report is
side-effect-free or safe at every earlier point. The player must already
exist; no synthetic player index or complete runtime packet sequence was
validated.

## Leading configuration refresh: bounded findings

`0x14126ebf0` first clears whole +0x164 and reads content service +0xe0,
virtual +0x1b0. A nonnull result supplies overrides at +0x30/count +0x38,
a default pointer +0x28, flags +0x40 and a float +0x48. It considers string
selection and validates a selected definition's +0x38 through
`0x141269d90`; failure can clear whole +0x168.

A null service result clears flag 0x20 and still reaches a fallback:
when +0x168 is null and built-in descriptor **108 (0x6c)** exists, it
resolves expected type **159 (0x9f)**, stores the definition, and uses the
same validation helper. If +0x164 remains zero and a definition remains,
it copies definition +0x28 into that field. Concrete selected content,
the validation helper and the service's source are not resolved here.
An empty quest object does not eliminate this leading configuration work.

## Source support and validation

`crates/gw2-proto/src/quest.rs` provides
`QuestManagerReportFrame209026` with the single player-index field.
Encoding appends one frame; decoding consumes one complete frame and
preserves the input on failure. The module does not select a player,
validate its client state, or send a packet.

`cargo test -p gw2-proto --lib quest_report_` passed **2/2** focused tests:
literal wire fixtures for zero, 300 and u32::MAX, concatenated frame
boundaries, every truncated prefix of the longest fixture, neighboring
message IDs and overflowing/unterminated varints. These are framing
checks; zero is not asserted to be a valid runtime player index.
`cargo build -p gw2-proto --lib` passed. No client startup or feature
smoke was run while the agreement hold remains in place.

## Evidence

Private bundle: `~/re/gw2/notes/209026-content-direction-startup-1.json`
(1,831 records). Executable SHA-256:
`99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`.
Byte-reference candidates were validated with bounded disassembly.
Unaligned leading bytes in exploratory ranges are not used. REA binds
the artifact, not the external helper source. No client replay occurred.

| REA evidence | Observation |
| --- | --- |
| `ev_34902bed9175252c8765e8342886ba8539fd0095cd855eb8e55e52fa26135c73` / `ev_5d46c114d4676268486330ee2aa71714efa610ca8bb4f6fff2e16e9e0c46ad57` | Six remaining subscription candidates and containing unwind ranges; five remain unclassified. |
| `ev_11f1c19a1bdf3d6567a1d3789f3c332c0300b6a4e2f5e3b934cc54410954f2d8` / `ev_9207200d255c4b343661c4afaaca9aeeec5ec0f92681fbb102fce9c21070641c` | Constructor interface and startup callback entry. |
| `ev_ef24d8ee284e0d6d81258d06b02304f8a293560bd7dabc9bfd44ec3a700115e1` / `ev_d66743de2960a6d34877387ddb85e8fd3319097024c5be09cb48d7722deefedd` | Complete callback branches and leading configuration entry. |
| `ev_3a61d502c298225e8dd0fdeb95bd115c08365b484459d536af52ea4497875f40` | Content-direction source path. |
| `ev_46c6eb5cc4e0d65434cc75dadda591712d6cb8f951433f491367e1d154d989ec` / `ev_a1a4e1705c6cbb083081e805749f3dfe1c6f9aaedf00a5270396a25e401b0d0b` | Player slots, embedded +0x9df8 getter and configuration fallback/tail. |
| `ev_d2548b7d82c1dd134c1b85c354ab4836ce5907ecbed948d5bd166a768d875d39` | Quest getter assertion, embedded constructors and callback flag clearing in constructor. |
| `ev_80a93b330756d6f14b140ef19cd0cd99c8ac8b56d1cec0dd3deea9b2b00c92c3` / `ev_d5ca1ea8af8aad9cedd15427a839fe6004ccaa08fdef92256bca49ba8a819ae7` | Report predicate slot/body, quest subscription entry and configuration selection body. |
| `ev_81d872047db79bde8b86514aa7646229765d91e2b4f7919a5d142c367af64171` | IsQuestMgrReported assertion bytes. |
| `ev_67963ae3f1a560b86179099e4e8f2a491729055777e24ec883920bc6bf6f3dca` / `ev_0941e22153e9656689d3651dc9564cb34b25c8f06e7737255e7033a6022e91a6` / `ev_9d3556380627bb699209c89aa51882447c2a8913868f295d175de9ff81961a0b` | Initializer caller candidate, validated call and handler range. |
| `ev_a818d85d433bb7becf8bca5b97895b0dc36d3889b0e2c2b6716faf7b11812a7a` / `ev_0106d85f1bb6316a9ec986eec01760c6bea8a2c20d5662540b6bc17c4526fdbb` | Empty RIP candidate search and actual descriptor/handler pointer pair. |
| `ev_881e2c382aad4647f0dc24100bfd28f321e5fe114a60f39b3dc30062b1e56b94` | Terminated 538 descriptor. |
| `ev_eae37e1e711ba8aabf66cfd5c0e4597817093545843e2e1b3667cb3e8330c486` | Complete handler and initializer, quest subscription tail. |
| `ev_30112db1af59c6f0ce5b2e72d65ac5463a15cfc7c6999f9d38fbd60ce4a95997` / `ev_7b7c34c1f18db7baf8b4883679673eb0d446af544b8e23ae7f14932e097150f2` | Constructor clears cached quest, player notification wrapper/trampoline and dispatch entry. |

## Configuration follow-up: fallback and override guards

Additional static REA work resolves the fallback and the base-content
alternatives. It does not identify which override record is visible in
Queensdale's runtime registry.

The built-in table at `0x1425e6980` has 72-byte entries. Descriptor 108
is at **`0x1425e87e0`**; initialization at `0x14020df25` stores expected
type 159 and GUID Dwords:

`eca465a4 42b29591 47ff568c 5b755c0e`

All 32 hash-checked base containers contain exactly one matching setting:
file **1282855**, content offset **0x224b8**, type 150 with declared and
variant type 159. Its target is file **1282836**, offset **0x2fe50**, type
159. Target +0x38 is **null**, and Float32 +0x28 is **0.699999988079071**.
The type-159 variant converter `0x1412e3910` returns the +8 pointer only
when its tag equals 159. The source hashes are respectively
`9f9226f71e3871066237d560e95b62ef33c847818a7bd78cea586507f492cfa6`
and `ac503e07b02c76ca0b1bbac4181e598c4a4a30260f49759db51b6ef1972b3f80`.

Validation helper **`0x141269d90` returns true immediately for a null
filter**, before its character lookup. Thus this specific fallback's
validation avoids the character/coreStats accesses. Its Float32 can fill
whole +0x164 when the caller's preceding value remains zero. This value
has not been assigned a timing or gameplay name.

The preceding content-service query is also resolved. Slot 0x1c
registration at `0x14021b4d0` binds factory `0x1412e19e0`, which constructs
through `0x1412e1110` with primary vtable **`0x14220e160`**. Virtual
+0x1b0, entry `0x14220e310`, is **`0x1412e1d00`**. It asks
`0x1412e6360` for type **268 (0x10c)** with cursor zero. The iterator
searches two registry pointers in order, returning the next type-array
entry and updating the cursor; it returns null when neither has an entry.
The getter's first available record need not be the first record in an
all-container offline census.

The census finds **16 type-268 records** in 13 base containers:

| Default pointer | Override count | Flags +0x40 | Records |
| --- | --- | --- | --- |
| Null | 0 | 0 | 10 |
| Null | 1 | 2 | 4 |
| Null | 1 | 0 | 1 |
| Nonnull | 0 | 0 | 1 |

All sixteen have zero Float32 +0x48. The sole nonnull default points to
type 159 in file 1282859 at 0x3acc30, also with a null filter. Each of the
five overrides points to a type-159 definition with Float32 +0x28 about
0.7 and a nonnull +0x38 filter containing four signed Dwords
**`[5, 0, 9, 0]`**. Their parent/target/filter offsets are:

| File | Type-268 parent | Type-159 target | Filter |
| --- | --- | --- | --- |
| 1282833 | 0x664fa0 | 0x664ab0 | 0x664f70 |
| 1282843 | 0x7c4dc0 | 0x7c4e58 | 0x7c5390 |
| 1282852 | 0x9a1e60 | 0x9a1ef8 | 0x9a2430 |
| 1282857 | 0x8842c8 | 0x883ce8 | 0x884298 |
| 1282861 | 0x8d95b8 | 0x8d9650 | 0x8d9b10 |

The startup refresh passes null as the validator's optional third
argument. With one of these nonnull filters, validation obtains context
virtual +0x98 (`0x1411b51b0`), which uses current player +0xa0 and its
virtual +0x80 (`0x1411bada0`). That getter returns player +0x20 only when
the owned character exists and its +0x178 mask 0x10 is set (finalized).
The validator asserts the returned character, then obtains/asserts
coreStats through character +8 virtual +0xb0 (`0x1411d75f0`, whole
+0x388). It calls coreStats virtual 0 with argument zero: the previously
identified [earned-level getter](209026-character-core-stats.md#earned-level-consumer),
whose player-state override can return 80.

For filter selector 5, jump-table entry at `0x14126a06c` is RVA 0x1269fe4.
The supplied nonnegative margin is zero, so the reached comparison accepts
an earned-level getter result **at most 9**. A stored wire level alone
does not prove this result because the getter can override it. The
character and coreStats are required before the comparison even when it
ultimately fails. This reinforces existing finalized-owned-character and
coreStats dependencies for the selected override path; it does not add a
new message ID or validate a complete fixture.

Private follow-up bundle:
`~/re/gw2/notes/209026-content-direction-config-1.json` (1,850 records).
All configuration and census reads are static; no additional source tests
or build were needed for this documentation-only follow-up.

| REA evidence | Observation |
| --- | --- |
| `ev_cd89dff601a79e3e30e2c85356017e77b0c4561f3b106dfe1a0f7e7964d23d0b` / `ev_8a8ecb569e46f5e7dbb8830b0868551c5a21a881e64ce58a046eb86837a74e62` | Validator null/nonnull branches, complete comparison/return and descriptor stride. |
| `ev_c0e8567ad7db1cb3daabf8901264b27303d4dd84d7529061552166c4fca4757a` / `ev_c8c8e3b2c7897b7bfd1d793ea134da5702adbfe70339a9fcd12bca83acad9c56` / `ev_5eed69ef3cf93fd10c2f050eefb00ef071117cc92c16969d66f219ad1d1d4593` | Descriptor-108 initialization candidate, validated GUID/type and variant converter. |
| `ev_658e61b64f73f766aabe04248156dbd823e28f9c92bc496c8710197b30b0b9a7` | Unique fallback setting/target and null filter in all 32 containers. |
| `ev_1476d33ce1409e64cbc5fba6152e0b842d35f9aceed2ce84db084fdaab6d3d3b` / `ev_80600df2b056f15ad58c83796557ac6a31cecdde8331f055a2119c673a3efbff` / `ev_e3eec06b36b7de86e2ebb97f0bbf43cf5198a32d7ee27b1c5bf2d545a0ae4f70` | Content-service registration/factory. |
| `ev_f21ca86b3dbeb7505b3f651aed142bbf047a9d40be49695fe1ac4e54a89f51b6` / `ev_d9863211aa860260cabd5b63a1ba545659e43e563329fabe4382a04737b7a825` / `ev_4700ed41fea1467e362a60c1794a61e1e03de6b5d0c19daae81d520c4b9b9533` | Content-service vtable and type-268 getter. |
| `ev_167b612959d17f4003e073cb024dbd9f19e0b0b50ebf3c6e6f208ae37aca6cf0` | Complete two-registry iterator. |
| `ev_ee7fc4e91d623875b50b557359ddcb21f75609e73b5fe16dbec243a72f520162` / `ev_24deeeb6645f95384cc807533e6b6028b3488a7dd35a437275242812193348f1` | All-32 type-268 census and all five override targets/filters. |
| `ev_6435b8a171f264bd8382413dbe768a0cf9d0d9da97c15ee23b307ca6d33ba529` / `ev_cc3bc4e87b8480267bd86706266028b972d53569648fe0c28ff00aeeeb01e037` | Context/coreStats slots, selector-5 jump target and getter bodies. |
| `ev_374c0c14fc5ca0cacc38d32198b6509434b262b6f0c58866b0f3a19741e216f5` / `ev_cd6f50a8778bdcb673fdd322e5f9a06ace788ea47c8080873aebfefedaaeb4db` | Player +0x80 slot, finalized-owned-character guard and coreStats return. |
