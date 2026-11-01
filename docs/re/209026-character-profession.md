# Build 209026: profession component creation and finalization

Static REA findings, 2026-11-01, for the frozen executable identified in
[mission transfer](209026-mission-transfer.md). No client was launched.

## Chatter registration requires profession

Server 170's finalizer `0x1411da260` calls `0x141276660` with character
+0x330. The helper's diagnostic source is `ChCliChatter.cpp` at
`0x142178680`; its initial assertion is `!m_isRegistered`. On the player
branch it registers listeners through the following getters, dereferencing
each result without a null guard before the virtual subscription call:

| Getter | Resolved storage | Consequence |
| --- | --- | --- |
| Character +8 interface, virtual +0xb0 (`0x1411d75f0`) | Whole-character +0x388, coreStats pointer | Already established coreStats prerequisite. |
| Character primary virtual +0x108 (`0x1411d74b0`) | Address of character +0x40 | Embedded interface, not a separately allocated component. |
| Character +8 interface, virtual +0xc8 | Whole-character +0x3f0, inventory pointer | Independently confirms inventory prerequisite. |
| Character primary virtual +0xa8 (`0x1411d85c0`) | Pointer at character +0x510 | Assertion names `m_profession`; chatter calls its virtual +0x100. |
| `0x1411d8520` | Associated player lookup | Player must exist; helper also explicitly asserts its result. |

The player's virtual +0x1b0/+0x368/+0x3a8 resolve to `0x1411bace0`,
`0x1411bb390`, `0x1411bb3e0`, which return embedded addresses +0x218,
+0x9978, +0x9be0. Player construction initializes them at
`0x1411b9e34`, `0x1411b9fc9`, `0x1411ba000`. These calls do not establish
additional network component creation requirements. Chatter also registers
with character/player event lists and a global object; this is not an
exhaustive proof of every listener's later behavior.

Character construction clears +0x510 at `0x1411d2cbd`. The automatic
allocation at `0x1411d2e39..0x1411d2ec9` occurs only when identity masked
with 0xf0000000 equals **0x20000000**. The established player identity
uses **0x30000000**, so that allocation does not satisfy this player-path
dependency. It constructs a 0x48-byte generic profession through
`0x141288bd0`; it is not a substitute for a player profession message.

The separate coreStats profession field at +0x12c does not itself store
the profession component pointer. Its setter `0x141209440` changes the
scalar and invokes attribute-update/notification routines. Thus merely
supplying the profession byte in server 182/183 is not evidence that the
separate +0x510 component exists.

## Creation messages

Nine handlers allocate a derived profession, construct it with the raw
character pointer, then call `0x1411e21a0`. That setter asserts
`!m_profession` (`0x142161e18`) before storing character +0x510. These are
alternative creation paths, not nine packets to send to one fresh character.
The base constructor `0x141240720` stores the character at component +0x38
and initializes listener collections and state.

All descriptors below are protocol 0, group 0, and begin with a **Dword
character index**. Field lists omit that index. Dwords use varints; Floats
are four-byte IEEE-754 values. Individual payload meanings and useful
initial values remain unresolved.

| Server ID | Descriptor | Handler | Constructor | Fields after index |
| --- | --- | --- | --- | --- |
| 473 | `0x1425caa20` | `0x1412560d0` | `0x141293fd0` | 3 Dwords |
| 469 | `0x1425caed0` | `0x1412562f0` | `0x141294520` | 3 Floats |
| 477 | `0x1425cb370` | `0x141256520` | `0x141294780` | 3 Floats, 2 Bytes |
| 481 | `0x1425cb9a0` | `0x141256770` | `0x141294fa0` | 2 Bytes |
| 484 | `0x1425cbdd0` | `0x141256920` | `0x1412952d0` | 2 Floats, 2 Bytes |
| 488 | `0x1425cc500` | `0x141256c00` | `0x141295840` | 3 Floats, Dword |
| 493 | `0x1425ccbe0` | `0x141256f10` | `0x141288c50` | Two variable arrays, each descriptor capacity 2, element Byte |
| 498 | `0x1425cd2c0` | `0x141257190` | `0x141295dc0` | 3 Floats, 2 Bytes |
| 504 | `0x1425cd9b0` | `0x141257510` | `0x141296640` | 2 Dwords |

