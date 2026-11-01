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
