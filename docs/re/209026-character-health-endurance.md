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
not a health-state transition mechanism. The later [transition trace](209026-health-transitions.md) maps dead/downed/alive packed commands through server 84.

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

### Creation callback ordering and component prerequisites

The allocation branch of `0x1411d6310` stores the new health pointer,
then dispatches character +0x1e0 listeners through trampoline `0x14026df60`
(listener virtual +0x130). **Only after that dispatch** does it store the
supplied state at character +0x188. Listeners invoked by this event therefore
cannot assume that this message's new state is already installed.

After the state store, character +8 virtual +0x68 (`0x1411d9be0`) checks
identity high nibble 3. On that player branch, virtual +0x30 (`0x1411d8e50`)
checks whether the newly stored state is zero/alive. A non-alive player
causes character flags +0x178 mask 2 to be set. This tail only sets that
bit; it does not clear it for an alive state. It is not a general death or
revival state-transition implementation.

Health constructor `0x1412071e0` initializes its two meters using local
world time, initializes its listener collection, stores its owner, and
sets the supplied meter fields. The direct allocation/constructor path
does not retrieve coreStats, inventory, profession, skill bar or player
components. The character event's runtime-dependent listeners remain an
untraced boundary, so this is not a proof that health can safely be sent
at every point after character creation.

Endurance allocation `0x1411d6240` constructs and stores its component
without a character-component event. The constructor's `0x141411670`
call registers the secondary interface (component +8) with a global
settings listener list at TLS +0x1c8, object +0x460. The helper inserts a
listener node and returns; it does not invoke the listener during insertion.
Its secondary vtable `0x14216ae48` has no-op entries at 0/+0x10 and callback
`0x141223180` at +8. That callback rereads the named
`CharEnableEnduranceDeficitTracking` setting and changes component +0x30
bit 0. Both construction and callback use `0x141414340` with fallback 1.
This settings registration adds no direct dependency on another character
component. The global settings service and allocator still must exist.

Private bundle: `~/re/gw2/notes/209026-health-endurance-initialization-order-1.json`
(1,133 records, 24,287,322 bytes). Static REA only; no client launch.

| REA evidence | Observation |
| --- | --- |
| `ev_bf45f05696e4b6c0c0846ccd242619d898b7b3d8d109ec9016ccbf5be3d9ad0c` | Full creation helpers and endurance constructor. |
| `ev_ffe07dc7133a7d6c5ee70d5abf1a8ca8a3ca0a73f1e92896b000e7d89bf95106` | Health constructor, event trampoline and global listener insertion. |
| `ev_aee1f98c302b3aee0ee219dfb6c27a804ed09bac25725181ec7e6359ab78e1e0` | Insertion tail, player predicate and endurance secondary vtable. |
| `ev_62028d8964304b3709d6150008afa6df7171e803dee85c5bc440a779b533bf44` | Settings callback, fallback lookup and alive getter binding. |

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
They are Dword varints on the wire, not four raw little-endian floats or a
fixed 16-byte block. The follow-up below establishes their storage and uses.

These traces establish message roles and component lifetime behavior. They
do not establish all required initialization messages, valid gameplay
values, or a completed controlled-character/map startup.

### Endurance values and updates

Vtable `0x14216adc8`, direct setters and named assertions establish:

| Creation field after index | Offset | Established role |
| --- | --- | --- |
| 1 | +0x10 | Current points. Setter `0x141223250` asserts `points <= m_effectiveMaximum`. |
| 2 | +0x14 | Maximum used by fraction getter `0x141222d40` and percentage adjustment `0x141222ca0`. Its relationship to the effective maximum is not yet fully named. |
| 3 | +0x18 | Effective maximum, named by the points assertion at `0x14216af38`. Fraction getter `0x141222d60` uses this denominator. |
| 4 | +0x1c | Segment divisor: `0x141222d80` returns unsigned effectiveMaximum / this value, or 1 when it is zero. A dodge-cost name remains unproven. |

