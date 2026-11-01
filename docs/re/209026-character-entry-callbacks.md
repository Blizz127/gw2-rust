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
and their rendering dependencies require further tracing; the subsequent
[view-listener trace](209026-character-view-reveal.md) resolves one active
subscriber and its ordinary allocation path. Agent REVEALED
is already changed before these notifications execute.

## Listener registration and two resolved subscribers

Character primary virtual **+0x558** resolves to `0x1411db7f0` (table entry
`0x142161138`). Its complete body asserts a nonnull, previously absent
listener and appends it to the collection at character +0x1e0. It does not
immediately invoke reveal or any other listener callback. Primary +0x560
(`0x1411dbce0`) removes the matching listener and asserts membership.
Character construction initializes the collection's head/tail to null.

Two actual subscribers have resolved reveal/conceal slots:

| Subscriber | Registration | Listener vtable | Reveal/conceal |
| --- | --- | --- | --- |
| Embedded chatter, character +0x330 | `0x141276660` passes chatter +0x08 to character +0x558 at `0x1412766c3`, on its player branch | `0x142177dd0`, installed by `0x141276170` | +0x448 and +0x450 both `0x140163990`, no-op |
| Player embedded object at player +0x618 | `0x1411e8110` passes object +0x50 at `0x1411e82ad`, guarded by registration flag 0x80 and a context-player match | `0x142162670`, installed by `0x1411e4f90` | +0x448 and +0x450 both `0x140163990`, no-op |

For the second path, the registration function recovers player by
subtracting 0x618, compares it with context virtual +0x70, and takes the
character from player +0x20. These address relationships identify the
registered interface without assuming it is a whole-object vtable.

Neither subscriber adds work on these two notifications. This does not
remove chatter's existing profession/coreStats/inventory prerequisites,
or imply that all character listeners are inert. A bounded FF /2 search
for virtual +0x558 in `0x1411b0000..0x141400000` found eight candidate sites,
including one that actually calls the agent interface and must be excluded.
Several remaining subscribers are unresolved; loaded function-pointer calls,
tail calls and sites outside that range are not covered by this census.
No new bootstrap default or ordering guarantee follows from that search.

Additional private bundle:
`~/re/gw2/notes/209026-character-entry-listeners-1.json`.

| REA evidence | Observation |
| --- | --- |
| `ev_76b8b523c95e2d9c414eaec1b76d5e55ea5ec5955071f9a87cf35e8cd76d5325` | Listener insertion body through its first return, complete removal body. |
| `ev_b70588f003aa09e06549df4cac5bed03f7c4dd0d6392e54008c852bb33a31d14` | Remaining insertion branches; chatter and player-object subscription calls. |
| `ev_7ebf50dc413ede39df9fb98e41624f0298abdd7ff7fb1165e724b2f4f22f66e6` | Primary vtable resolves registration/removal to +0x558/+0x560. |
| `ev_d347c2161a82c4296ee430e92a278c4056d67c3995a826a57d816e438febe07f` | Constructor initializes the listener collection head/tail to null. |
| `ev_307958f3cba33a44f8b4374c59b8c59717bd312a4493c1e408c9833bf67cf640` | Complete chatter constructor and player-object registration entry/gate. |
| `ev_d62469383ecf1d77d7cf17d69370281d41381613d33879206aef4442c2125c12` | Player-object constructor installs its +0x50 listener vtable. |
| `ev_e0badb093760d885aaf758aa587ca95795bf713b7976a93ea02ddd244f2f5b59` | Chatter reveal/conceal slots are no-ops. |
| `ev_0f2bd8def7df886d4296813073471fa7e90fe6c46675ed767410bb5120633604` | Player-object reveal/conceal slots are no-ops. |
| `ev_d1de48a73450d0b7c1a310786c4fe2e92b5ffc922c50da7ae594b0d0926f2f64` | Bounded virtual-call candidate census. |
| `ev_f01eb7a35bf31fd44878f8597ccb45514116c1d9e02e254c8d693f0ebef5ec91` | Call-site contexts, including the unrelated agent-interface call. |

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
