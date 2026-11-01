# Build 209026: wardrobe manager and server 431 prerequisites

Static REA analysis, 2026-11-01, on the frozen executable identified in
[mission transfer](209026-mission-transfer.md). No runtime message replay
or User Agreement interaction. This continues the
[reward-track consumer trace](209026-reward-track-messages.md).

## Lazy wardrobe allocation

Character +0xb00 is **m_wardrobeMgr**, named by assertion `0x141acbc60`.
Character constructor `0x1411d2810` clears the pointer at `0x1411d2d2b`.
Helper `0x1411d83b0` returns an existing manager or allocates 0x6f0 bytes,
constructs it through `0x1412190e0`, stores it, asserts nonnull and returns.
The constructor installs primary vtable `0x14216a6b8`, secondary vtable
`0x14216a9f0` at +8, and stores the character owner at +0x10.

The complete constructor (`0x1412190e0..0x1412199fa`) initializes its
arrays, lists and flags, including clearing the listener list +0x180
and four Words at +0x1bc..+0x1c2. Its final content-service GUID lookup
can leave +0x140 null; there is no direct owner coreStats or inventory
dereference in this constructor. That does not establish later wardrobe
operations as valid with missing character components.

Two small server messages establish concrete allocation paths. Both use
character-context virtual +0x18 (`0x1411b4500`, raw index lookup), skip
the operation if the character is missing, and otherwise call the lazy
allocator before their setter:

| ID | Descriptor / handler | Fields after ID | Setter |
| --- | --- | --- | --- |
| 247 | `0x1425e19a0` / `0x14124f6a0` | Character-index Dword, Word | `0x14121f0a0` |
| 248 | `0x1425e1b80` / `0x14124f620` | Character-index Dword, four separate Words | `0x14121f020` |

These are protocol 0/group 0 pairs `0x14216f410` and `0x14216f420`.
The Words use the existing fixed little-endian u16 encoding, not Dword
varints. Gameplay names for these fields are not established here.

247 passes the Word as a type-**23** numeric ID to content virtual +0x70,
stores the resulting pointer at manager +0x128 and dispatches manager
+0x180 listeners through virtual +0x20 (`0x14026e848`). There is no null
assertion on the content result in this setter.

248 compares all four Words with +0x1bc, +0x1be, +0x1c0 and +0x1c2.
If all match, it returns. Otherwise it stores all four and dispatches
manager +0x180 listeners through virtual +0x18 (`0x14026e840`). Thus a
first all-zero 248 can allocate the manager and leave these freshly
cleared fields unchanged without dispatching that event. This is a
static initialization candidate, not a verified wardrobe/gameplay state.
For an existing manager, changed values can reach untraced listeners.

Server 250 also reaches this allocator, but its descriptor has a bounded
array form; it is outside this small-field codec/initialization scope.

## Server 431 follow-up

The previously mapped payload-free 431 calls `0x1411b7700`. Its first
context +0x230 event uses listener virtual +0x80 (`0x14025553c`). The
event's runtime subscribers remain unexhausted. After that event:

- Skill-bar helper `0x1411f9de0` asserts **m_ownerCharacter->IsPlayer()**
  and **m_ownerCharacter->GetPlayer() == CharClientContext()->GetControlledPlayer()**
  (literals `0x142165c28` and `0x142165c50`), then invokes normal rebuild
  through virtual +0x1f8. Its established profession/coreStats/inventory
  requirements therefore also apply here.
- Character +0xb00 is dereferenced directly, so a wardrobe manager must
  already exist by this point. Virtual +0x310 resolves to `0x141220320`,
  which sets manager +0x1b0 bit 0, registers its +8 interface with the
  input service, and tail-calls `0x14121a710`. That routine immediately
  dereferences owner coreStats and reads virtual +0x28/+0x38/+0x48.
  Its content enumeration and broader effects remain unexhausted.
- The controlled player's reward-track manager refreshes all four slot
  records, as documented in the reward-track trace.
- Helper `0x1411e9440` operates on embedded player +0x18c8. It checks
  **EnableFashionLoadoutsCli** through `0x1410f3c30` (default 1). When
  enabled, it requires the object's +0x40 value to be in 1..9, clamps
  a settings value through service selector 0x14, and writes it back.
  Constructor `0x1411e89c0` initializes +0x40 to **2**, so the fresh
  value passes this range check without another message. The diagnostic
  identifies this object's subsystem as `FashionLoadouts`.
- The context sets +0x208 bit 0. Getter `0x1411b5eb0`, context virtual
  +0xe0, reads that bit. Tail helper `0x1411b9680` first requires a
  controlled player and this bit, then reads progress and other embedded
  player interfaces, updates context fields and emits further callbacks.
  The complete readiness meaning of the bit and those dependencies
  remain open; it is not equated with the mission loader's READY state.

The evidence establishes a wardrobe prerequisite for the direct 431
body; it does not prove that the initial context event cannot create it,
or that 248 is the only way to supply it. No bootstrap sequence is replayed.

## Evidence

`crates/gw2-proto/src/wardrobe.rs` implements transactional 247/248
frames with a raw character index and the documented Word fields.
`cargo test -p gw2-proto --lib wardrobe` passed **3/3**; the library
build passed. Independent byte fixtures cover Word endianness and fixed
width, distinct four-Word ordering, zero/u16::MAX/u32::MAX, concatenation,
append boundaries, every truncated prefix, wrong IDs, and overflowing
or overlong character-index varints. No client replay was performed.

Private cumulative bundle:
`~/re/gw2/notes/209026-wardrobe-startup-1.json` (1,165 records).
Raw displacement/rel32 hits were only candidates; named conclusions use
the subsequently inspected instruction boundaries and descriptors.

| REA evidence | Observation |
| --- | --- |
| `ev_77fd85207be98b79aed02a4336cfb3dcd1e79bdec6eae5af847658bef9d48137` | Skill-bar assertions, fashion helper, context tail and candidate pointer references. |
| `ev_f58e7d6e8539331deb11268620d9aa20f756ee61599201665c7d0b4f31365461` | Character pointer clearing, initial context event and context flag getter slot. |
| `ev_f6457a2aae2876bec63dd76760dd6bd7fe66c70408da0b04f38f3e2f681d067a` | Lazy allocation entry and context flag getter. |
| `ev_af35e464364d5b3d7be117adcc585e2e6944b28d1f64695b2f3406cddb66f209` | Complete lazy allocator and caller candidates. |
| `ev_c07f0d2e946248e2149c7106a2459946823222d358e7e0b956375fac60af52cc` | Wardrobe assertion, vtable and message handlers. |
| `ev_f558b97973808ff4e27c97feaaa909f8eeaaa9ad2938a82ad5daab19a2c74d23` | Message registration and constructor first portion. |
| `ev_6b4c20a24fc74413d8090b261ed853105dadc7534eec7c2c23bcd45a7994b6ca` | Descriptors, startup virtual +0x310 and constructor tail. |
| `ev_288ea85af95b71386beafb61406d366024a501d9446a1765094731abb8d26bb5` | Complete 247/248 setters, constructor middle and wardrobe coreStats dereference. |
| `ev_bc89804f8a5499c05eb1ca7340b809f3e4b59058b3a8d59b04a694bcc0d1da92` | Fashion constructor default, feature flag and bounded context-tail call listing. |
| `ev_58358692eda777ab36c0d04a99b63ef95d30933e9c1c35cf3fcfd42cf0fd42d5` | Raw character-lookup slot and wardrobe event trampolines. |