493's array descriptors use kind 17, parameter 2, nested definition
`0x1425acd60` containing a Byte. The handler passes count/data pairs to
the constructor. This trace does not independently establish the array
count's wire width or all constructor count requirements; do not substitute
two unprefixed fixed byte pairs based on capacity alone.

The constructors install these primary vtables. Their virtual +0x40 slots
return distinct constants matching the [profession enum](209026-character-creation.md):

| ID | Vtable | +0x40 getter | Constant | Profession correspondence |
| --- | --- | --- | --- | --- |
| 473 | `0x14217baa8` | `0x140c91bf0` | 6 | Elementalist; direct `ChCliProfessionElemental` source references. |
| 469 | `0x14217bd48` | `0x140279450` | 3 | Engineer, inferred from enum correspondence. |
| 477 | `0x14217bee0` | `0x1402406f0` | 1 | Guardian; direct `ChCliProfessionGuardian.h` source reference. |
| 481 | `0x14217c0f0` | `0x140624ba0` | 7 | Mesmer; direct `ChCliProfessionMesmer.cpp` source reference. |
| 484 | `0x14217c2f8` | `0x14025afc0` | 8 | Necromancer, inferred from enum correspondence. |
| 488 | `0x14217c4b0` | `0x14025afb0` | 4 | Ranger, inferred from enum correspondence. |
| 493 | `0x14217a418` | `0x1404b89a0` | 9 | Revenant, inferred from enum correspondence. |
| 498 | `0x14217c668` | `0x140279c50` | 5 | Thief; direct `ChCliProfessionThief.h` source reference. |
| 504 | `0x14217c898` | `0x140278c40` | 2 | Warrior, inferred from enum correspondence. |

The four named implementations support the shared-enum inference, but a
direct named consumer of every remaining +0x40 getter was not established.
The neighboring +0x18 virtual returns different constants (1, 6, 11, ...);
those must not be confused with these profession values.

## Guardian meter and update layouts

The prepared local character candidate is profession 1. Its creation
handler 477 calls `0x141294e90` with the three floats in wire order, then
`0x141294f70` with both zero-extended bytes. Guardian construction clears
the relevant fields; no playable defaults are supplied by that zeroing.

| Creation field | Storage | Observed meaning/use |
| --- | --- | --- |
| Float 1 | Component +0x84 | Meter maximum. |
| Float 2 | Component +0x88 | Meter rate. |
| Float 3 | Component +0x80 | Meter current, clamped for ordinary finite inputs with nonnegative maximum. |
| Byte 1 | Dword +0x90 | Raw value, exposed by Guardian secondary virtual +0x20. Gameplay name unresolved. |
| Byte 2 | Dword +0x94 | Raw value, exposed by secondary virtual +0x28. Gameplay name unresolved. |

The setter first changes maximum, evaluates the old meter while changing
rate, then replaces current with the supplied third float and records a
fresh timestamp at +0x8c. The shared evaluator `0x141288dd0` uses this
four-field meter at component +0x80. For ordinary finite values and
nonnegative maximum:

```text
if now == 0: return stored_current
return clamp(stored_current + f32(u32(now - stored_timestamp))
             * f32_bits(0x3a83126f) * rate, 0, maximum)
```

It does not mutate the meter while evaluating. Time source `0x1409cb4a0`
uses an optional callback at `0x142682360`, otherwise imported
`WINMM.dll!timeGetTime` at IAT slot `0x141915110`. The multiplier is f32
0.001. Thus the fallback clock gives rate units per second; no claim is
made here about an installed callback's runtime behavior. This is distinct
from the health meter's world-time path.

Guardian primary virtual +0xf8 (`0x141294980`) divides the evaluated
current by maximum **without a zero-denominator guard**. A zero-filled
creation payload therefore is not established as a useful default. The
first byte is also compared against an integer requirement in
`0x141294870` under a separate character-state condition. This trace does
not name the meter as a particular profession resource or resolve those
state and byte semantics.

The following operations begin with Dword character index:

| ID | Descriptor / handler | Fields after index / behavior |
| --- | --- | --- |
| 478 | `0x1425cb580` / `0x141256610` | None. Calls profession removal `0x1411d6a00`, invokes the component deleting destructor and clears character +0x510. |
| 479 | `0x1425cb6a0` / `0x141256670` | **Five Floats**. The handler passes only the first three (maximum, rate, current) to `0x141294e90`; the two trailing floats are part of the descriptor despite being unused by this handler. |
| 480 | `0x1425cb830` / `0x1412566f0` | Two Bytes. Calls `0x141294f70`, storing both and notifying listeners. |

479/480 fetch the existing profession through character primary +0xa8 and
call Guardian methods directly. They do not create missing state or check
the derived profession type in the inspected handlers. Their valid use
therefore requires the matching existing component; a framing codec cannot
establish this lifetime/type condition.

Private bundle: `~/re/gw2/notes/209026-guardian-meter-1.json`.

| REA evidence | Observation |
| --- | --- |
| `ev_2573d7d1ad13ca3e6045c3097c68f4acee3b94c2f2be491e235e494fa3062b19` | Creation arguments, meter and byte setters, vtables and zero initialization. |
| `ev_dd3e17fb42814b4d6cc9db4843f25a3ac92e1327ff7a017c9be6f13884ac72ca` | Meter evaluator, ratio getter, conditional byte use and clock wrapper. |
| `ev_cf4a421cf1d4d3c282942d5fd292d206107f44f0cd0ce3aff9541b61d69fd2be` | timeGetTime import, update/removal handlers, five-float 479 descriptor and raw-byte getters. |
| `ev_cd25a14fbd23545b97767a6fc979e04256188b2c2d71ea706dbc62213f5407ea` | 480 descriptor, 0.001 constant, removal body and bounded clock-reference search. |

### Specialization-dependent byte check

Guardian requirement predicate `0x141294870` requires evaluated meter
current >= its first integer argument. It additionally compares the second
integer argument (signed <=) with stored first byte +0x90 only when
`0x14122aa20(character +8)` returns **3**. The second byte +0x94 is not
read by this predicate. Secondary-interface getters `0x1403481d0` and
`0x14038be00` return the two stored values; this does not name them.

The condition is now traced to specialization state:

1. Character +8 virtual +0xe0 (`0x1411d84a0`) returns its associated player,
   treating player index zero as absent.
2. Player virtual +0x30 (`0x1411bb3f0`) returns embedded player +0x9cc0.
   Its constructor `0x141224c70` installs vtable `0x14216b2f0`; diagnostics
   identify **ChCliSpecialization.cpp**.
3. Specialization virtual 0 (`0x141225360`) gets slot 2 through virtual
   +0x30 (`0x141225440`), reading the pointer at component +0x68. If present,
   it returns the content Dword at +0xc8.
4. If slot 2 is empty, it reads the player's owned character, obtains its
   profession through primary +0xa0 (`0x1411d8610`, assertion **m_profession**),
   and calls profession virtual +0x18. Guardian's entry is `0x1402406f0`,
   returning **1**. Missing player/owned character yields zero along the
   inspected guards; an owned character with missing profession is not guarded.

Construction clears all three specialization pointers at +0x58. Therefore
an unconfigured player with an owned Guardian reaches discriminator 1,
not the first-byte comparison branch for 3. This does not establish
playable defaults for either byte or the meter, nor name the discriminator
values. Profession virtual +0x18 is distinct from the profession enum
getter at +0x40, even though Guardian returns 1 from both.

The separate primary Guardian predicate `0x141294920` accepts its argument
values 12..14, and accepts 15 only when this same discriminator is 5.
Its argument's complete meaning and callers are not resolved here.

Private bundle: `~/re/gw2/notes/209026-guardian-specialization-state-1.json`
(1,097 records, 23,946,681 bytes). A broad disassembly caller search timed
out; its only retained useful output was a vtable pointer. A hash-bound
E8/E9 candidate scanner subsequently located direct references, but those
unconfirmed candidate boundaries are not used to assign gameplay names.

