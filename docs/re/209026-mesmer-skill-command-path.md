# Build 209026: Mesmer skill request and response boundaries

Static REA, 2026-11-05, against the frozen executable SHA256
`99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`.
This follows [explicit healing-skill selection](209026-mesmer-starter-skill-requirements.md)
and [client 23 framing](209026-agent-behavior-directions.md). No client was
launched, no new runtime capture was made, and no gameplay reply was sent.

The trace establishes a skill request inside client 23, a separate active
skill-order representation, correlated server 84 order updates, and
per-skill recharge messages. It does **not** establish a complete minimal
server reply sequence for executing skill 10176.

## What run 116 establishes

The retained run 116 `server4.log` records three client 23 frames of
15/23/15 bytes after the isolated key 6 input. That process used
`--no-capture`; the log contains IDs and lengths, without plaintext
payloads. These entries cannot identify the packed opcodes, skill keys,
slot bytes, targets, flags or behavior values. A size match to any layout
below is not a recovered request.

## Exact request suitable for a future diagnostic

Client23 is the generic behavior envelope. Serializer `0x14103df90`
selects23 for context type1 and84 otherwise, with identical outer field
kinds. The tick, behavior value and raw buffer remain distinct. See the
[direction/framing note](209026-agent-behavior-directions.md).

The character's embedded order object is at +0x4b0. Its vtable
`0x142179b90` +0x1d8 is wrapper `0x141283f20`; +0x1d0 is producer
`0x141283f90`. The diagnostic path identifies `ChCliOrder.cpp`, and the
producer asserts `m_isControlled`.

The wrapper gets character primary virtual +0xc0. This is
`0x1411d86d0`, the same target as +0xc8, returning character +0x520
(skillbar). Skillbar virtual +0xe8 is `0x1411f8530`, which asserts
`skillbarSlot < arrsize(m_slotSkillDefs)` with bound23 and returns
`m_slotSkillDefs[slot]` from +0x1d0. The wrapper forwards this skill
pointer and the same slot to the producer. Thus the trailing byte below
has a traced skillbar-slot source; it is not the physical key number.

On the ordinary branch at `0x141284202`, the producer allocates exactly
10 bytes and fills:

| Packed offset | Representation | Proven source |
| --- | --- | --- |
| 0 | u8 | Opcode0x23. |
| 1 | little-endian u32 | Supplied target object's +0xc, or0 when no target is supplied. |
| 5 | little-endian u32 | SkillDef +0x28, the skill data key. |
| 9 | u8 | Supplied skillbar slot. |

It passes outer command flags3 and the whole behavior container through
base-agent virtual +0xa8 (`0x14102d3b0`), which allocates the local kind29
command using `0x141038720`. Its serializer is the client 23/server 84
serializer above. The local behavior value is independently initialized;
it is not the skill key or slot.

For the explicit fixture, a captured request with key 10176 and slot 0
would establish that the selected heal reached this path. Target0 is an
observed encoding for an absent supplied target, not yet a validated
server self-targeting policy. The producer has content/local-state gates:
a null skill returns, and a flag check involving SkillDef +0x38 mask
0x80000 and order +0x58 bit0 can return before sending. Merely displaying
an icon does not remove all action prerequisites.

A second branch in the same function emits opcode0x24, length22:
byte opcode,16 bytes returned through a position/target query, u32 skill
key at17, slot byte at21. Related producer `0x1412842e0` emits0x25/0x26/0x27
with lengths22/34/38 and additional positional/scalar data. These layouts
are not needed to recognize the exact0x23 form; their detailed coordinate
semantics and use by10176 remain open. Unknown opcodes should remain raw.

## Active skill order and why server 84 alone is insufficient

Order factory `0x14127ffb0` switches on packed opcode0x50..0x7e. The table
at `0x1412816fc` maps0x78 to `0x1412813eb`, allocating an0x110-byte object
through constructor `0x14127da80`. This is distinct from the locally
predicted object created by the request producer: `0x14127de10` gives
that object kind0x53.

The0x78 constructor requires exactly13 packed bytes. Base constructor
`0x14127d9b0` copies the behavior value/container and takes opcode and
local order flags from the first two bytes:

| Packed offset | Representation | Inspected use |
| --- | --- | --- |
| 0 | u8 | Opcode0x78, stored as order kind. |
| 1 | u8 | Order flags, stored at order +0x14; separate from outer command flags. |
| 2 | little-endian u32 | Skill data key, resolved through content virtual +0x230. |
| 6 | u8 | Skillbar slot, stored at order +0x80;23 bypasses a slot mutation. |
| 7 | u8 | Raw value stored at order +0xf4. |
| 8 | little-endian u32 | Agent identifier resolved through `0x141024060`, held at order +0x60. |
| 12 | u8 | Bit0 adds order flag0x8000; bit1 selects the skill action variant passed to `0x14129a4d0`/`0x14129a1a0`. Other bits are not named here. |

