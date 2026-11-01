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
+0xf0 mask 1 skips its initialization block; otherwise it notifies player listeners
and performs service subscriptions, and registers quest +0x18
when +0xf0 mask 8 is clear, then sets mask 8. Another path at
`0x14120291e` registers when quest refresh is waiting for the player's
owned character. Neither constructor state nor the callback address alone
proves that this listener is present at a particular server-431 event.
The [content-direction follow-up](209026-content-direction-startup.md)
identifies server 538 as the report/initialization message and establishes
a reported-quest prerequisite for another registered startup listener.

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
The [navigation-guard follow-up](209026-quest-navigation-guard.md) resolves
that query to a direct navigation-link count check. Its constructor-zero
count can be populated from content; it is not assumed permanently zero.

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

## Final quest callback: built-in content and category gates

The service getters above use a **thread-local service context**, not a
single process-global object: `0x1409b1fb0` reads the TLS index at
`0x1428155c0`, the TLS block from `gs:[0x58]`, then its +0x10 pointer.
`0x1409b1fd0` sets that pointer. The context builder `0x1409b1c00`
allocates 0x318 bytes and, unless input flag 2 is set, walks 0x62 factory
slots at `0x142680f60`, storing each returned pointer at the corresponding
8-byte context offset. It temporarily installs the new context while
calling those factories, then restores the previous context.

Registration `0x141466b80` supplies factory thunk `0x141466bb0` at slot
**0x45**, giving context offset **0x228**. The thunk reaches
`0x141466630`, which allocates 0x48 bytes and installs vtable
`0x1422eadf8`. Its first method is `0x141466980`, the method called by the
quest startup callback. This is a static registration/construction chain;
actual registration execution and allocation success were not observed.
The earlier direct-displacement store scans did not establish an installer
because this builder writes through a moving slot pointer.

That first method tests built-in descriptor **398 (0x18e)** with
`0x1412ddbb0`, then resolves expected content type **68 (0x44)** using
`0x1412dde50`. If the resolver reports an indirect variant, it reads the
variant at returned record +0x30 through `0x1412e3dc0`; that getter returns
variant +8 only for type 68. Missing content returns null. See
[descriptor resolution](209026-skill-slot-levels.md) for the shared
existence/cache machinery; 398 is a descriptor index, not a packet ID.

The descriptor initializer at `0x14021363f` writes type 68 and GUID
Dwords `224d61df 4c271210 ae8fefaa 5eabe662` into descriptor
`0x1425ed970`. A hash-checked census of all 32 frozen base containers finds
exactly one matching record:

| Item | Archive file | Content offset | Result |
| --- | ---: | ---: | --- |
| Setting | 1282832 | 0x37d10 | Type 150; declared and embedded variant types both 68. |
| Variant target | 1282852 | 0x139e40 | Type 68, data key 156. |
| Target +0x38 reference | 1282850 | 0x126540 | Type 389; Dword +0x28 mask **0x20**. |
| Target +0x60 reference | — | — | Null. |

Source SHA-256 values:

- 1282832: `63bd50638f1386c9d92ecdecf9c9260e02500d1d9ac616e33883577033c387de`
- 1282852: `54de38b4b7c4fdf69c18154fb254a3f5f346c1b031b68a8416153c4f751ac5d3`
- 1282850: `7c4267d7cba4fc5878a66512f207b53f4abb69f86e3a49102d9339475b79e4e4`

Quest primary +0x88 is wrapper `0x1411ffe80`, which supplies R8=0 and
forwards to +0x80 (`0x1411ffe90`). For category indices 0 through 5, the
latter calls `0x1411c8e80` on the supplied definition. That helper maps the
index through the table rooted at `0x1428a7ef8`, then tests the mapped mask
against the referenced definition's +0x28. A matching category requires
`0x1410f5310(index)` to return true; otherwise the quest predicate returns
false immediately. The mapping follow-up below identifies mask 0x20 as category index 4;
its gameplay name remains unresolved.

