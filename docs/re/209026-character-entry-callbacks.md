# Build 209026: character interface callbacks and vtable correction

Static REA follow-up to [agent reveal](209026-agent-reveal.md). No client
launch or runtime packet transmission was performed.

## Correct interface base

Type-0 factory `0x1411b66e0` constructs a character through `0x1411d2810`
and returns character +0x48 at `0x1411b681d`. The constructor installs
**vtable `0x142161670`** at that offset (`0x1411d290b`–`0x1411d2912`).
The base-agent constructor stores this returned pointer at whole +0x10.

Earlier world/scene-entry notes incorrectly used `0x142161710` as the
game-interface vtable base, producing an incorrect no-op conclusion for
the final +0x10 creation callback. That address is inside the actual table.
The corrected entries are:

| Interface slot | Entry address | Implementation |
| --- | --- | --- |
| +0x10, creation completion | `0x142161680` | `0x1411dc6e0` |
| +0x18, conceal | `0x142161688` | `0x1411dd470` |
| +0x20, world entry | `0x142161690` | `0x140163990`, no-op |
| +0x28, world exit | `0x142161698` | `0x140163990`, no-op |
| +0x30, reveal | `0x1421616a0` | `0x1411dd4a0` |

## Creation callback

The complete `0x1411dc6e0` first checks interface +0x58 (character +0xa0)
masked by 0xf0000000 against **0x30000000**. Equality returns immediately.
The character constructor obtains +0xa0 from its supplied agent interface's
virtual +0x118 at `0x1411d2924`; this is the identity field traced in the
creation notes. Thus the existing candidate player identity 0x30000001
takes the early return. This is a branch-specific result, not a no-op
function for all characters.

Otherwise it calls character primary virtual +0x288. If the returned
pointer is nonnull and its float +0x64 is ordered greater than zero, it
obtains an interface through character +0x08 virtual +0x00 and invokes
that returned object's virtual +0x428 with the float. The nested targets
and the float's meaning remain untraced here. The complete callback has
no direct scene-entry call, but this does not exhaust that other branch.
The full/simple collision constructor-tail +0x20 callbacks remain verified
no-ops independently of this correction.

## Reveal and conceal notify character listeners

`0x1411dd4a0` (reveal) and `0x1411dd470` (conceal) each recover the whole
character by subtracting 0x48 from the interface pointer. They pass that
pointer's address as R8 to `0x14026dce0`, with the listener collection at
interface +0x198 (**whole character +0x1e0**) as RCX. The callback thunks are:

| Notification | Thunk | Listener virtual slot |
| --- | --- | --- |
| Reveal | `0x1411d40f0` | +0x450 |
| Conceal | `0x1411d40e4` | +0x448 |

The inspected dispatch loop loads a nonnull listener from a list node,
passes it as RCX and the whole character as RDX, then calls the supplied
thunk. These are listener notifications, not direct character-primary
virtual +0x450/+0x448 calls. Actual listener registrations, implementations
and their rendering dependencies require further tracing. Agent REVEALED
is already changed before these notifications execute.

## Evidence

Private bundle: `~/re/gw2/notes/209026-character-entry-callbacks-1.json`,
on the same frozen executable identity recorded in the reveal note.

| REA evidence | Observation |
| --- | --- |
| `ev_b03f0c41ef7f2b137d24892f2695801af612d522121a46954fdba06d3048251d` | Constructor installs the actual +0x48 vtable; factory returns that interface. |
| `ev_9865e9d71f844c8e1adde09ea2e674e9c44b1242d86685e649ab40ef3b8548e6` | Raw actual-table entries for creation, world entry/exit and reveal/conceal. |
| `ev_4104a3ee7ce7fa6d5ffa5e42d8f90f3435c7ae4d7400cfc38fa94eef2aa76feb` | Complete creation callback and complete reveal/conceal listener wrappers. |
| `ev_b58336e19d7ed46b2d20b85fa652c9afb33057cce2f9b9b93de5bff83882bade` | Complete callback thunks and bounded listener-dispatch loop. |
| `ev_157b75de2d2e7de9bffa638bd94694343fb3e909152851cc1608b10574047a4a` | Base agent stores the factory-returned interface and invokes its +0x10 slot. |
