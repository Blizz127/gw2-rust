# Build209026: packed0x29/0x2a producers and local consumers

The post-send update in packed0x29's producer is a **fishing-feature-gated
consumer of the current slot skill**, not a demonstrated ordinary sword
repeat latch. Packed0x2a is independently proven to contain a raw slot
byte, but its complete producer does not justify naming it “release.”
Neither body establishes that the client automatically requests another
cast after completion.

This is a bounded static follow-up to
[repeat admission](209026-repeat-attack-admission.md) and
[sword chain selection](209026-mesmer-sword-chain.md), on frozen executable
SHA256 `99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`.
No runtime, archive, server behavior or helper was changed. Existing
fixture names such as `RepeatStart` remain implementation labels, not
independent source proof of retail repetition semantics.

## Exact two producers

The constructor-established order-component table `0x142179b90` binds:

| Component slot | Complete producer | Packed payload | Outer flags |
| --- | --- | --- | --- |
| +`0x210` | `0x1412848f0..0x141284a3c` | `[0x29, low_byte(EDX)]` | 3 |
| +`0x218` | `0x141284a40..0x141284ad3` | `[0x2a, low_byte(EDX)]` | 3 |

Both assert component +`0x40` as `m_isControlled`, initialize a **new**
behavior container through `0x141023f10`, allocate exactly two payload
bytes, send through bound-agent virtual +`0xa8`, then destroy the local
container. The existing carrier supplies its fresh behavior value and
other envelope fields. Neither payload includes a SkillDef key, target,
chain count, duration or active order's old correlation.

Before constructing its request,0x29 calls bound-agent virtual +`0x40`
and that result's virtual +`0x678`; a zero return skips the send and local
update. This slice leaves that gate's name unresolved. The0x2a producer
has no corresponding gate or post-send player/skill-bar update in its
complete body. The producer layouts are exact, while the numeric meanings
of every possible low-byte argument are not recovered here.

## What0x29 does after sending

At `0x1412849a1`, the producer checks the **full original EDX value**
against `0xffffffff`. That value skips the following local path, although
the already-built wire byte is `0xff`. Otherwise:

1. Character +8 interface virtual +`0xe0` (`0x1411d84a0`) resolves its
   player and the producer asserts `player`.
2. Player table `0x14215e9d8` +`0x290` is `0x1411bb0f0`, returning the
   embedded object at player +`0x9310`.
3. Character primary +`0xc8` gets its skill bar. Primary skill-bar
   +`0xe8` is `0x1411f8530`: it asserts slot `<23` and reads the cached
   `m_slotSkillDefs` array at +`0x1d0 + 8*slot`.
4. The resulting **current slot skill** is passed to virtual +`0xa0` of
   the embedded player object.

The skill-bar operation is a read of an already resolved slot. It does not
write the chain predecessor/count at +`0x1c8`/+`0x1c0`, select a palette or
advance from10170 to10171. The existing server628 setter and subsequent
palette rebuild determine that cached selection separately.

Player constructor `0x1411b9cc0` explicitly calls `0x1412326e0` for
player +`0x9310`; this constructor installs primary table `0x14216cea8`.
Its +`0xa0` target is complete function `0x141233f70..0x141233fc8`.
That body requires all of:

- `0x1410f3dd0()` succeeds. Its exact source string is **EnableFishing**.
- The object's owner equals the context's controlled player.
- The supplied skill and current mode pointer at object +`0x28` are
  nonnull.

Only then does it forward the skill to current-mode virtual +`0x30`.
The constructor installs eight embedded mode tables; seven bind that slot
to `0x140163990`, a complete `RET 0` stub. The remaining table
`0x14216cc08` binds it to `0x141233fd0`.

That nontrivial body first compares the supplied content object's initial
16 bytes with two fixed global values. On a match, it queries three
effect keys `0x3d`, `0x3e`, `0x3c`, and conditionally sets object +`0x130`
mask8 and switches the mode pointer to embedded object +`0x50`, invoking
exit/enter methods and a local notification. The global values and effect
keys are not assigned new names here. This completes the narrow local
update's structural explanation; fishing-mode listener consequences were
not expanded into a whole-client negative claim.

## Demonstrated skill-bar callers and their limits

A bounded check of the established skill-bar implementation found typed
calls through owning character +`0x4b0` to component +`0x210` at
`0x1411f518d`, `0x1411f60d4` and `0x1411f6318`. These are distinguished
from unrelated calls using the same virtual offsets on other interfaces.

Complete skill-bar function `0x1411f5f90..0x1411f6359` is primary virtual
+`0x1c0`. It compares the incoming slot with +`0x74` and +`0x180`.
One branch returns when +`0x74` is sentinel23 and +`0x180` is−1; other
branches clear a matching +`0x180` to−1 and clear retained target and
position fields. Controlled-character branches emit0x29 with the supplied
slot. The later branch then sets +`0x74` to23 and notifies listeners.
The earlier tail-call branch does not perform that last store. Skill-body
and mode-dependent branches can bypass these actions, so this is not an
unconditional per-frame or completion-driven send.

