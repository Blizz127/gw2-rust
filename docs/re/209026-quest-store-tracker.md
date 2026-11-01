# Build 209026: quest store request tracker error cleanup

Static REA follow-up, 2026-11-01, to
[store process dispatch](209026-quest-store-dispatch.md). The startup
consumer stores a request ID with a null value. A separate store-error
listener can remove that entry, even though the quest-facing service's
own error slot is a no-op. This establishes conditional error cleanup,
not successful empty-response completion or a zone-readiness condition.
The client remained stopped throughout.

## Request entry semantics

The static object at `0x14255a8f0` has primary table `0x14193fd30`.
Its virtual +0x18, **`0x1402effc0`**, rejects ID zero. For a nonzero ID,
it queries the map at whole +0x48 when its count (+0x4c) is nonzero.
The early-return guard checks the found slot's **value pointer at +8**,
not its occupied marker at +0x10. A nonnull value returns false; otherwise
the method calls `0x1402ee5a0` with R8=0 and returns true.

That helper finds/inserts a 24-byte map slot:

| Slot offset | Value |
| --- | --- |
| +0, Dword | Request ID |
| +8, Qword | Supplied value pointer, null for the startup call |
| +0x10, Dword | Occupied/hash marker |

It increments the map count only when adding an unoccupied slot. Thus
repeating the startup method for an existing null-valued ID is not rejected
by the nonnull-value guard. This refines the earlier shorthand that an
already tracked ID necessarily skips insertion. Primary virtual +0x10,
`0x1402ee7d0`, is a related entry that asserts a nonnull supplied value
and nonzero ID before storing through the same helper.

## Store-error subscription and matching cleanup

The executable's static data installs `0x14193fd60` at object +8.
Primary virtual +0, **`0x1402efb70`**, registers that +8 interface with
context +0x1f8's listener insertion method. The interface's +0x160 entry
at **`0x14193fec0`** is **`0x1402efef0`**, matching the store dispatcher's
ordinary error notification `(request ID, error pointer)`. Registration
and delivery at runtime remain unobserved.

The callback receives the +8 interface, so its map offset +0x40/count
+0x44/slots +0x48 refer to whole-object +0x48/+0x4c/+0x50. It checks the
occupied marker for the received ID. If present:

1. A nonnull value receives virtual +8 with the error pointer; its return
   value becomes an optional notification object.
2. The map entry is removed through **`0x1402ee410`**, regardless of whether
   its value was null. Removal repairs the probe chain, clears the final
   marker/value, decrements the count, and releases a nonnull stored value.
3. The callback only notifies its own listeners if the optional object is
   nonnull and the error's first signed Word is positive.

For the null value inserted by the quest startup consumer, step 1 is
skipped and the optional object stays null, so step 3 is also skipped.
The absent-entry branch instead invokes `0x1402ef480` and `0x1402ee890`;
the matching null-entry branch bypasses both. Their broader presentation
behavior is not needed to establish this distinction.

This is conditional on outer event 0x23 reaching the store receiver,
the receiver taking its ordinary listener-notification branch, and the
tracker interface being subscribed. It does not prove that an error is
required, that all errors use this branch, or that no other listener has
side effects. Missing `Items`/`Packages` in a successful response still
has no identified explicit completion emission in the inspected callback.
Any cleanup on that separate success path and any readiness dependency
remain open. No emulator response policy changed. The
[reader sweep](209026-quest-store-tracker-readers.md) finds no pending-state
reader in this object.

## Selected request release path

The second search request has primary table `0x142111808`, whose +0
entry is `0x140ff6190`. Its request-interface +0 entry `0x140ff59f4`
adjusts RCX by -0x10 and jumps to the same destructor. The destructor
releases the auxiliary buffer at whole +0x58, decrements the Dword counter
at `0x14289a504`, invokes a nonnull whole +8 callback's virtual +8, and
conditionally releases the request allocation. Counter helper
`0x1409b2040` is an atomic decrement with no callback.

For callback table `0x1419248c0`, +8 is `0x140246e90`, which calls virtual
+0 with EDX=1. That resolves to `0x1402443d0`, which only conditionally
releases the 16-byte callback allocation. Neither selected destructor
emits an event or accesses the tracker. The request's complete success
entry `0x140ff72f0` calls callback +0x20 and returns on nonnull XML;
it contains no extra completion call after the callback. Thus the checked
success callback and release chain do not supply the missing cleanup.
Other lifecycle dispatch outside this chain remains unclassified; this
is not proof of a permanent entry or an observable client hang.

Private follow-up bundle:
`~/re/gw2/notes/209026-quest-store-release-1.json` (1,920 records).

| REA evidence | Observation |
| --- | --- |
| `ev_53fff20c420b814507ad5c2e4089d678731374a0ee18903afcefd599d56f418b` / `ev_9fed1389a77bec4be577ac139be6d39055b7c112d21064c1181193db72de2b32` | Callback and request destructor table entries. |
| `ev_e1ca42dab96331445d67675f611c5504bbc625672a883f6b225a63cc25bfaf54` | Callback deleting destructor and search-request table installation. |
| `ev_e42452edba1f13c7d0679e8c1a49c5a9815413ecf57d869ca227150e803f316f` | Complete request destructor and interface adjustment thunk. |
| `ev_c495241dc4ff92bf0cbb615818506a39c75cb62a653de1d3195a0d1bf08df0c2` | Atomic counter helper, callback release wrapper and complete search success entry. |

## Evidence

Private bundle: `~/re/gw2/notes/209026-quest-store-tracker-1.json`
(1,913 records), frozen executable SHA-256
`99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`.
Static REA only. Direct static table reads establish the interfaces;
exploratory unaligned disassembly edges are excluded from the claims.

| REA evidence | Observation |
| --- | --- |
| `ev_c204f3223064d56bd608838dc0b132301bf8740f53c2f928d083d34335208800` | Complete startup consumer and insertion helper. |
| `ev_50bc702d344688cc1ddf039924eb2a0334c80c4470db81e9a6311d61574139c8` | Primary methods and +8-interface error callback entry. |
| `ev_60acb8671bc8e725d44a8610dd630c0f6b9a0bd91482573f16bb42e25c2a8afe` | Complete error callback, registration and nonnull-value insertion method. |
| `ev_62f86e3ae47caec7ec40b9ed10f1ac2ae6f7f860c3559380f9ae8f2727472b30` | Direct static primary/+8/+0x10 interface tables. |
| `ev_c00e36d68f19c46de700ed6ee56bb3821b078f55f98c595e83f6e2b97db0ba6a` / `ev_3208ad1e241a290bc97cc98f0fbf4fd5ac9b14b56f607c0e87a3f4a2b6aaa566` | Complete map removal, count decrement and optional value release. |
