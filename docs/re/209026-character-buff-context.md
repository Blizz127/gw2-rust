# Build 209026: constructor-owned character buff context

Static REA follow-up to [endurance deficit](209026-endurance-deficit.md),
using the same frozen executable. No client launch or message replay.

## Construction supplies the context

Character constructor `0x1411d2810` first clears whole-character +0xd0 at
`0x1411d2945`. Later, at `0x1411d2df7`, it obtains the global service from
`0x1412c0550` (global context +0xb8), then calls that service's virtual
+0x18 with the character's +0x40 interface. It stores the returned pointer
at character +0xd0 (`0x1411d2e1a`).

The service constructor at `0x1412c0570` installs vtable `0x142183dc8`;
its +0x18 factory is `0x1412c05f0`. That factory requires a nonnull owner,
allocates 0xf8 bytes and calls `0x1412c2560`, passing the owner's +0x40
interface. The latter constructor's diagnostic source string at
`0x142184430` ends in `Game/Combat/Cli/CmbtCliBuff.cpp`. It installs primary
vtable `0x1421841a8`, retains the owner at +0xb0 and creates empty local
collections. In particular, it stores null at +0x28, the backing pointer
used by the membership predicate below.

At the constructor tail, the buff context subscribes its +8 interface
through the owner's virtual +0xf8. For the character +0x40 interface this
is `0x1411db920`, the listener insertion already traced in
[core stats](209026-character-core-stats.md). That insertion itself does
not dispatch the new listener. This identifies a constructor-owned buff
context, rather than an additional network-created component needed just
to obtain a nonnull pointer. It assumes successful normal allocation and
the initialized global service; it is not a complete bootstrap proof.

## Membership predicate used by endurance

Character +0x40 virtual +0x50 is `0x140400100`, returning interface +0x90
(whole-character +0xd0). The buff context's primary virtual +0x58 is
`0x1412c3850`:

1. A null backing pointer at +0x28 returns false immediately.
2. Otherwise, it scans +0x20's Dword number of 0x18-byte entries, skipping
   entries whose Dword +0x10 is zero.