There is also a concrete0x2a caller: primary skill-bar virtual +`0x220`
is complete leaf `0x1411fb360`. It stores EDX at skillbar +`0x180`, gets
the owning character's order component and tail-calls component +`0x218`
with that same value. Adjacent skill-bar +`0x228`/`0x230` setters cache
target/position inputs and send a **different** component method +`0x220`.
These slot/target bookkeeping facts do not establish a physical key event
or a named press/release enum. In particular, the0x2a producer must not be
renamed release solely because it follows0x29 in address/opcode order.

## Remaining execution boundary

No ordinary repeating-request loop or completion-to0x29/0x2a invocation
is demonstrated by this slice. It therefore cannot decide that another
client request will arrive after each cast, nor justify a server timer
that invents casts without another request. The separate
completion/listener investigation must retain its own actual caller
bindings; a no-op listener alone is not whole-client absence.

The usable conclusions for the server are narrower:

- Preserve the exact two-byte requests and their fresh envelope separately
  from SkillBegin0x23 and an existing typed0x78 order.
- Treat628's predecessor/count state and cast acceptance/completion as
  independent inputs. This post-send path supplies no chain progression.
- Keep existing generic damage/repeat fixture policy labeled research.
  A typed input or completion caller leading back into these methods, or a
  separately authorized finite observation of that caller, is still needed
  before assigning ordinary repeat scheduling.

No further codec or server implementation was added in this lane.

## Evidence

The complete producers were reused from
`ev_45911bb23b1facbe58ab9961c5869a4295ada34918852b561604e1be281c22f2`
in `209026-skill-cancel-request-correlation-20261106.json`. The new private
14-capture bundle is
`/home/blizz/re/gw2/notes/209026-skill-repeat-producers-20261106.json`, SHA256
`14dd5fc7a5504df76a66e5935207db2a0eb70d860475566c6b8ff645a679f5c7`.
All captures exited0, were nontruncated and verified their owned-process
cleanup. No live target was attached or launched.

| Evidence | Scope |
| --- | --- |
| `ev_10455e922875caf65cae7e008b4a9736281beca68d1f532a178c0550fdad4ef1` | Character/player getter and exact0x29/0x2a table bindings |
| `ev_1377044a07f8d0350f47e802bc1e0c0fce1c97e71cd00053571b61fd1f2eaf62`, `ev_9f5fd9161fe110b2db953e9b9da3609a6b64ebc62ffd3bcd785f7c225a9cc6b0` | Complete player getter and character-to-player lookup |
| `ev_4aa9fff1ea41d7ccfa70b1fe590f42fc58c3976e550186c5b26b32530f39642b`, `ev_c8b65dc03584ec2c7386eb9d79a411c1831f0888e9352a2889152ea9d1b9ffb4` | Player constructor, embedded object constructor and eight mode tables |
| `ev_78038e2c9221306fbda86bfd28a69ff0a42dfc69328db10e4f3eb9daab506482`, `ev_09e359cf223f9b46bc8ee95ec8e58de2ee9e1846ec9ed12944c7d1e78fb53d21`, `ev_e9c7b81ff38794c9e92a3d2c70abefd6e049fddaf163c03324e248083d1d528b` | Exact +0xa0 target, complete update and named EnableFishing gate |
| `ev_fd94a686148b4373fc07c1a9b0648c16466951d57cd8cba4e9cf697ab121ff87`, `ev_329cd1f609245c0a93f1665e2293783b01536740531396fd5a223c7592defbfb` | Eight mode +0x30 targets and complete no-op stub |
| `ev_199261748c2ae6cc64af9e1ed972b7006ce907f59b3bb5e6549d23e401c21c05`, `ev_1a9f97567e5d114987826c9561f653ad3df43f26340a43d2a63e0c851c2d50c9` | Nontrivial mode method including its continuation through the common return |
| `ev_ffe01970d289b97a0f0b7a44203cee9edfc4c4aca7a5998ae4fb4b4af38a0825`, `ev_8f610ad0aa393488e3a1c870b7927509dd41b3a1904eb42f84e12fb4bda7b6b6` | Local candidate search, complete slot/target setters and complete0x1c0 caller |

The cached-slot getter and constructor-established skill-bar tables are
also retained in the preceding sword-chain bundle; the reused full
`0x1411f5040` caller appears in
`209026-skill-recharge-context-binding-20261106.json`, record
`ev_2e95765169eaac60f651dab102ffe9a9021b3a3922575470203d69b21daf7fc5`.