The constructor requires a type0 skill-body variant and a nonnull selected
action, then initializes the action object at +0x88. It can call
`0x1411fb5b0(skillbar, slot, 1)`. These are source constraints, not a claim
that zeros are valid for the unnamed bytes or that every skill accepts
both action selectors.

The common command executor `0x141025e80` invokes character-bound virtual
+0x88 only when command flag bit12 is set and bit17 is clear (and its
other completion guards pass). The character +0x48 interface maps this
to `0x1411dc070` -> `0x14127ef80` on embedded +0x4b0. That routine chooses
one of two active-order entries using outer flag bit5, checks/replaces
existing orders, and can reach the factory above. Packed order flags also
have their own gates, including mask0x10 and mask0x4.

**Server 84's constructor always ORs0x12000 into its command flags.** It
sets bits16/13 and does not set bit12. Its received flag byte cannot alter
those high bits. With these flags it does not reach this order-start callback
through the inspected common executor with those constructor flags. Echoing a0x23 request as84, or
placing0x78 in84, is not a demonstrated skill execution mechanism.
The valid carrier and scheduling for an authoritative0x78 order remain
the next missing source link.

## Correlated order notifications through server 84

Server 84's character callback `0x1411dcfa0` uses outer flags mask0x10 to
select response dispatcher `0x1411dd4d0`. Its verified selector tables
map these opcodes to the following order helpers:

| Opcode | Target helper | Payload/use |
| --- | --- | --- |
| 0x44 | `0x141286820` | Exactly2 bytes: opcode and raw result byte. Stores order +0xe8 and dispatches character event through `0x1411db500`. |
| 0x45 | `0x1412868b0` | Does not inspect a payload length/detail beyond outer opcode dispatch. Either sets deferred flag0x2000 on an order with flag0x100, or dispatches `0x1411db540` and finishes through `0x14127ff00`. |
| 0x47 | `0x141286940` | Exactly3 bytes: opcode and two raw bytes. Calls `0x141287410`, updating +0xf4, action timing and order flags from the second byte's bits0/1. |

All three select `order[(outer_flags >> 5) & 1]`, require an existing
order of kind0x78, and compare its copied behavior value at +0x20 with
the incoming behavior value. A missing order or mismatched value is
ignored. These establish correlation and mutation; they do not establish
success/error enum names, mandatory ordering, or which notifications
10176 needs. The outer flags mask0x10 requirement is separate from the
packed0x78 order flags.

## Per-skill recharge is a separate message family

Registration and descriptors at `0x142170a60` onward identify:

| ID | Fields after frame ID | Handler -> skillbar helper -> recharge helper |
| --- | --- | --- |
| 630 | Dword character, Dword skill key, Byte category, Dword remaining, Dword total, Word raw value | `0x14125c2f0` -> `0x1411fb470` -> `0x141214030`. |
| 631 | Dword character, Dword skill key, Byte category, Byte pause selector | `0x14125c3e0` -> `0x1411fb4b0` -> `0x141214470`. |
| 632 | Dword character, Byte category, Byte secondary selector, Dword remaining, Dword total | `0x14125c4b0` -> `0x1411fb740` -> `0x1412145d0`. |

The handlers require an existing skillbar;630/631 resolve the skill key
through content virtual +0x230. These are outer protocol Dwords/varints,
not the fixed-width packed request integers.

