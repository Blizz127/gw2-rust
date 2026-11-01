# Build 209026: health and endurance initialization

Static REA findings, 2026-11-01, for the frozen executable identified in
[mission transfer](209026-mission-transfer.md). Both messages are registered
in protocol 0, group 0's character table. No runtime replay occurred.

## Server 256: health component creation or update

Descriptor `0x1425e22b0`, pair `0x14216f490`, handler `0x14124fa90`:
**Dword character index, Byte, six Floats**. The handler requires a nonnull
raw character, then calls `0x1411d6310` with the six floats and the byte.

The helper uses whole-character +0x3e8. Existing-component getters and the
removal path explicitly assert **m_health**. If null, it allocates 0x88
bytes and calls constructor `0x1412071e0`, whose vtable/source diagnostics
identify `ChCliHealth.cpp`. The constructor stores the owner at +0x78 and
initializes two `CharMeter` objects at component +8 and +0x24.

If the component already exists, the helper invokes six setters rather
than allocating again. This differs from coreStats and endurance creation,
whose helpers assert that no component exists.

Numbering floats F0 through F5 in wire order:

| Float | Existing-component setter | Storage/operation established |
| --- | --- | --- |
| F0 | `0x1412076b0` | First meter's current value at component +0xc, through `0x141275fd0`. |
| F1 | `0x1412076f0` | First meter's maximum at +0x10, through `0x141276030`. |
| F2 | `0x141207770` | First meter rate at component +0x14, through `0x141276120`; units per second (follow-up below). |
| F3 | `0x1412077b0` | Second meter's current value at component +0x28. |
| F4 | `0x1412077f0` | Second meter's maximum at +0x2c. |
| F5 | `0x141207730` | Component +0x40, also exposed by getter `0x141207490`. Meaning unresolved. |

The current-value setter clamps ordinary finite inputs to [0, maximum].
The maximum setter asserts `valueMax >= 0.0f` and preserves the previous
current/maximum fraction while changing the maximum, with a zero-maximum
special case. The update helper sets each maximum before its corresponding
current value. The constructor follows the same ordering. No assertion here
establishes that arbitrary NaN/infinity inputs are usable.

The Byte is stored at character +0x188 **only on the allocation branch**.
That branch also emits a character-component event and can set character
flags +0x178 mask 2 after checking player/state predicates. The byte's
meaning is established below; the predicates' complete role remains open. Sending the
same layout to an existing component therefore does not repeat every
creation-side effect. The follow-up below names the byte as health state.

### Health-state byte and first-meter rate

The Byte is zero extended at handler `0x14124fad2` and passed to the
creation helper, whose allocation branch stores it as a Dword at character
+0x188 (`0x1411d6477`). Character primary virtual +0x238 resolves to
`0x1411d7d50`, which returns that Dword.

The party WebUI consumer `0x1403670b0` calls that getter at `0x1403670f5`,
asserts `state < CHAR_HEALTH_STATES` with upper bound 3, and uses the result
to index a three-pointer label table at `0x14255c078`:

| Byte | Embedded UTF-16 label | Literal address |
| --- | --- | --- |
| 0 | `alive` | `0x141ad70b8` |
| 1 | `dead` | `0x141ad70c8` |
| 2 | `downed` | `0x141acf3f0` |

Independent predicates agree: primary virtual +0x360 (`0x1411d9260`)
compares +0x188 with 1, while +0x370 (`0x1411d9280`) compares it with 2.
The character +8 interface's virtual +0x30 (`0x1411d8e50`) compares its
+0x180, the same whole-character field, with zero. A named consumer at
`0x14141b813` uses the latter and +0x370 in the assertion
`character->IsAlive() || (character->IsDowned() && character->IsInWater())`.
Thus the labels are directly tied to the health creation byte. The update
branch still ignores this byte; sending 256 to an existing component is
not a health-state transition mechanism.

The same party consumer obtains health through primary virtual +0x68,
`0x1411d7cb0`, which returns character +0x3e8. It publishes health virtual
+0x10/+0x18/+0x20 under literal keys `current`, `max`, `rate`. Health vtable
`0x142168cf8` maps these to getters `0x140ccb790`, `0x14089ba20` and
`0x1412074a0`, reading component +0xc/+0x10/+0x14. This names F2 as rate.

The rate setter stores meter +0xc and clears meter flags bit 1, selecting
the supplied rate directly instead of recomputing it from the separate
base/multiplier pair. Health update helper `0x141207460` passes the local
world time from `0x141023500` (world +0x1d4) into meter update
`0x141275f20`. The [clock trace](209026-agent-command-clock.md) establishes
that this field counts milliseconds. For ordinary finite inputs, when
not paused and after the initial zero-clock special case, the update is:

```text
delta_ms = u32(now_ms - previous_ms)
current = clamp(current + f32(delta_ms) * f32_bits(0x3a83126f) * rate,
                0, maximum)
```

The constant at `0x14192d3c0` is f32 0.001: F2 is current-value units per
second. The subtraction is a wrapping 32-bit subtraction, then converted
as a zero-extended 64-bit integer. Meter flags bit 0 pauses accumulation
while still advancing the stored clock. These are client-side meter rules,
not server-authoritative combat behavior or a complete runtime validation.
F3/F4's second meter and F5 remain unnamed.

Private follow-up bundle: `~/re/gw2/notes/209026-health-state-rate-1.json`
(972 records, 22,016,895 bytes). Static REA only; no client launched.

| REA evidence | Observation |
| --- | --- |
| `ev_b38fc084aa017441e35d2fcbe0eed9394a0960318fd9bfa6298b82486c586f49` | Creation/update split, state store and health constructor. |
| `ev_f59f424976978fe46cc1ced33d1699152c119b14cfcc5ddda83ee33026fe81c0` | State getter, equality predicates and state mutations. |
| `ev_5d1dd503783bd9d4b8551fda81c8acfbfa7e7636c58e8732dff24a704bb1fc86` | State assertion strings and predicate vtable entries. |
| `ev_7de29f67db02a624987395b352b13f9a4d61145ff79b252aea5df4e2b29e465c` | Label-table consumer and independent alive/downed assertion. |
| `ev_cbc1da600f0e5c3d05081bb8afde4457d97f8c162870375f17dc58fa54a4ac59` | Party consumer calls, label pointers and health property keys. |
| `ev_6bcaedd5c27855b9fb4eaf0a19324bc1539723bb6e7e55a8964208e44bfe30ff` | Character/health vtable bindings and health tick caller. |
| `ev_069edcfb873b43f91db0f95bb7a60f35460a9b448020145607fe3dc5ccd73186` | Exact getters, byte zero extension and direct rate setter. |
| `ev_fc3266bc7f314d2716af71db173b8151482758dc209ff85fb1f8fdb52769c872` | Meter initialization, accumulation, pause and local world-clock getter. |
| `ev_77f3e77f7848b6bc933cf37fec5f22e2f9794e878b5db867bd800cde67453063` | Rate conversion constant and all three state strings. |

## Existing-health messages 257–260

The consecutive registrations following 256 identify four further messages:

| ID | Descriptor | Handler | Fields after character-index Dword |
| --- | --- | --- | --- |
| 257 | `0x1425e2440` | `0x14124fb30` | Float current; Float second-meter current. |
| 258 | `0x1425e2510` | `0x14124fbb0` | None: removes the health component. |
| 259 | `0x1425e2590` | `0x14124fc10` | Float current; Float maximum; Float second current; Float second maximum; Float unnamed F5 value. |
| 260 | `0x1425e26d0` | `0x14124fcc0` | Float first-meter rate. |

All four require an existing raw character. 257, 259 and 260 obtain its
existing health component through primary virtual +0x68, whose getter
asserts `m_health`. They do not allocate health. The 259 handler sets each
maximum before the corresponding current value, despite the opposite order
on the wire, and does not update the first-meter rate. The 260 handler
calls the same direct-rate setter as 256. None of these handlers sets the
character's health-state Dword at +0x188.

258 calls `0x1411d68b0`, which asserts that health exists, destroys/frees
the 0x88-byte component and clears character +0x3e8. It does not remove the
character or set the health-state Dword. It is not a death-state message.

The second meter and F5 retain unnamed gameplay semantics. Its getters
are established mechanically: health virtual +0x28/+0x30/+0x38 read
component +0x28/+0x2c/+0x30 (second current/maximum/rate), while +0x40
reads +0x40 (F5). The bounded UI-name search did not supply a direct name
for them; this note does not assume barrier, shield or defiance semantics.

Private bundle: `~/re/gw2/notes/209026-health-updates-1.json`
(980 records, 22,116,985 bytes). Static REA, no messages sent.