| REA evidence | Observation |
| --- | --- |
| `ev_ef9fda63d669459d236a44fc746aa84e9e87b026fb36e021020ddb24ab1cc05a` | Guardian requirement predicate and primary/secondary vtables. |
| `ev_9f22c9fa9292ad0741a9f5a76fb2818c35bd6b1d192cf59e161a3c3c148fdf8f` | Null-checked specialization wrapper and secondary methods. |
| `ev_f55a30d294b8db9a80cc9d7934a5b810815afc461367fccfed7c6541da8e00f2` | Character associated-player getter. |
| `ev_3d89ccbccce7b98ff62195f695e179bc9cbc34844b5c120081c21d380daa9584` | Getter tail and player vtable installation. |
| `ev_593f297b683d36518cd7882541898b7b24f97c0d6ced560a72f67fd672fa1b15` | Player getter slots and both byte getters. |
| `ev_a6d2ad574b1c3142a88b7d4d53ea661306d824018f00dcdf21267b04a1314567` | Embedded player getters and specialization construction call. |
| `ev_670d9c210b55cf17f5e078915912881d6a64db4797f55d8c3e0c341213e67974` | Specialization constructor and zeroed arrays. |
| `ev_4920ed379241834e9c480b4313c2767ba18f32aa62caf0235e83edd8cbdad870` | Specialization vtable and named source/assertions. |
| `ev_7723cbb7f98d156a6ccce2a75961b23d7ee56f5c7d3875badf7ed1bfa0ebe8a2` | Slot-2 discriminator and owned-profession fallback. |
| `ev_4053d0234749734fa812e23f7fc75309abbc7e6b843e460cb4d78e7920b8f048` | Specialization slot getter and profession getter target. |
| `ev_3d8b2de1f4e2ed4306837ef2c17c237cb71cdd3a03ac2838fb1cd54d98b9aaf4` | Profession fallback dereference and assertion. |
| `ev_c607878594c0a4cde2be0faa2f54b6645b8357ef174cb8423abef7531f982248` | Constant-one getter, m_profession literal and candidate references. |
| `ev_a73d3cd0baa3e42e14ef0ef70db09e43b4d8fe3126f4c97246480abca7419273` | Guardian primary predicate using discriminator 5. |

### Guardian creation callback boundary

The complete 477 handler constructs the component, stores it through
`0x1411e21a0`, sets the meter, then sets the two bytes. The pointer setter
only checks `!m_profession` and stores +0x510; it dispatches no creation
event. Base constructor `0x141240720` stores the character and initializes
its own state without retrieving another character component. Guardian
constructor `0x141294780` initializes a separate listener collection at
+0x50, with its head +0x58 initially null.

Byte setter `0x141294f70` dispatches that collection through `0x14026dce0`
and trampoline `0x140255568` (listener virtual 0). The dispatcher inserts
an iteration marker, but with an initially empty head it skips the listener
call loop and removes the marker. No registration occurs between construction
and this setter in the inspected handler. The meter setter only accesses
its meter state and the previously traced clock/evaluator.

Thus this fresh Guardian creation path does not introduce a coreStats,
inventory or skill-bar prerequisite. This does not describe later 479/480
updates after listeners register, other derived professions, allocator or
clock callbacks, or establish valid initial meter/byte values.

Private bundle: `~/re/gw2/notes/209026-guardian-initialization-1.json`
(1,083 records, 23,816,598 bytes). No client replay.

| REA evidence | Observation |
| --- | --- |
| `ev_dfa4e8e908df3419f44c258790f397202cf2584eeaf555fcac4f35bf72a24ade` | Full Guardian creation handler, constructor, pointer setter and initial field setters. |
| `ev_a5a151e011fc961fea08895c31218add8bc8e7ece2c4c319af48da64e688f4f9` | Base constructor, empty-list dispatch branch and virtual-0 trampoline. |
| `ev_79bb4ff4e657cfa66d56ff40f14a83fca38c8ba090f78b7701b6fdf585f3d297` | Dispatcher cleanup and return. |

## Typed Guardian framing check

`gw2_proto::guardian::GuardianFrame209026` covers 477–480 with an explicit
character index and operation enum. The meter names describe observed
arithmetic; the two bytes remain `values`, and 479's unused wire floats
remain `trailing`. All floats retain their original bits. The codec selects
no defaults and does not validate profession type, lifetime or gameplay
values. Input advances only after a complete frame succeeds.

