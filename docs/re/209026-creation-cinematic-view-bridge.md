# Build 209026: cinematic playback and creation-view handoff

Static REA, 2026-11-06. The [cinematic playback contract](209026-cinematic-start-contract.md)
connects to the concrete creation-cinema view states and then resumes the
name-based world-join route. This removes an unbound interface edge. It
does not establish which cinematic the Norn intro uses or its authoritative
message order, flags and completion behavior.

## Concrete installed view interface

Factory `0x1409507f0` installs table **0x141b7e818** at the allocated
object's +8 and returns that interface on its nonnull path. Initializer
`0x14094a810` stores the return in **0x14267f878**, which getter
`0x14094a800` returns. Thus the cinematic scene's calls through that getter
resolve to these exact installed methods:

| View slot | Target | Observed responsibility |
| --- | --- | --- |
| +0x110 | 0x1404e6180 | Return view-interface+0x6c state |
| +0x158 | 0x14094dc70 | Save prior view and select a cinematic loading view |
| +0x160 | 0x14094dd60 | Loading-to-playback view transitions |
| +0x168 | 0x14094ddf0 | End-of-cinematic view continuation |
| +0xc0 | 0x14094f810 | State assignment and notifications |
| +0x48 | 0x14094b030 | Existing creation-name auth join route |

Offsets in this note are relative to that +8 interface, not the complete
allocation. The [creation note](209026-character-creation.md) establishes
enum0 `char_create`, enum1 `char_create_cinema` and enum2
`char_create_cinema_load`.

## Start, play and end

The complete scene path `0x1412b1cf0` calls view+0x158 with scene+0x18's
definition when the cinematic context has no hanging scene at +0x20.
Under its subsequent flag-0x10 branch, it calls view+0x160. These are
conditional calls; a wire request alone does not prove either occurred.

Complete **`0x14094dc70..0x14094dd21`** first excludes prior loading states
2, 7 and 11. Otherwise it saves the old view at interface+0x68. For old
view0, it sets view2. It then has additional branches that matter:

- A nonnull definition whose +0x38 flags contain 0x40 selects view16.
- Otherwise a nonzero resource-context predicate selects view7.
- Otherwise old view16 selects view11; old view0 retains the view2 just set.

No default outcome for those content/resource predicates is assumed.
Complete **`0x14094dd60..0x14094ddee`** maps 2→1, 7→6 and 11→10, with a
separate assertion for view13. It does not choose a cinematic definition.

The complete scene end routine `0x1412b18f0` calls view+0x168 before its
conditional client263 response and listener cleanup. Complete
**`0x14094ddf0..0x14094deb9`** returns immediately when view flag0x200 is
set. Otherwise, in view1 it calls character context+0x70 and then view+0x48.
That is the already demonstrated `0x14094b030` creation-name continuation,
which ultimately starts auth request22. This note assigns no new meaning
to character context+0x70 or the flag0x200 policy.

The separately retained **server1048** result path updates the roster and
calls view+0x30. Its leaf `0x14094b090` returns in states1/2 and otherwise
calls that same +0x48 continuation. These two paths explain how an active
creation cinematic can defer the ordinary post-creation join, then resume it.
They do not prove the server's Norn start key or the timing required to
place the view in states1/2 before the result arrives. The current fixture's
direct world entry is not a completed retail intro.

## Narrowed source leads

The [terminal submit path](209026-creation-submit-cinematic-boundary.md)
reaches client415 without directly selecting a cinematic. Its known
submit-event listener manages submission/error controls. The remaining
selection gap cannot be closed by renaming its raw flag8 as “skip cinematic.”

Two earlier named seeds have concrete unrelated owners:

- `m_cinematicDef` at `0x141b2cfa8` has one validated LEA reference in
  `0x1406661a0..0x140666306`, explicitly labelled
  `HeroDialog/Achievements/AchSuggestedLocation.cpp`. This is an achievement
  suggested-location update, not demonstrated Norn intro selection.
- [`Cannot play the cinematic`](209026-cinematic-guild-mission-seed.md)
  belongs to `WuModuleGuildMission.showMapCinematic`.

The creation-cinema named assertions resolve to `VdfLoad.cpp`'s generic
state guard, not a filename selector. The state setter's `0x140954e10`
callee is only a leaf store of its supplied view. These bounded leads
are classified; repeating their name/offset searches would not resolve
the missing Norn content input.

## Evidence and implementation boundary

Private main bundle:
`~/re/gw2/notes/209026-creation-cinematic-view-bridge-20261106.json`, SHA-256
`2f6e3f6cbc5e12e2facde2acd297783c16fe3734de34cfc3bce92d855da318d1`.
Principal REA records:

- `ev_86ac81eadd2d75ae9dbf8c3ab196cdec23eeac0e54e1f765a15d281d6f2dfdfb`,
  `ev_241f59f64ced9a836abeaa3f92a1fdf1f9261e3cc2d00e20e328177f3cd99ecf`,
  `ev_cf5abae71c017632f84db42db51f1aee38131b32cdedfaa28206901fe965ea42`:
  exact table installation, nonnull factory return, global binding/getter.
- `ev_c1c9db957dd548e8b584aeb533f160b2f4f7f2e34abb536df7f3b9d9858410cc`,
  `ev_79f19ef8b9271f5bd472bd69542fa830268769d2ddb29fb4e75daf8c7016f080`:
  exact cinematic slots and complete start/play/end view methods.

The prior [start/end evidence](209026-cinematic-start-contract.md) supplies
the concrete scene callers. The result-gate/submit evidence is linked from
the submit note. The achievement seed's separate two-capture bundle is
`~/re/gw2/notes/209026-cinematic-achievement-seed-20261106.json`, SHA-256
`52ee52af974c344a12468ba0ef64b4be0269708aa73987a5b23e5c44d702154a`.

All fifteen main captures exited0 with complete raw terminal output and
verified owned-helper cleanup. Two records omit one rendered frame each;
their raw output is complete. Missing `.pdata` entries and initial interior
fragments were followed with bounded explicit windows. Partial neighboring
functions are not used as complete-body evidence. All helpers assert frozen
PE SHA-256 `99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`.
No client launch, archive read, runtime lease, packet emission, gameplay
change or publisher action occurred. A source-backed Norn selection/sequence
is still required before wiring this contract into the starter flow.