After all six category checks pass, the predicate checks definition
+0x60. For this frozen target that reference is null, so it returns true
without the player progress-manager lookup. If any category check fails,
the outer startup callback instead subscribes quest +0x20 through service
context **+0xc8**, sets quest +0xf0 mask 0x10, queries that service's
virtual +0x48 with the definition, and passes its result to another
service. The downstream methods are traced further below.

`0x1410f5310` bounds-checks index <6 and forwards to `0x1410f71b0`.
The inspected body obtains context **+0x68**, enumerates values through
`0x1412dffd0`, searches a 12-byte-entry collection at service +0x10/count
+0x1c, and calls `0x14023ceb0` when the key is missing. The follow-up below resolves this category
and the missing-entry behavior. No runtime account state, preferences or
credential caches were accessed. The archived definition is present;
neither its presence nor an empty quest object proves that this callback
will skip the category-dependent subscription branch.

Private bundle: `~/re/gw2/notes/209026-quest-startup-constant-1.json`.
Two initial helper calls omitted the required `0x` argument prefixes and
failed before disassembly; corrected calls supply the evidence below.
Unaligned edge bytes in the broad initializer read are not used.

| REA evidence | Observation |
| --- | --- |
| `ev_5c3ebd515e150bfdfcd7dc58fb24dd63c3dd1edcacd892d3a465dc784bc22744` | TLS context getter and setter. |
| `ev_4a8d022161ce8746a1f6c2164bc6ba4824ae0e59d92c60e0bcd74bbf7eed3a8a` | Context builder, registration arrays, slot-0x45 registration and descriptor-398 query. |
| `ev_8ef686c491c1ff87d1d05d29cfb0d6ca54978fe4693e46cb0d1a771201bfc47f` | Factory allocation and installed vtable. |
| `ev_07c446227011d94b628827c64e3638da096b98f54a364f685c412df3666693a1` | Service first-method entry. |
| `ev_107fd1c4f4244a08a0d1b346b98e9807bac9da2b7426c111e238f79daa12459b` | Initializer store candidates. |
| `ev_187dabde7b99585640a29bbbc59314cdeb3c0cc9ad8b2273ee4e75be6e9a0adf` | Descriptor initializer and type-68 variant getter. |
| `ev_5c4af74b3f60094c2ffdb1448e73535d2b3e0d4afcd99f6799e1576f754ed428` | Full base-container GUID census and resolved target. |
| `ev_a8fbadef7da544b32291919af1ba5888042955baab29f33fa10590c6991b716f` / `ev_e55fc4fdf1b909e96252c2d72bca154a74208ee4e65dc35018e78742abd575a7` | Quest +0x88/+0x80 method entries. |
| `ev_9cf778cb8069f14ba30d6ebe313a78babeba178d83eaa733150ff8a13a694597` | Predicate wrapper, initial category checks and outer callback branch. |
| `ev_c9b7f128121d6d8fa68b59a278db31daa4663e8a133cfca6c6105c41e1eed0a9` | Remaining category checks, progress guard and mapping helper. |
| `ev_65977a2d110a879bf30e68b05f0644114d440dc5bebf0e1fd7a5fc1520076466` | Frozen guard references and mask 0x20. |
| `ev_e16d84d750da0556c62c36c6b1ea4ca580ef01181056a3e9e7cb927f8bc86631` | Mapping-mask test and category-value wrapper. |
| `ev_be3b2a713b446dd2bfec745fb7dcad4c6beefc0a86b8e412d7ea88ce10f92929` | Bounded category-value search and missing-entry fallback. |

## Category 4: stored value and fallback precedence

Initializer `0x1402097c0` constructs the category-mask map from six pairs
copied from `0x142160440`, `0x142160490` and `0x1421604a0`. The collection
constructor `0x1403c00c0` copies each pair's first Dword into its key and
second Dword into its value:

| Category index | Mask |
| ---: | ---: |
| 0 | 0x02 |
| 1 | 0x04 |
| 2 | 0x08 |
| 3 | 0x10 |
| **4** | **0x20** |
| 5 | 0x40 |

For the frozen type-68 key 156 reference, only category 4 therefore
reaches the value query, assuming normal map initialization. The iterator
`0x1412dffd0` uses jump-table entry `0x1412e0298` to select
`0x1412e01b4` for category 4. Its first call writes **78 (0x4e)** and
returns true; subsequent calls clear the output and return false.
Converter `0x1412e03a0` maps ID 78 through entry `0x1412e0690` to
`0x1412e0508`, returning **59 (0x3b)**.

The complete category-value path now distinguishes these cases:

- If context service +0x68 contains key 59 in its 12-byte-entry collection,
  the entry's Dword +4 determines the result: nonzero is true. A present
  zero value does **not** consult the fallback.
- If key 59 is absent, `0x14023ceb0(78)` searches the process-memory array
  at `0x14266c008`, count `0x14266c014`, with 16-byte entries. It compares
  entry Word +0 to 78 and returns Word +2 on a match, otherwise zero.
- Because this category enumerates only ID 78, a false result exhausts the
  iterator and returns false; there is no second category-4 value.

Thus, after the empty refresh paths and successful content resolution,
this callback skips its final subscription when the category-4 value is
true. A false value enters the service +0xc8 subscription/query branch.
Neither the stored collection nor the fallback array's runtime contents
were read or changed. Their initialization/population remains a separate question; the downstream
subscription methods are traced below. This trace does not
choose or manufacture account state.

Private bundle: `~/re/gw2/notes/209026-quest-startup-category-1.json`.
No runtime replay or source change was made.

| REA evidence | Observation |
| --- | --- |
| `ev_a3241db03679697281e67554fe6eb09efcce3b68405940c5711e62943ddd5d02` | Query true/false returns, complete fallback lookup and category iterator dispatch. |
| `ev_04aa7bf485ce59da6648650cfe680b211f4dd6f1b4ffb0edde2eeb13dfaa9fd4` | Category-map initializer candidates. |
| `ev_2a9c111e09598d07d70f6a77f6ae7dc2200d417d4df37d5df586bdc54938e279` | Initializer source blocks and destination map. |
| `ev_9969d67b06fa213cd82ef81aa40b73c938f49e2785d3df583e2a2bd79c51652d` | Six key/mask pairs and category jump-table entries. |
| `ev_8f9bc5ba8719cfc4ca0ff23979efb32c72ee2f1512756186974f4728429b6ac7` | Map constructor call, category-4 iterator body and value-key converter dispatch. |
| `ev_c4d9175cec4782366dcfc28e2efd5e47a08f56cf0e99f1858a83453353f79273` | ID-78 converter jump-table entry. |
| `ev_78e14ab34d52c014d5a12a7a7a10c0ef606713e868bda1e8e8f9b5867067d484` | Key-59 return and pair-map constructor input. |
| `ev_5b2f8218c00afc891517ff8e882354c6d395ec75955b47393c642d3cdca41bae` | Pair-to-map key/value copies and exhausted iterator return. |

## Category-failure subscription and query

Context service +0xc8 is factory slot **0x19**, registered by
`0x14020b7a0` with thunk `0x1412cfbc0` to `0x1412cf970`. The factory
allocates 0xc8 bytes and installs primary vtable `0x1421857a8`. It clears
listener head/tail at +0x20/+0x28, several lookup collections, and flags
+0xa8. Its tail calls `0x14110e8f0` with a callback fixture; that additional
constructor behavior has not been traced. Initial field writes alone do
not prove the later runtime state.

The quest callback uses these primary methods:

| Slot | Method | Inspected behavior |
| --- | --- | --- |
| +0xd8 | `0x1402efc70` | Search listener pointers; add a list node only when absent. |
| +0x48 | `0x1412d04a0` | Resolve supplied GUID and query another service for a numeric result. |
| +0x30 | `0x1412cfd10` | Return `(~flags_at_a8) & 1`; used as the query gate. |