Three focused tests and `cargo build -p gw2-proto --lib` pass. Hand-authored
fixtures exercise all four layouts, distinct maximum/rate/current values,
raw bytes 0/128/255, output prefixes, and negative-zero/NaN trailing bits.
A concatenated stream verifies the next frame begins after all five 479
floats. All truncations, invalid neighboring IDs and overflowing/overlong
character-index varints are rejected without advancing input. No client
messages were sent.

## Scope and evidence

For a fresh player, profession creation must precede server 170's chatter
registration. This extends the [known ordering constraints](209026-character-finalization-dependencies.md);
it does not establish a complete playable bootstrap or authorize passing
the pending User Agreement gate. No wire fixtures were sent or runtime
claim made. Further initialization consumers and profession payload values
remain to be traced.

Private bundle: `~/re/gw2/notes/209026-profession-prerequisite-1.json`.

| REA evidence | Observation |
| --- | --- |
| `ev_dc04dafc4618ab4536860ce447e13dc922819ce9d9fd18b21ff9d5eeb2e464a4` | Finalizer's unconditional chatter call. |
| `ev_8cace72ea3f49a3294b010cbe1aac11c8741a447a3f4b01b117d0b8950b34e3e` | Chatter's component dereferences and finalizer's other calls. |
| `ev_7ea0f55f335bbad7a556972e2f5b3a47a0a08ec10f60c3b75e8bbc644282de55` | Chatter source/assertion, vtable entries, remaining registrations. |
| `ev_c108d121bcbecc60f7e9f0bfba5d3ca3cd43f2a52b1276507c98f23022cfbb29` | CoreStats, embedded-interface and profession getters. |
| `ev_166666685f457e546cca240d91c36f58165f8e8675944b0633be76b4f3cf766e` | m_profession literal and initial null/allocation sites. |
| `ev_4c00ae400a141e61bec6db3e247394bf36b968f5388a6afb66a1dac5be362d49` | Constructor's identity guard and player embedded getters. |
| `ev_bf76600d21069dc0adda462eb593673be407160d4f64a888496aa9cf4f224df0` | Profession setter and player embedded construction. |
| `ev_1dee5bd39f3b9cd332414f862afbd9a83fc742008805443db14024d91705c8a7` | Nine creation callers of the pointer setter. |
| `ev_041b8f2e71018b7830f95228ed96f39b41cf3a48701689a42a5e570984553437` | Registration pairs. |
| `ev_e949ff1ea54a07502631b715fe6acff95f53538fb8ec79daacf9bc655f993ea0` | Hash-guarded descriptors and derived vtable installations. |
| `ev_f97b68243a1ab21fc43dff2939065bdf515d048d1e088d760686805eec54a31b` | Four named profession source literals, base constructor and nested Byte definition. |
| `ev_fdde1654b3b43681ecfdaaa647e92bf1c4660c86ae12282811762763862926c1` | Vtable bases and 493 count/data arguments. |
| `ev_518fc962ab94793d8ab6b6e4f4119db86b21641d9f9185c5942a7f8338b94b7a` | Complete Elemental primary vtable and constant getters. |
| `ev_211112a5b4327b63cd1fb69e8fa28ad8cfb7e510dbb0af4e295cb2293d4dd377` | Remaining +0x40 slot pointers. |
| `ev_5f1cb3f216d48e3fd8a7d9cfc9a7c06ac8aff5739f410046ba2f817e8b8c3a56` | Constant getter bodies and shared profession methods. |
| `ev_a5da6bedf33d25225d0d0b2b265c265d9e1ef511e7d30e6f623354902c1f6d8c` | Raw-character lookup at each creation handler and !m_profession assertion. |
| `ev_0acbda9dcfb2cef36d13d4d1273a35605d55c0eea38c114daa20329c3fa467be` | Separate coreStats profession scalar setter. |
| `ev_5ff878fda9c3f422b500afac18ec657a3f363c72590b1600e40696b9054485d2` | CoreStats setter's attribute-update callees. |
