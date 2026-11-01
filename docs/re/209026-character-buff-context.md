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
membership test for that exact record. The record's human name and the
wire operations that populate this collection remain open. No buff
creation/update frames or default gameplay values are introduced here.

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