3. For an occupied entry, it loads the object at entry +8 and compares
   that object's pointer at +0x10 with the requested skill pointer
   (dereferenced from the caller's argument).
4. Any exact pointer match returns true; exhaustion returns false.

The inspected method does not call into other components or apply a
level/locale test. The constructor's null backing pointer therefore makes
this predicate false before any later population. It does not establish
that the context remains empty after all later character callbacks or
server messages.

The [endurance lookup](209026-endurance-deficit.md) requests the frozen
skill data key 783, UID 69789. Thus its exemption gate is a buff-context
membership test for that exact record. The record's human name remains
unresolved. The later [instance lifecycle trace](209026-buff-instance-lifecycle.md)
binds population and removal to registered messages 734–739; it supplies no
default gameplay values or policy for applying this particular buff.

## Evidence

Private bundle: `~/re/gw2/notes/209026-character-buff-context-1.json`
(1,289 records, 26,799,337 bytes).

| REA evidence | Observation |
| --- | --- |
| `ev_55252a3c31d91378a9bb82a5fa95a97cf2a52905afe65cdcde5ef81298a48401` | Character interfaces and initial null +0xd0. |
| `ev_8e718d14c2aec22d414d5ea7dc17cccf741c8a339fb0fe3e4e7bf5524a299995` | Factory call with character +0x40 and result store. |
| `ev_a781cec0e696da7ed67bb25ac05b76434691d62e8bae482d04efaba4e60050ee` | Global-service getter and service vtable installation. |
| `ev_f4b7ed10b27533e3a695cc2b14b65ce110233c9bdcca84451031f87c25937c7f` | Service factory vtable entry. |
| `ev_cf1609d2a6c8c4425497d6689664ce6bbe425bfc545f5683452adcabea08cca8` | Complete 0xf8-byte factory and owner check. |
| `ev_1dd686ab907ca131e2fedd13e047836736580c78075821efd10ca76e45925afb` | Buff constructor vtables and empty collection initialization. |
| `ev_bd79dd3232ec0015dc58365e2e2412cb24c2be3c5d05c8a47162a78f0ee14338` | Constructor tail, owner subscription and complete membership predicate. |
| `ev_6ce5c95869f0133ff087bb555ee91e6911a2c5b1d0096d6cb997afb2ce1bac53` | Buff primary +0x58 entry. |
| `ev_0842e54381587b6f057b823da9b0e1c2d188ab428eee62419ccb0410b62b728d` | Embedded buff source diagnostics. |

## Bounded service and owner-listener follow-up

The exact service-table lead was inspected on 2026-11-06 with target-free
REA/Rizin reads of the same pinned executable. No client execution or cold
analysis import was needed. Service +8 (`1412c0680..1412c0714`) resolves
the supplied agent's existing context: type0 uses the character interface's
virtual +108, and type10 uses the alternate interface's +50. Service +10
(`1412c07f0..1412c080d`) is only a null/type0-or10 predicate. Neither method
populates the buff collection.

The constructor-installed buff+8 listener table `142184220` has two
nontrivial callbacks among its thirteen inspected entries. Callback
`1412c4980..1412c4aaf` records a raw state and walks occupied entries only
when a transition involves state1. For a tag1 SkillDef with body+4 bit40,
it calls stored-owner virtual +140 on entering state1, or +128 on leaving
it. The body does not insert or remove buff instances. Neither the state
nor that flag has an established gameplay name in this trace.

Callback `1412c4ab0..1412c4b5f` verifies the owner, unsubscribes and clears
the stored owner. It is a lifetime callback, with no demonstrated network
removal input. Primary virtual0 (`1412c3110..1412c3132`) separately reports
unsupported client spawn scaling and returns zero; this does not classify
the remaining primary methods.

That service/listener slice closed without establishing a buff packet,
instance insertion or targeted removal. The primary-method follow-up below
subsequently recovered the typed message-to-mutation path. The separately retained Ether Feast
“Conditions Removed” display fact does not supply classification,
selection, delivery or an authoritative removal policy.

Private findings: `209026-buff-delivery-typed-boundary-findings-20261106.json`,
SHA-256 `3301d3d6fb811080c2ab3dea120f825210a244dd37c39c91ed0347838bee671c`.
Complete evidence: `209026-buff-delivery-typed-boundary-evidence-20261106.json`,
SHA-256 `15ea84ba0d4254552393935ee30a4430ed958cbe899070e6f2cb08802db84611`.
The four captures are `ev_c43247ae07e0942853d1903a5f1efdbee470e4d7ddb4e783771c56f6324a876b`,
`ev_9d6b90c2e808e35a6dd1cd96b9cf66f5411d4f04fdd9274bc98718a823beb394`,
`ev_e5e98c1b9399d0d593c85706ce2b606a686aa576f09c5831c97b9d8df89a7bd5`
and `ev_21ec3066c92871ccebe121a061f5e72af9fc90cc19771552c403e54bf31821b1`.
All exited0 with complete output and unchanged executable hashes; bounded
adjacent code and Rizin's string-region warnings remain in the raw evidence.

## Registered instance producers

The subsequent [lifecycle trace](209026-buff-instance-lifecycle.md) follows
direct `CmbtCliMsg` handlers from registration group `1421840e0` into the
same constructor-owned context. These mutations are not insertion slots on
the inspected primary query/iteration vtable. Message field 0 resolves the
combatant through `1412c0720`; combatant virtual +0x40 supplies the context.

735 resolves a type-64 SkillDef with a nonnull tag-1 buff body, constructs a
120-byte instance through `1412c23a0`, and inserts its buff ID, pointer and
occupancy into the 24-byte collection at `1412c3a60`. It asserts that the buff
ID is absent. 734 activates an existing instance, 736 deactivates it, and 737
updates its raw value. 738 removes all instances matching a skill key; 739
removes one buff ID. Their shared removal `1412c4250..1412c4810` clears the
occupied entry, updates the indexes and notifications, releases the source
reference and frees the instance. The source agent/skill key/buff ID and
remaining raw fields are distinct; repeated creation is not an established
idempotent update.

740/741 belong to the same registration group but insert/remove a separate
12-byte value array, not these buff instances. All eight individual packet
descriptors terminate and agree with the extracted XML; the registration
array itself is not claimed to have a null terminator.

This closes the wire-population gap, not the gameplay rules. Duration units,
raw byte 5 semantics, stacking policy, the remaining boon-class mapping and
authoritative removal selection remain unresolved. The condition-count subset
is bounded below. Raw byte 6 is stored
without Boolean normalization in the instance field named `isActive` by a
native assertion. The existing visibility/state bits and Ether Feast tooltip
facts do not substitute for those missing rules. No buff sender or effect is
enabled by the raw 734–739 codec.

Complete evidence: `209026-buff-lifecycle-source-evidence-20261106.json`,
SHA-256 `38c9fceeecd9348345e8c8b45f6c654491197a58ab0acbd677dc8357aac5bb5d`.
Findings: `209026-buff-lifecycle-source-findings-20261106.json`,
SHA-256 `f30bada6103c458a1efda97c2d9a329845985c5090d73f07ed794a98048dc724`.
The linked lifecycle note records exact fields, complete handler/mutation
boundaries, descriptor evidence and the separate classifier limit.

## Named condition-count subset

The creation/removal callers pass the resolved tag-1 buff body to complete
counter methods `1412c2c90..1412c2da9` and `1412c2db0..1412c2f5c`. Body Dword
+0x14 keys a separate 16-byte-row collection at context +0x80. Each row tracks
total count and a subset incremented/decremented only when body Dword +0x18
equals 2. The removal assertion at `1412c2f39` explicitly names the subset
`buffStatusKey->conditionCount` (`CmbtCliBuff.cpp`, line 272). This supplies a
named condition-count relation beyond the earlier UI styling predicate.

Both methods skip status key 94; a zero total deletes the whole status row.
This does not establish an all-status condition-instance total, the meaning
of other +0x18 values, or any authoritative cleanse eligibility, priority,
count or timing. In particular, neither a boon class nor Ether Feast's
removal policy follows by elimination. The [lifecycle note](209026-buff-instance-lifecycle.md)
records the exact counter fields and branches.

Private findings: `~/re/gw2/notes/209026-buff-status-class-counter-findings-20261106.json`,
SHA-256 `4ae0a219818b72890a000f2813147abbc2de4f9fdb78dd7bea6175598bc9ca03`.
Complete raw capture: `~/re/gw2/notes/209026-buff-status-class-counter-edge-20261106.json`,
SHA-256 `9ac171ae85a80c77ef2a87a8f05e8308e8f30cdefe50f9d1cb833848a3fc2fb5`;
REA `ev_408381750df6698a3facd781a79bd951d305437f7a7306c6d915bdf6cda9c717`.
The target-free capture exited 0 with complete, nontruncated output and the
same frozen executable SHA-256; no client or archive access was used.