| REA evidence | Observation |
| --- | --- |
| `ev_85d46f683b7559b9097580c9e802fc1c8fe29b30436395781b61503ef960433d` | Complete health vtable and second-current getter. |
| `ev_9b518cf93155e32754a5ebc05907027b50be319bda1332748ca80876f9dbb535` | Second-maximum/rate getters and bounded UI-name search. |
| `ev_4fe9bf5d10d5a2bf8761ba01324940fb88f6c6e5257f8631876a7cd0244d492b` | Direct setter call sites. |
| `ev_16060f5cde33acaa16f5c758b337cf6cc2c662e2e84a51dc1996f66e93002607` | Registration pointers and complete 257/258/259 handlers. |
| `ev_3b2b93dca748d762bdaa2c0e6342f91219f8fd9e98a72eb266e719689274e76f` | Hash-guarded descriptors, 259 registration and removal body. |
| `ev_01b1868c372bc1c68cf6b5e4dac0d30c33196130091cdb3b182f22b3886cc4b5` | 260 descriptor/handler and common float setters. |

## Server 217: endurance component creation

Descriptor `0x1425e00d0`, pair `0x14216f300`, handler `0x14124ec90`:
**five Dwords**. The first is the raw character index. The handler passes
the remaining four values to `0x1411d6240`, which checks **!m_endurance**
at whole-character +0x3d0 and allocates 0x68 bytes through `0x141222b40`.

That constructor stores the four values at component +0x10/+0x14/+0x18/+0x1c
in wire order, copies the first to +0x20, and retains the owner at +0x28.
It also consults the named `CharEnableEnduranceDeficitTracking` setting.
Units and individual scalar meanings remain unresolved; they are Dword
varints on the wire, not four raw little-endian floats or a fixed 16-byte block.

These traces establish message roles and component lifetime behavior. They
do not establish all required initialization messages, valid gameplay
values, or a completed controlled-character/map startup.

## Offline framing check

### Typed health-frame scope

The typed health codec covers IDs 256–260 with an explicit character index
and operation variant. Creation includes a raw state byte, rate and the five
other floats; existing-value updates omit state/rate, current-only updates
have two floats, rate-only updates have one and removal has no further
fields. It preserves float bit patterns and unknown state bytes for bounded
framing inspection. It does not mutate game state, choose health defaults
or validate a component's existence. Decoding advances the input only after
one complete frame succeeds; all variant payloads have fixed size.

Implemented as `gw2_proto::health::HealthFrame209026` with an operation
enum. Three focused tests and `cargo build -p gw2-proto --lib` pass.
Hand-authored fixtures cover all five message IDs, raw state preservation,
negative zero/NaN/infinity bit patterns, distinct creation/update field
orders and an output prefix. A concatenated stream checks every boundary;
all truncations plus invalid IDs and an overflowing index leave the input
unchanged. These are local framing checks; no health message was sent.

### Descriptor/census validation

The hash-guarded extractor wrote private mode-0600
`data/protocol/209026/character-health-endurance.xml` (733 bytes, SHA-256
`d37b82b7e323ece6dbdc9bdd1d8e76b9a51659504f57aa9fe22ed6c73e99e915`).
The existing census consumed two hand-authored frames completely: server
256, 29 bytes; server 217, 15 bytes; total 44. A truncated final health
float was rejected. The endurance values exercise varint boundaries and
are not asserted to be meaningful gameplay values. No traffic was sent.

## Evidence

Private bundle: `~/re/gw2/notes/209026-character-health-endurance-static-1.json`.

| REA evidence | Observation |
| --- | --- |
| `ev_73100f8ad1f48465e0985faf7b88e68f8054e1d27c3c5645b54a6574098c01cb` | Named health/endurance assertions and component offsets. |
| `ev_55c063c877a889b5c21b5b66479b290a87e336eb46f89b54bd6ece6470230431` | Allocation/update split and endurance allocation. |
| `ev_6c642f060b982734db34d161b3d218aca3946bfcff0e436038af5045f0c0c724` | Allocator call sites in message handlers. |
| `ev_a5ca3498e88fd5243f22244c7a2c57043198bb448274616b699ae3cee084329a` | Full handler arguments and endurance registration pair. |
| `ev_c29007260277eb4417522d41052326a42c4d0e371633709d3f9c89c374c5a89f` | Health registration, endurance descriptor and component constructors. |
| `ev_3aedb5b10a834e84b42a900032b957825320972c009eb314540f5e9f98516d77` | Health descriptor, float setters and meter operations. |
| `ev_38375ed008e6945ca241fcefe66736e0c1c56484c2aa35478741bc9481c7f97d` | Health vtable/getters and named meter maximum assertions. |
| `ev_d8c53bac43bb3003a4d2be082894a8c2c2aa1eb46fd4436e68ee2bbaed299c1a` | Hash-guarded two-message schema extraction. |