The corresponding raw getters occupy virtual offsets +0x20/+0x10/+0x18/
+0x28 respectively. Virtual +0x38 (`0x141222da0`) returns the segment
divisor / effective maximum as floats, with special cases 1 for a zero
divisor and 0 for a zero effective maximum with a nonzero divisor.
Both fraction getters return zero for a zero denominator.

The percentage condition consumer `0x141306fa0` is identified by
`CONTENT_FUNCTION_CONDITIONAL_TYPE_HAS_ENDURANCE_PERCENT`. It compares
current / effectiveMaximum against a content float multiplied by f32
0.01 (`0x141934804`, bits `0x3c23d70a`); zero effective maximum fails.
The absolute condition at `0x141306e30` compares current points converted
to float with its content threshold. Neither establishes useful initial
point counts or the fourth field's gameplay name.

The creation constructor copies values directly; it does not perform the
points setter's maximum assertion. The maximum-pair setter `0x141223200`
stores +0x14/+0x18 and clamps current points down to the new effective
maximum using an unsigned comparison. The delta adjuster `0x141222ce0`
instead adds a 32-bit delta and clamps with signed comparisons; it must
not be conflated with the direct wire points update.

All following messages begin with a Dword character index and require an
existing endurance component. Registration pairs follow `0x14216f300`.

| ID | Descriptor | Handler | Fields after index / effect |
| --- | --- | --- | --- |
| 218 | `0x1425e0240` | `0x14124ed00` | None. Calls `0x1411d6840`, destroys/frees the 0x68-byte component and clears character +0x3d0. |
| 219 | `0x1425e0310` | `0x14124ee80` | Dword points, passed to `0x141223250`. |
| 220 | `0x1425e0450` | `0x14124edf0` | Dword maximum, Dword effective maximum, passed to `0x141223200`. |
| 221 | `0x1425e05c0` | `0x14124ed60` | Dword segment divisor, passed to `0x1412231e0`. |

The three update setters reset the deficit-tracking fields +0x20 to
current points and +0x24 to zero. The [deficit follow-up](209026-endurance-deficit.md) traces separate balance/deficit
bookkeeping and its half-maximum predicate; action callers remain open. No update above creates a missing component.

Private REA bundle: `~/re/gw2/notes/209026-endurance-fields-1.json`
(987 records, 22,252,022 bytes, mode 0600). Static analysis only.

| REA evidence | Observation |
| --- | --- |
| `ev_a6cdfca8f5dc0bc1b2bf6b66c62bc083e9e4f60ee454be9dffadf67a4eee8071` | Constructor, wire argument order and fraction adjuster. |
| `ev_6bfd24f5d5df6a7f7090cf0e0f503c4b8d8be23616a623742a3f775a22e67ee1` | Vtable, ratios, segment division and delta adjuster. |
| `ev_426a98c525aa74b56a350e5b5d0365b64f8928c181a71b4602202a7bcd1b745b` | Direct setters and named effective-maximum assertion. |
| `ev_c06a81dffeeaadce85cb28e85cb718aa6eaffad8548e961b97da436ba6be878e` | Setter call sites and named content conditions. |
| `ev_0ab3479ce8b6dae11e164dd1116a3f711ffeb459a6f7f96909ab22d3ceabaf8f` | Absolute-points condition consumers. |
| `ev_e58a6001b883c04a9358e4cc25f063fbeb41a2722689ee26584713c25265de31` | Registration pairs and full percentage condition. |
| `ev_c3fb04bc894a5168c4c9b8bbf4cefcb443978e4d58f3c65815224631acfc8850` | Hash-guarded descriptors, update handlers, removal and percentage constant. |

## Offline framing check

### Typed endurance-frame scope