The subscription method's complete body only searches and inserts into
the listener list, using node allocator `0x14025b0b0`. It does not invoke
the newly registered listener. The outer quest callback then sets quest
+0xf0 bit 0x10 before making its query.

The query returns zero when its +0x30 gate is false, or when content
service +0xe0 virtual +0x68 cannot resolve the supplied GUID. With the
constructor's zero +0xa8 flags, the gate is **true**, so constructor state
does not itself select that zero return. For resolved type **35 or 68**,
the method directly copies the input GUID; other types have a separate
collection lookup not needed for the established type-68 target. A GUID
unequal to the sentinel at `0x141b91390` is passed to context service
**+0x1f8**, obtained through `0x140266200`, virtual **+0x1a0**. The [store-request follow-up](209026-quest-store-request.md) resolves
that method to nonzero request-ID generation and asynchronous submission. The query's returned Dword becomes
the next call's input; it should not be assumed to be zero merely because
the +0xc8 service's collections began empty.

The final consumer getter `0x1402eea80` returns static object
`0x14255a8f0`, whose image vtable is `0x14193fd30`. Virtual +0x18 is
`0x1402effc0`. Its first branch returns false immediately for input zero.
For a nonzero input it searches its +0x48 collection, skips an existing
non-null associated entry, or calls `0x1402ee5a0` and returns true. The
[tracker follow-up](209026-quest-store-tracker.md) resolves the helper to
null-valued ID insertion and identifies conditional store-error cleanup.
This proves a
zero-input no-op at this consumer, not a zero result from the preceding
query or absence of the already completed listener registration. The
follow-up establishes a nonzero ID on normal return from the reached
+0x1f8 method, so this zero-input guard does not skip that path.

Private bundle: `~/re/gw2/notes/209026-quest-startup-subscription-1.json`.
All reads were static; no client launch, service execution or state edits.

| REA evidence | Observation |
| --- | --- |
| `ev_eaedb6016d9ff85d07990fa4b4d0baf05f94be6a8b4477931eca2274525397f2` | Service factory fields and vtable installation. |
| `ev_b2d518d2a4d09424a316f5a6b1a316256eac6862c700142fa5818374b0d25ef3` | Factory tail callback setup and thunk. |
| `ev_e33fabcddcca9347e29d2a5d015ce2fd9cae230a0c7eb1acf6bb67f14223961a` | Registration address candidate. |
| `ev_1f3b4ec28ba9d89957eb98fb8868eed708b169d5eb42f0c9341ae220b78f316c` | Subscription/query method entries. |
| `ev_ac33452cbbbee3434b4cbb7b74cba3012eb9d5de3dd7448e9654cf6f70fa55f4` | Slot-0x19 registration, complete subscription and query branches. |
| `ev_711a5a9e3ed0f64249f4f05f451def5e7f176fa00f323881cc7c09ac4ab26207` | Query-gate method entry. |
| `ev_314ef5a2b6e396abc8f2a3581541a93612badf9613d38579754aca88aa17ff80` | Gate body, final consumer getter and query epilogue. |
| `ev_8f401128e3c626840b3ca3d9f45a6577a70d840f4287a29db6c3cfe31ae7fd4d` / `ev_6eafe5919b8ab8ee434e323e4f972ef6216fe2f2ab0e5b1cff515ed00262446e` | Static consumer object/table and +0x18 entry. |
| `ev_475b92744bf365097668ef04d6a13d8476d6b31cac1ae4c16a964c4f124853db` | Complete consumer zero/nonzero paths and context +0x1f8 getter. |

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

## Alert and chat-context startup slots

Two more candidates from the bounded subscription search resolve to the
same no-op `0x140163990` for this event:

| Service identification | Constructor / registration site | Registered interface / table | Startup +0x80 entry |
| --- | --- | --- | --- |
| Alert (`AlContext.cpp`) | `0x14116e320` / `0x1411726be` | Whole +0x20 / `0x14214f1c0` | `0x14214f240` |
| Chat-context table region (`CtCliContext.h`) | `0x1412a0b40` / `0x1412a2fc6` | Whole +0x28 / `0x14217e9d0` | `0x14217ea50` |

Alert initialization wrapper `0x14116f150` sets whole +0x100 mask 1
before tail-calling `0x141172610` when that bit was clear. The latter
registers its +0x20 context listener when mask 8 is clear and records that
bit. The wrapper's assertion source points directly to `AlContext.cpp`.
Its primary table is `0x14214ebc0`; resolving the registration's actual
+0x20 interface is necessary to select the startup callback.

The other service has primary table `0x14217e470`; its virtual +0x10 is
`0x1412a2f10`, which guards initialization with whole +0x628 mask 0x40.
The constructor explicitly installs the registered +0x28 interface table.
Adjacent diagnostic strings name the chat context. Neither primary table's
initialization body is treated as the +0x80 event callback.

These findings add no component requirement to **this startup event**.
They do not prove initialization harmless or classify other virtual slots.
The last three sites in the original bounded search are classified in
the follow-up below. The search covered only
`0x141100000..0x141300000` with a nearby context-field heuristic; the
[map-layout listener](209026-quest-navigation-guard.md) outside that range
already demonstrates why this is not an exhaustive runtime subscriber list.
The [content-direction callback](209026-content-direction-startup.md)
was classified separately and establishes the server-538 report dependency.

Private bundle: `~/re/gw2/notes/209026-startup-alert-chat-1.json`
(1,867 records). Static REA only; no source behavior or client state changed.
Pointer and RIP searches were candidate discovery; final table claims use
validated constructor stores and direct table reads. The previously
exported `209026-content-direction-startup-1.json` contains the registration
sites and function ranges used here.

| REA evidence | Observation |
| --- | --- |
| `ev_e189a24e89a2152b523f3e54e9a9dd6796a737a7696616e0dd94c2106822d56d` / `ev_d8b539a6af4888ca6275da8143659f04122537e19b2e0037750cdc23a752f712` | Chat initialization function pointer and primary-table start. |
| `ev_7633e3ab219b3da19b4d3283c4d534513962344070c92c9456088b6e1df2e4f9` / `ev_c5c427c8d281aed48f7e3c5df2ec4a0c773cb2e4e552db0e3eb18649de4435d8` | Constructor-reference candidates and validated constructor/destructor stores. |
| `ev_53d14d98a98b82c4d0e47666f192ee1a41afb64e81e431b9f3fa180f73cea12e` / `ev_99d514cf2df141fdde3d6c4f1f503e259802e2c81d00670621021055cf79f666` / `ev_82e4a803fa5186243204a6e0c83c5e2afd4a97f4c261d16965c2c877bc1815de` | Chat registered-interface installation, no-op entry and adjacent source strings. |
| `ev_fe6e842db2e69072c404e22362b22bca897184131e5ee2f35dcbd5d0f531372f` / `ev_b0636558159016672902ea3a411a98f9a1a96d028ced13aa42aee10e66f375ae` / `ev_b92938b8b6928aa324dc48f7d93dfd9972b63aed746b9af4fb7714141243429e` | Alert wrapper candidate, validated initialization branch and unwind range. |
| `ev_86e8480d62dd2dfe844aa3eba52ad0c852c9fd5d1cf3243c801807627908765c` / `ev_e2e6e5ac1791ac7900702fa22d39c35c5c396d487ccc45bbfc4744c5f3759f3d` | Alert primary-table function reference and start. |
| `ev_e3b13215bc02a08b3dac127d9b1158edd8eb0a58d4bb068f293849fa2129c50a` / `ev_34fe750dffad82e7b3f658390bc5c27cdde27f026de13f8da820647a674d9ee1` / `ev_01180183d43bf97898cb4124429b2251f626028ff7554c8a23252fb5f16919c0` | Alert constructor-reference candidate, interface stores and validated constructor boundary. |
| `ev_93104572b22e4acedc8e61576d540bf5e373fd3f45a6f807537f475cf477fa18` / `ev_cec0814ca13673d92be0def46188e8b0eafd620c8eba6a45c4ca8d187abd8096` | Alert no-op startup entry and source-path/flag assertion strings. |

