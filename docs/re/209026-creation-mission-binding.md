# Build 209026 creation-to-mission character binding

REA source investigation, 2026-11-05, against frozen client SHA-256
`99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`.
This extends the [creation response path](209026-character-creation.md),
[mission preamble](209026-mission-join.md) and
[mission redirect](209026-mission-transfer.md). No creation/save playback or
new server implementation is claimed by this note.

## Selected character identity in the preamble

**The selected character GUID occupies bytes 48–63 (offset 0x30, length 16)
of the 84-byte mission preamble.** It is the second of the three adjacent
16-byte blocks, not the third block.

The identity chain is explicit:

1. Auth client 22 sends the name-selection transaction's request ID and name.
   The established auth server-8 handler forwards its second GUID to
   `0x14023a770`. That consumer matches transaction +0x30, requires state
   +0x20 equal to 3, and for low transaction type 0x15 copies that GUID to
   transaction +0x68 at `0x14023a8d5`.
2. Auth server 3 supplies the address and result and changes the pending
   transaction to completion state 1. As already mapped, server 8 must populate
   the GUID while the transaction remains in state 3.
3. Completion `0x14023e070` tests the result's low Word. On zero error,
   `0x14023e086` loads transaction +0x68 and `0x14023e09b` stores all
   16 bytes at global `0x14266c050`. It also copies the transaction's name
   to the adjacent name buffer at `0x14266c060`, then emits event 0x5d.
4. Join start `0x14023e740` builds its preamble at `rsp+0x20`.
   Instructions `0x14023e7e5` through `0x14023e809` copy four Dwords
   from globals `0x14266c050` through `0x14266c05c` to
   `rsp+0x50` through `rsp+0x5c`. The relative preamble offset is
   therefore `0x50 - 0x20 = 0x30`.
5. At `0x14023e85d`, the same `rsp+0x20` buffer and exact length
   `0x54` are passed to `0x14023f1e0`.

Consequently, an observer serving an explicitly selected local account can
look up its saved character by the GUID at preamble[48..64]. It need not
derive identity from appearance, the latest saved file or a global
last-selected record. The source establishes the field's identity; GUID
membership in that account remains the local server's lookup contract.

The third 16-byte block, at preamble[64..80], comes from getter
`0x140245340`, which returns `0x14266c748`. Its semantic role is not
established here. The earlier block at [32..48] comes from
`0x14266c040`; this note does not broaden the interpretation of that block.

## Creation continuation and redirect contract

After successful name selection, the already mapped connected-creation view
branch calls `0x141410f30`. This function asserts creation redirect type
zero at mission context +0x1b4 and a nonzero character GUID at +0x20, then
tail-calls `0x1414188d0`.

That sender serializes precisely empty mission client **430 (0x1ae)**,
using the active mission connection. It does not include a request ID or
install another transaction/result record in the inspected body.

Server **1065** is a usable client redirect mechanism, but the frozen client
alone does not establish that the original server specifically replied to 430
with 1065 or prove the original server's ordering. Treat that pairing as the
local implementation's continuation choice, to be checked in playback.

The complete handler `0x141412fa0` confirms these source contracts:

- With mission context +0x150 mask 0x20 clear, it calls
  `0x141417e30`, then `0x141417b80` with redirect type 1, field 2,
  field 3, destination field 0, field 4 and final scalar zero.
- If mask 0x20 is set, it stores that same tuple in context +0x184 onward
  and sets mask 0x10. The previously mapped join-completion path consumes
  that deferred tuple.
- The handler does not read field 1 or field 5.
- The common join helper prepares the map through view slot +0x188 for
  redirect type 1, then calls `0x14023c990` directly. This branch does
  not issue a new auth client-29 route lookup.

The previously unnamed join scalars now have exact wire destinations:

| Source in server 1065 | Common context | Join transaction | New mission preamble |
| --- | --- | --- | --- |
| Field 2, Dword | +0x158 | +0x40 | offset 16, Dword |
| Field 3, Word map-definition key | +0x15c | +0x44 | offset 24, Dword |
| Field 0, Address | +0x160 | +0x48 | connection destination, not copied as an Address into preamble |
| Field 4, Dword | +0x17c | +0x64 | offset 28, Dword |
| Final argument fixed to zero by this handler | +0x180 | +0x68 | offset 80, Dword |

These paths establish transport placement, not the semantic meanings or
retail values of fields 2 and 4. Preamble offset 20 is still not assigned by
the inspected join-start body. The established character GUID at offset 48
is independent of those scalar fields.

## Local creator integration boundary

The existing source-backed response sequence is sufficient to define the next
bounded implementation and playback:

1. Decode mission request 415 and persist its complete supplied character
   data under the explicitly selected local account and a stable character
   GUID. Persistence format and server acceptance policy are local behavior.
2. Send mission result 1048 with that GUID, the supplied name and a valid
   character-summary blob within the consumer's 512-byte bound. This updates
   the client's name-keyed character list and triggers auth name selection.
   Server 1051 feeds the creation error UI and is not the success response.
3. On auth request 22, resolve the supplied name within that account. Echo its
   request ID through server 8 and server 3, with the saved GUID as server 8's
   second GUID, before completion changes the request state.
4. On the connected creation branch's mission 430, a local 1065 redirect can
   lead to a prepared map and a fresh mission join. Select the saved character
   for that new join from preamble[48..64].
5. On a later login, return the saved character through the established auth
   17 character-list path. Its persisted summary must describe the same
   character as the retained creation request and mission population.

Steps 1, 4's server response choice and persistence/relogin behavior require
implementation and runtime verification. This source note does not claim that
they already work or that the local account is a retail authentication service.

## Evidence

Complete successful captures are retained privately in
`~/re/gw2/notes/209026-creation-mission-binding-1.json`.

| Evidence | Observation |
| --- | --- |
| `ev_f5576a1c1ebfd1a4388e700c1a6c97105d50e013020c4e553c911c9d8ca00bbb` | Complete name-selection completion and GUID/global copy; join entry fragment. |
| `ev_79aafdfbff7e50a92f41f7be2ea4cf0bf5d5498178843d6e870658c5d18819bc` | Join preamble construction and pending type-0x15 server-8 GUID write. |
| `ev_61f431788f127ee875a89c86b7f02de46ff00aa5b802335055e90bc4eec22871` | Complete creation continuation, empty 430 sender, 1065 handler and common join helper. |
| `ev_49d5441a6f82ebabe545fdb72c9595d49b4448891353357d37113c81b31b1f90` | Full join constructor, redirect helper, third-block getter and join-start return tail. |

All captures exited zero and verified cleanup of their owned process groups.
Some cleanup reports also list unrelated inaccessible /proc entries; no
claim of observing those external processes is made. No shared Ghidra target
was switched and no gameplay code changed in this investigation.
