# Build 209026: quest store process-event dispatch

Static REA follow-up, 2026-11-01, to the
[store response callbacks](209026-quest-store-callbacks.md). Store result
IDs 0x42/0x43/0x50 are nested inside process event **0x23**. The receiver
decodes item/package buffers and notifies store listeners. Its ordinary
error notification reaches a no-op slot in the quest-facing service.
This does not establish request-tracker completion or zone readiness.
The client remained stopped; no request or agreement acceptance occurred.

## Queue envelope and buffer lifetime

`0x14106dc50` constructs a 16-byte outer header with event kind 0x23
(optionally OR 0x8000 from queue flags), followed by a 16-byte inner
header and the copied input buffers. Relative to the inner header:

| Offset | Value |
| --- | --- |
| +0, Dword | Caller-supplied result ID: 0x42, 0x43 or 0x50 here |
| +4, Dword | Caller EDX, zero for these emissions |
| +8, Dword | Sum of copied buffer lengths |
| +0x10 | Copied payload bytes |

The queue checks its own flag 1 before allocation and returns false when
set. Otherwise it reserves/grows queue storage, copies headers, and loops
over the supplied buffer/length arrays. At `0x14106def3` it invokes
`0x1409b7320` with destination queue storage, source buffer and byte count;
it advances the write cursor before returning success. That thunk jumps
to the byte-copy routine `0x1418e9ae0`. For the twelve-byte error payload,
the size table selects `0x1418e9bc3`, which copies eight plus four bytes.
This establishes a copy of the supplied byte range, not recursive
ownership of any pointers that a payload might contain. The outer
`0x141069d20` process gates still apply, including its true-without-queue
branch documented previously.

## Receiver registration and inner dispatch

Process registration API `0x141069df0` asserts its event number is below
0x31 and not 0x2b, obtains thread-context +0x30, and registers through
`0x14106d3e0` on process +0x40. A raw search for direct 0x50 registration
therefore targeted the wrong layer; its empty result is not receiver absence.

`0x14025a420` guards service +0x6c0 mask 1, sets that bit, and registers
**`0x14025f600`** for outer event **0x23**. The receiver passes the inner
header to `0x14025f700`, using context +0x1f8 service +0x1d0 as its
object. That dispatcher subtracts 0x2b from the inner ID and selects a
38-entry RVA table at `0x140260b24`:

| Inner ID | Table entry / branch | Result |
| --- | --- | --- |
| 0x42 | `0x140260b80` / `0x14026035e` | Decode using `0x1425555b0`; notify listener +0x68. |
| 0x43 | `0x140260b84` / `0x1402603a3` | Decode using `0x142555720`; notify listener +0x70. |
| 0x50 | `0x140260bb8` / `0x1402609f1` | Request-ID/error bookkeeping and conditional error notification. |

Both decoding branches pass inner +8 length and +0x10 bytes to
`0x140ff2600`, then use `0x1402528c0` over service +0x88 listeners.
Trampolines `0x140255598` and `0x1402555a0` select +0x68/+0x70 respectively.
The list helper invokes the trampoline with the decoded result pointer.
Full decoder schemas are outside this trace.

The error branch reads request ID at inner +0x10 and error bytes at +0x14.
It first conditionally removes a matching entry from service +0x6e0.
It then searches the Dword array at +0x608/count +0x614. A match removes
that ID by replacing it with the last entry and decrementing the count,
then calls `0x14027be60` on service +0x620. If absent, it calls
`0x1402623d0`, which handles other request-specific fields and normally
notifies service +0x88 listeners through trampoline `0x140255584`, virtual
**+0x160**, with the request ID and error pointer. There is also a
request-specific retry path that skips that notification. These fields
are not identified with the separate startup request tracker.

## Quest-facing service interface

Context +0xc8 service constructor `0x1412cf970` installs its whole +0x10
interface table **`0x1421858b8`**. Its primary virtual +0x18 resolves to
`0x1412d1620`, which subscribes this interface through context +0x1f8
virtual +0 (`0x14025d870`, insertion into the +0x88 listener list).
This establishes how the interface can receive the notifications, without
asserting runtime subscription state.

| Listener slot | Entry address | Callback |
| --- | --- | --- |
| +0x68, item | `0x142185920` | `0x1412d0a40` |
| +0x70, package | `0x142185928` | `0x1412d0660` |
| +0x160, error | `0x142185a18` | No-op `0x140163990` |

The item/package callbacks have executable bodies; their full mutation
and notification paths remain to be traced. The error slot itself cannot
clear this service's quest subscription or complete the separate tracker.
Other registered listeners may react to the same error, so this is not a
claim that the error event has no global effect. No empty-result/error
fixture is runtime-validated by this analysis.

## Evidence

Private bundle: `~/re/gw2/notes/209026-quest-store-dispatch-1.json`
(1,905 records). Frozen executable SHA-256:
`99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`.
All artifact reads used REA. Raw scans selected candidates; table values
and validated disassembly support the claims. Unaligned exploratory range
edges were excluded. No source implementation or runtime state changed.

| REA evidence | Observation |
| --- | --- |
| `ev_a727436db93446afba597be61b66f9070c2933530729d966bcee2564c3455da4` | Registration API constraints and process gates. |
| `ev_3178dd9d5a9a4b8e58f5f51dab1c4bfdff806a427801a88d50f52eb20643d18c` / `ev_0894b49c8925e42ac83b1b21124a17decb491e1a84d2fa101dd8526e703821f4` | Queue envelope, storage and byte-copy loop. |
| `ev_bac88e8f62d72b0e6d0a8e2338d08b3793768c2479ebc0b45b670f852854067d` / `ev_37ffb7b0153de331af7bd4a4f41d6922a96b9f98c1b80765449976783b92a7ec` | Event-0x23 candidates, validated service registration and receiver. |
| `ev_092267e12d47b829d5cb95cba6b868cb28adae2d491951e2ce9ba784ec3cb88e` / `ev_df0c662ce221fa2f0dbdc5f2bb4df013a7ee498b3efaa6893fb471659e52b30b` | Inner dispatcher and complete selection table. |
| `ev_633a3c3c7f61ed41a911dd093566f1b15a1a01374a82e71c4c0efbbc61d65400` | Three selected result branches. |
| `ev_e99b7b8a027b740a92e8e2eec26d0bf9bbb39d4119db1a9230967b1970652a27` / `ev_9aad02240f11c6acf02f13559344a76909fe829b9ce0cc7ea166415efa6d2cac` / `ev_828003c7087ba5652472f3e8268c4374a527e1adade39d1e3e0b6ceef5a506f2` | Notification helpers, error routing, constructor interface and copy routine. |
| `ev_a0750a01b06add91c0ce550812c1bed7658423ac4ab45c9d9e6aa9fc461ca9a7` | Twelve-byte copy case table entry. |
| `ev_a402dd7d05f961aa3c1fcad6d2779325802207a34ac7987df5b327e4c89cd3e6` / `ev_03f67df638771859f021a8aa03d91c5d5f8e0df87ff10cf9fed09e408ef821bd` | Store service callback and registration method entries. |
| `ev_5808a0d4423cdf98b3e24230692861976aca2c9790431ff63916962fa18eaed3` / `ev_72f0cd5c3aca2e4680274f642b446f20a48fbdcece93007cbd077da34c349a4f` | Callback leading bodies, registered interface and listener insertion. |