## Character, buff and state-dependent listener slots

The final three retained candidates also use `0x140163990` for startup
+0x80. These classifications close the six outstanding sites named by the
original bounded search, without extending its coverage claim.

| Object | Constructor / registration site | Registered interface / table | Startup +0x80 entry |
| --- | --- | --- | --- |
| Character | `0x1411d2810` / `0x1411dfd98` | Whole +0x10 / `0x142161358` | `0x1421613d8` |
| Character buff component | `0x1412c2560` / `0x1412c3d0c` | Whole +0x10 / `0x142184288` | `0x142184308` |
| Unnamed state-dependent object | `0x1410f8dc0` / `0x1411031e3` | Whole +0x20 / `0x142140560` | `0x1421405e0` |

Character registration is in `0x1411dfd30`: it requires whole +0x248
nonnull, queries the current player through context +0x70, and registers
when that result is null and whole +0x178 mask 4 is clear. Constructor
stores at `0x1411d28b8..0x1411d28c6` establish the +0x10 interface table.
The buff constructor installs its +0x10 table at
`0x1412c259c..0x1412c25a3`; its primary table is `0x1421841a8`.

For the unnamed object, `0x141103180` compares requested state EDX to
whole +0x90. Its registration branch requires the previous state to be 1,
a different requested state, and whole +0x388 null; it stores the context
there and subscribes whole +0x20. The direct wrapper `0x141103f40` supplies
requested state 1, so that wrapper alone does not establish this branch's
startup reachability. Its primary table `0x14213fed0` includes the wrapper
at `0x142140098`; the validated constructor installs the actual listener
table at `0x1410f8e0a..0x1410f8e11`. The object remains unnamed because
identity is not needed to classify the constructed callback slot.

None of these **startup slots** reads components or emits messages. This
does not classify their initialization methods, other callbacks, or actual
runtime subscription state. The earlier search still excludes other code
regions and can miss aliases or register-indirect call patterns.

Private bundle: `~/re/gw2/notes/209026-startup-remaining-1.json`
(1,883 records). Static REA only; no client launch, network request or
source behavior changed. Raw reference scans selected candidates; final
claims use constructor stores, validated disassembly and direct table reads.

| REA evidence | Observation |
| --- | --- |
| `ev_7430d3c5defa0941417d26b2258c477a97ed3e749c64340c726ef4c3ffe74d36` | Character and state-dependent registration guards. |
| `ev_40885e1517a09db26e7f36d693fec0326e36055728c51de284b3720da4bb84ba` | Character constructor leading table stores. |
| `ev_6a7aba08934afe4372207a8de5ebc69369e4c24ee28869d0bf91296b9767dbd5` | Character and buff registered-interface stores. |
| `ev_1c8ebf8aed7a6766c147283b82d39c78a707055dea87c7c426a9811e4a8b5a7d` | Both startup table entries resolve to the no-op. |
| `ev_ed2aff6774ec30e7585979ab3871ab727f2ce5430988a2a4693102e3baed19dc` / `ev_b2bfb9ed9d8324d30236559539df1beda2daf78f5d083e78a4630bd44a20df52` | State wrapper and primary-table reference. |
| `ev_4f0f56ee06cd7363fbb948373269483b0a16cc327f452ffce2520329e74a376e` / `ev_bea7fb24045904e2a57d87b4f50beecb2c3db04b083e24ff169da325040f4910` | Validated constructor and +0x20 interface store; complete registration branch. |
| `ev_5ac115b655cc7090b4646394c902ce83fcadb19a071241d9d83135c22c59ef60` | Unnamed object's startup table entry is the no-op. |
