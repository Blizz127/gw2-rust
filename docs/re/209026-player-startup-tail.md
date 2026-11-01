# Build 209026: player startup tail, bounded dependency follow-up

Static REA trace, 2026-11-01, on the frozen executable identified in
[mission transfer](209026-mission-transfer.md). This follows server 431's
[wardrobe/startup path](209026-wardrobe-startup.md). No client execution
or User Agreement interaction.

## Embedded player state

Context helper `0x1411b9680`, called with a null second argument after
431 sets context +0x208 bit 0, obtains the controlled player through
context virtual +0x70 (`0x140498900`, returns +0xa0). It returns if that
player is null or context virtual +0xe0 reports the bit clear.

The tail's inspected player getters resolve as follows. Player vtable is
`0x14215e9d8`; constructors are called by the previously traced player
constructor. These objects do not each require a separate allocation packet:

| Player slot | Getter | Result | Relevant inspected consumer |
| --- | --- | --- | --- |
| +0x08 | `0x1411bacd0` | Embedded +0x330, account object | Account +0x100, +0xf8, +0x1c0, +0x158. |
| +0x18 | `0x1411bb240` | Embedded +0x9760, progress object | Optional progress lookup by GUID/key. |
| +0x48 | `0x1411bb430` | Embedded +0x9ea8 | Virtual +0x20 reads its +0xa0 Dword. |
| +0x1b0 | `0x1411bace0` | Embedded +0x218, achievements | Four initial Dwords are summed by virtual +0x20. |
| +0x3e8 | `0x1411bb460` | Embedded +0xa090 | Virtual +0x60 reads its +0x40 Dword. |

Account constructor `0x1412367a0` installs primary vtable `0x14216d3f0`;
the source diagnostic at `0x14216d5f0` names `ChCliAccount.cpp`.
Its selected virtual methods are simple reads:

- +0x100: `0x14045de30`, returns Qword +0x70; the caller keeps its low
  Dword. Constructor value is zero.
- +0xf8: `0x14045de40`, returns Dword +0x68; initially zero.
- +0x1c0: `0x141237700`, returns whether Qword +0xc0 is nonzero;
  initially false.
- +0x158: `0x1412374e0`, reads +0xb8 but tail-calls `0x140d859a0`
  with EDX=0. That target returns EDX, so the inspected frozen method
  returns zero without a further pointer dependency.

The +0x9ea8 constructor `0x14120f4d0` installs `0x142169a78` and clears
+0xa0. Its +0x20 method `0x140282190` returns that Dword. The +0xa090
constructor `0x141209da0` installs `0x1421693a0` and clears +0x40;
virtual +0x60 `0x1402b6260` returns it. The tail clamps that latter value
to an unsigned minimum of 1 before calling `0x1409a4060`; its gameplay
meaning is not assigned here.

There is also a genuinely optional pointer at player +0x97b8. The tail
calls player virtual +0x338 (`0x1411bb2f0`) to test it before invoking
virtual +0x330 (`0x1411bb250`) and then object virtual +0x60. It skips
that branch when absent. This is distinct from 431's unguarded wardrobe
and reward-track manager dependencies.

## Achievement/content processing remains material

The tail calls `0x1411b8ba0` with 1 when its own second argument is null,
as in 431. This helper reads embedded player +0x218 through virtual
+0x20 (`0x141216240`), summing Dwords +8/+0xc/+0x10/+0x14 into context
+0x260. Constructor `0x1412158c0` clears all four. The source diagnostic
`0x14216a470` names **ChCliAchievement.cpp**.

This helper does more than read the zero fields: it enumerates content,
looks up GUID-selected achievement categories, and calls achievement
virtual +0x58 (`0x141216470`). That method explicitly handles a null
category by writing zero to supplied output fields. For a nonnull
permanent category, it dereferences category +0x60 without a null guard,
iterates its achievement pointers and calls further achievement methods.
The outer helper also asserts permanent category type and nonnull
permanent-category data in its selected content list.

Achievement virtual +0x68 (`0x1412167d0`) and +0x70 (`0x141216920`)
derive a nonnegative remaining interval from +0x70/+0x78, +0x68 and
`0x1409cb4a0`; no additional component pointer is used in those two
methods. The full content lists, achievement evaluation, context
notifications and final global callback are not exhausted by this note.
The evidence narrows allocation requirements; it does not establish an
empty player object as a complete usable startup state.

## Evidence

Private cumulative bundle:
`~/re/gw2/notes/209026-player-startup-tail-1.json`.
Bounded call-only listings were used to choose deeper reads; their
omitted instructions are not treated as a complete function trace.

| REA evidence | Observation |
| --- | --- |
| `ev_88f246512c93f9014c3a94eae3fe8544e880ab1e2b6a3f55356a24ede3d061f0` | Tail calls/guards, controlled-player getter and achievement helper entry. |
| `ev_64744b2135adbe97e612ea40adc663fa032936ac5f55de9333fdf820b04630f6` | Embedded getters, optional pointer predicate and bounded achievement call listing. |
| `ev_347dcdbe6a826d9f538a14a5fe71542b05f2127b142c575127387699486d8470` | Achievement source/sum, category queries and content assertions. |
| `ev_65950bc990c711e28556cf9a0b1bddc3408b8d7f2dcae410be9deace305a80af` | Constructors and selected method-table entries. |
| `ev_423262bdfefa6a057d1023c6b02f61547c633c8914d1ffbf1071984414d823fc` | Category null branch, timer methods and scalar getter. |
| `ev_dcf5b25ceb2c1ff85f455fd4b43a16d3337c06527a45185fcb46dd1067ea97f0` | Account vtable/initial values and category-output epilogue. |
| `ev_76816d2791756f3edbeb7e5840200ac30953db7f6a2c9e604c48bd7df0fa2edc` | Account method slots and content iteration call sites. |
| `ev_c4fccc474f54c62654173d3b3866b0bc9788bb58174a8f8738c0c0b48297b31c` | Account methods, source diagnostic and embedded scalar initialization. |
| `ev_ed326da96216d3b69dbd11cc2d15d37bbad82ce92581d4afbe1caccf5db4aaed` | Constant-return target, +0x9ea8 initialization and permanent-category dereference. |