630's helper uses the object at skillbar +0x150. It finds a record by
skill pointer and category, clamps remaining to `min(remaining,total)`,
and stores remaining at +0x10, total at +0x20, category at +0x24 and the
raw Word at +0x28. A zero clamped remaining removes a matching entry.
It advances existing recharges, reorders the list and reschedules using
the world-millisecond/rate machinery already traced in the
[skillbar note](209026-character-skillbar.md#recharge-rate-scale). This
supports the remaining/total timing role. The complete category enum and
raw Word semantics remain open. Category1 additionally checks a skill
content value at body +0xf0 or a context-specific row at +0x190/count198;
a zero value skips the update. No category or retail duration is chosen
for10176 here.

631 finds an existing record by the same pair and changes the pause flag
at +0x2c, advancing elapsed time when pausing and rescheduling.632 mutates
a separate recharge list keyed by its two byte selectors and supplies no
skill pointer through this handler.627 only changes recharge rate and
cannot substitute for initializing a per-skill record.628 is also not a
recharge setter: it stores a skill pointer and byte at skillbar
+0x1c8/+0x1c0 and rebuilds.

## Numerical healing and remaining experiment boundary

Existing [health updates](209026-character-health-endurance.md#existing-health-messages-257260)
provide server 257 (character, first current, second current) and259
(character, two current/maximum pairs, unnamedF5). These mutate existing
meters. They do not validate a skill request, create an order, determine
10176's healing amount, or communicate every combat event. A meter
increase alone would not demonstrate that the heal executed correctly.

The next useful runtime evidence is a bounded client 23 diagnostic that
records decoded outer fields and recognizes only exact0x23 length10.
That can distinguish the actual key 6 request from surrounding behavior
traffic without interpreting lengths as opcodes. Before a cast reply,
the missing work is the authoritative order carrier/flags/timing,
10176's action-variant/content timing and heal formula, recharge category
and raw Word meaning, and the required correlated events. This note does
not claim a playable combat loop or propose guessed acknowledgement bytes.

## Evidence and retained limits

Private bundle:
`~/re/gw2/notes/209026-mesmer-skill-command-path-20261105.json`
(720 records, 17,100,028 bytes; includes parallel session evidence).
Hash-checked helpers and their selected objdump windows are retained
under the same private notes directory. Public documentation contains
addresses, layouts and conclusions, not a raw binary/data dump.

| REA evidence | Observation |
| --- | --- |
| `ev_7653a404eed112bfa3c2a1bfaa44525cb566734be199dd2ccb3abdcefc34c13e` | Exact shared client 23/server 84 serializer. |
| `ev_af8489e8f592e0af3ba669bb024c5f3411eb18eda534671a07585c1d2336f172` | Slot wrapper and packed0x23..0x27 producers. |
| `ev_3f2a66e598c3972168c449554d0d69045fba7b20a2e5829429a30024ac0e3794` | ChCliOrder source/assertion strings and response dispatch entry. |
| `ev_a09000b0e7a8cf3fa22c172c2e9f4608ad7d132bb7a7174f9a080c6d52dda1b2`, `ev_df41f468a29276c9351ddcf7e7ef5d9243ff2af7842f21daa63fe1950267a9a5` | Order/skillbar virtual bindings and character getter bindings. The behavior-name table entries tested are empty strings. |
| `ev_0a866958214b595665a126433578c7c127555d1fdde01355b3f0854456232d6e` | Character getter and embedded order callback thunks. |
| `ev_d86b63f4d56fe74e7b2e4a5ec74b45598d3b002fcdca0bf61464684809cbd947` | Named skillbar slot lookup and630..632 handlers. |
| `ev_c5944b77eff87c9b948bfc97768a07466683d160009fcd15e38312c42f70fd00`, `ev_e15156b16d8df039dc95d003fd0094802ece12bd2c0b4b6b537f543ae1cb7bc3` | Exact0x78 factory branch and allocation. |
| `ev_b15d022689140fcdd8a8517a7417f13300f8eff1397f29afc61c9714b478c279` |0x78 constructor and packed fields. |
| `ev_3a7fc3cc8427f5b8bbec2dc53f8bb1d520b5068c95a9b9a5111bf3fdac574902`, `ev_b0864fa711be5aa662d98ccfcc6bd27dd2def19e6c0bdf0695a084dbe1327dd2` | Common executor callback gates and order-start path. |
| `ev_fc3f7fff13c1286db27266335ec63abf244e8b738ad3fa2d491a61f4064c24b0`, `ev_636ecbb257e917f5e05e57f0efcf4bb0f98d59f37074b27cf26d3c340524f393` | Kind29 constructors and the unchanged fixed0x12000 flags stored by the common constructor. |
| `ev_428f9de3a778298303dc29707113e26a6e6c77f8cc363b5eeab821966f254164`, `ev_770913ec20372f04a3f9bad754b44228f17ece46f04cfcf6b12cf497faac14ea` | Response opcode table and dispatch to order helpers. |
| `ev_14e6f473422745215a2f8659accac9333330648cd731f2bc72deff3b2112927e`, `ev_8fa8d4c9fae7a544e2680b6447b20e2830012d291450e0bd4f7572bd54b5c8b9` | Correlation gates and0x44/0x45/0x47 bodies. |
| `ev_2f2869abdc5e2a7c12cc27e987329251a2a491e6a8591beae9edd256ae113bec` |0x47 mutation helper and bounded skill-order start. |
| `ev_95ced15a82a6808604cf2b5da4dae4937da35684c9d9bc2acf053192e3596821`, `ev_7e5b83c381cf7074fd33c43221dd0eb2558ec218d3301e318bc6b7ccfaeb5d98` | Recharge forwarding and complete630/631 mutation helpers. |
| `ev_7b83f99f50a2e29bec2962e3cc9b44680c4763074f49139cc32fcd568f7153a5` |632's separate recharge-list mutation and behavior-name getter. |

The wider resize-helper census was only a locator; its truncated output
is not a complete producer inventory. Several pdata entries split logical
functions, so conclusions above use explicit aligned continuation windows
where needed. No complete key 6 input-handler-to-network replay, server
implementation, or cast-success observation was obtained.