`gw2_proto::endurance::EnduranceFrame209026` covers IDs 217–221 with an
explicit character index and operation enum. All Dwords remain unsigned
varints. The fourth creation value is named `segment_divisor`, describing
the observed division without assigning a dodge-cost meaning. The codec
preserves raw values, chooses no defaults and does not enforce lifetime
or gameplay constraints. Decode commits input advancement only on success.

Three focused tests and `cargo build -p gw2-proto --lib` pass. Independent
hand-authored fixtures exercise every operation, unsigned varint boundaries
through u32::MAX, output-prefix preservation and concatenated boundaries.
Every truncation, both invalid neighboring IDs, and overflow/overlong
varints in all five creation Dword positions leave the input unchanged.
No endurance frame was sent to the client.

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

## Health setter notifications: values differ from stored meter state

A further static trace identifies the six setter notifications without naming
F5. All use the listener list at health +0x48 through `0x1411cffa0`.
That dispatcher supplies the listener as RCX, the owner stored at health
+0x78 as RDX, and the float addressed by its fourth argument as XMM2.
The small trampolines select these listener virtual methods:

| Setter input | Listener virtual offset | Float passed to listener |
| --- | --- | --- |
| F0, first current | +0x08 | Original setter input. |
| F1, first maximum | +0x10 | Original setter input. |
| F2, first rate | +0x18 | Original setter input. |
| F3, second current | +0x20 | Original setter input. |
| F4, second maximum | +0x28 | Original setter input. |
| F5, unnamed field | +0x10 | First meter maximum, read from health +0x10. |

For F0–F4, the setter saves the incoming float in its caller-provided stack
space before calling the meter operation, then passes that saved float to
the dispatcher. It does not re-read the meter's post-operation value. Thus,
for example, a finite current input above the maximum can be clamped in
storage while the callback still receives the original larger input.
This is a distinction for future replay/state models, not permission to
choose out-of-range initialization values.

F5 setter `0x141207730` reads the first maximum, stores its own incoming
float at health +0x40, then dispatches the saved maximum through
`0x140255570` (listener +0x10). It does not pass F5 as that callback's float.
There is no equality test in these setters before dispatch, so unchanged
inputs still reach the listener-dispatch operation. Individual listeners
can impose their own behavior; their bodies are not exhausted here.

The shared callback with maximum health is not enough to give F5 a gameplay
name or derive a valid initial value. It remains a separate raw float in
the wire codec. A bounded getter-consumer search found no direct E8/E9 calls
to `0x141207490` or `0x1411d7cb0`; byte-pattern searches for nearby virtual
+0x68/+0x40 calls produced unrelated-interface candidates in the inspected
windows. These searches are incomplete and do not establish that the
getter is unused. No client was started or packet replayed.

Private bundle: `~/re/gw2/notes/209026-health-listener-values-1.json`
(1,267 records, 26,590,115 bytes).

| REA evidence | Observation |
| --- | --- |
| `ev_a9913ba0fca47861506f7770e2aa5dfbd29ceeed9ca3ff2e652a36b75a3a2ed7` | F5 getter and complete setter; listener registration/removal implementation. |
| `ev_950d235ec6c71807e17ee9e56b7e3e7557bcb463a708db991e12f949b9872b42` | Complete F0–F4 setters and dispatcher argument retention. |
| `ev_73b4c59a63f0bb9ba368b192cd76eb88826d9f3c096c4a4b466d370588aca3eb` | Float/owner dispatch and all five virtual-method trampolines. |
| `ev_9f98aa7d35e4d810c259ab6424e289f6ad5353a3b6791261499eb51eeb9e5262` | Aligned callback boundary confirms listener RCX and argument dereferences. |
| `ev_73ec2a03fd4a4af8b89d207e7f659b504e529641a099a1018c3adb87e3c3d4e8` | No direct getter calls; two F5 setter call candidates match previously traced creation/update paths. |
| `ev_d57dcf9e757e85fcc66686f3984f39a2f89b057947bf0cdc8d6304b3ff70edf8` | No direct calls to the character health-component getter. |
