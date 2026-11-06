# Build 209026: Mesmer sword range and deferred activation

The three starter sword skills author range **130**, but the demonstrated
consumer implements client **deferred out-of-range activation**. It does not
establish a mandatory server rejection of every SkillBegin outside that range,
nor an authoritative strike-time collision test. This source slice changes no
gameplay. Its timing and damage limits complement the
[animation records](209026-mesmer-sword-animation-data.md) and
[damage facts](209026-mesmer-sword-damage-facts.md).

Static REA used frozen executable SHA-256
`99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`
and an existing decoded content cache. No client, archive extraction or shared
runtime was used.

## Authored range and native predicate

All three definitions are type 64, kind 0, in decoded file 1282860, SHA-256
`05a2bdefaac4937d4d6386913d5e0596af0200391e16704ae426448896f16b23`.

| Skill | Definition offset | Ability body | Body +0xa4 | Override pointer/count +0x150/+0x158 |
| --- | --- | --- | --- | --- |
| 10170 | 3965608 | 3965984 | f32 130.0 | null / 0 |
| 10171 | 4263688 | 4264064 | f32 130.0 | null / 0 |
| 10172 | 4032088 | 4032432 | f32 130.0 | null / 0 |

The f32 bits are `1124204544`. The bounded reader validates each type, key,
kind, classified body reference and consulted bounds; it has no archive
fallback. Evidence:
`ev_020155deeaa20c0c92305a5253efe765d02421eae68858918d1f3e84565b4f47`.

Complete helper `0x1411f8cb0..0x1411f8db9` selects the first matching context
from 12-byte override rows, using the character's virtual +0x228 result and
row float +8; otherwise it uses body +0xa4. For ordinary finite inputs:

```text
reach = max(range + target_radius - margin, 1.0)
result = reach * reach >= distance_squared(attacker_point, target_point)
```

It preserves f32 arithmetic order. Distance helper
`0x1402d18d0..0x1402d19b3` subtracts all three coordinates and sums
`dy² + dx² + dz²`. Equal Point3 tags compare directly; different tags can
invoke the transform resolver when their two high bits are set. This is not
an XY-only test. Predicate/distance evidence:
`ev_fd955054b6f9efefe77d9ee1f60107c16069659f8983ccf5227efec2614cf2b6`,
`ev_205ad0f709d28d646105f5c63343c826384c75e51b5641d7d89d46d5775b2540`.

The whole-agent table `0x14211a7a0` binds radius virtual +0x180 to
`0x14073ae40`, reading agent +0x80: the original server-37
[collision radius in map units](209026-agent-dimensions.md). Position
virtual +0x160 reaches the collider's ordinary getter `0x14109df40`;
+0x168 reaches `0x14109df90`, which first advances/interpolates a stored
position through `0x1410b4830`. Both convert collision coordinates to map
coordinates as `[32*x, 32*y, -32*z]`, consistent with the
[creation-position path](209026-agent-position.md).

The advanced getter uses collider +0x1a0/+0x1c0 and two physics-clock getters.
Its result is not necessarily the configured spawn or the last server-received
position. The clock semantics and all transform cases were not expanded.
Complete position-path evidence:
`ev_fcda1d8b2b1b2fac5eb8d3942bc189567c0939ba5dcf57bae67c7f21ba29aabf`.

## Conditional caller and named deferred callback

The [ordinary input path](209026-skill-input-action-lifecycle.md) calls
`0x1411f6570` at `0x1411f5b92`. That selector has settings indexes 12/13,
character, skill-content, target and eligibility gates. An early failure
returns zero without consulting range. At `0x1411f6754`, its geometric call
uses the target's ordinary position, target radius and margin **0**. It
selects the alternate path when the range test or separate predicate
`0x141292330` fails. No universal setting value is assumed.

A nonzero selector result calls `0x1411faee0` and leaves this ordinary
activation branch. A zero result allows later admission to reach the normal
packed **0x23 SkillBegin** producer `0x141283f90`. Therefore the range test
is not an unconditional prerequisite for receiving SkillBegin.

Complete `0x1411faee0..0x1411fb239` stores the pending slot at skillbar +0xd0,
target at +0xd8 and Point3 at +0xe0. It manages target subscriptions, emits
a listener notification and schedules embedded timer +0x50 with raw delay
`0x28` through `0x141023800`. Constructor `0x1411f4720` installs timer table
`0x142165880`; its +8 callback is `0x1411f9fb0`, whose assertion explicitly
names the state:

```text
m_outOfRangeActivationSkillbarSlot != CHAR_SKILLBAR_SLOTS
```

That callback resolves the current slot SkillDef and obtains the pending
target's advanced position and radius through skillbar virtual +0x90/+0x98
(`0x1411f8070` / `0x1411f8150`). It repeats the range predicate with margin
**32.0**, then checks `0x141292330`. When both succeed, order-component
+0x1d8 calls `0x141283f20`, which resolves the current slot and invokes
+0x1d0: the normal SkillBegin producer. Otherwise it schedules another raw
`0x28` delay. The successful branch also changes a per-slot bit through
`0x1411fb680`; this is not proof that all pending state is cleared.

These complete bodies establish deferred activation, not auto-approach
movement, damage, repeat-hit timing or a server-side strike deadline. The
raw scheduler delay must not be substituted for an authored sword hit time.
Decisive evidence:

| Evidence | Contract |
| --- | --- |
| `ev_e5c189324f4c387639e51b49dfc671280c944f1d90e41ae42f45df51649a594e` | Existing complete ordinary activation caller |
| `ev_76d5330bebfdc9001d1f2887c8b54cadd95a86c24767fc4a88ab7012feb848be` | Alternate pending-state setter |
| `ev_c056d020e41a14c5a68f3e101fe171ff91bad4ce5abd18cd5ff1d1d0cdcfb660` | Timer construction and receiver offsets |
| `ev_fc59fe3d67760ad226e4a614bcd8ce6ba355718a4088af311aa3f480b31e4133` | Complete named deferred-activation callback |
| `ev_3f517805bb5371b287b7d2104e29df3af2c153ee63b4a43d0929505d972d7567` | Current-slot wrapper to ordinary SkillBegin |
| `ev_928ff068685715cc569da47e5460e7299fea7503cb68e417a8aa7309ba7c5aa4` | Pending-target getters and per-slot bit update |

## Run240 and implementation limits

The retained [run240 observation](209026-ether-feast-reconstruction.md#run240-positive-native-health-recovery)
contains an ordinary 10170/slot-5/target-4 request and fixture health
118→88, without UI movement input. Configured player
`[-3200,-25600,-3742]` and wolf `[-3050,-25450,-3745]` differ by
`sqrt(45009) ≈ 212.153` map units. **That is configured-coordinate
separation, not a measured simultaneous native collider separation.**
With fixture radius 1, the helper's initial/deferred thresholds would be
131/99. The observed request does not contradict the source: earlier gates
can bypass deferred activation, and the native advanced positions were not
sampled. The server's immediate fixture hit does not establish retail reach
or hit timing.

Reviewed implementation `7cfe9c4` applies the existing fixture hit immediately
after its ownership/availability/chain checks, without distance testing.
Movement reports are converted to map units and rounded to integers for
heart/POI checks, rather than retained as a precise current attacker position.
NPC positions and radius 1 remain fixture inputs; authored species dimensions
are unresolved.

A future reconstruction must distinguish activation acceptance from damage
execution, define when positions/range are sampled, preserve ownership and
cancellation/death/replay handling, and advance server 628 only after an
applied hit. It also needs current position tracking and reconciliation with
diagnostic actor elevations. Substituting XY distance, altering coordinate
scale or rejecting every out-of-range request is not justified by this slice.
Retail hit timing, latency tolerance and strike collision rules remain
unknown. The current implementation is unchanged.

## Private evidence and pins

All paths below are under `~/re/gw2/notes/`; raw third-party source remains
private. Full records were retained before compact findings.

| Artifact | SHA-256 |
| --- | --- |
| `209026-mesmer-sword-range-source-20261106.json` — 232816 bytes, 11 Evidence records | `34c96ba6019323d5e83b8b022bf6ddae53a9842b915a1e417223f72a4ebb937f` |
| `209026-mesmer-sword-range-caller-followup-20261106.json` — 379322 bytes, 10 Evidence records | `6051cdbcbba7c6a0a81d63cf6a7af8ca19f9a83fb310ca32e831aebbb8815fd1` |
| `mesmer-sword-range-fidelity-findings-20261106.md` — consolidated private findings | `f5d1d1c0090a94fac97b99810f88d4ce574267962dfac008d2fd54f1fcb50d62` |
| `read-sword-native-range-20261106.py` — cache-only reader | `465921345761d84305012c8815603425ccfd8c64d5a53edaf1c052f67ad7a71e` |
| `209026-skill-input-action-lifecycle-20261106.json` — reused caller evidence | `83f2d3de59cea41a95c3655a97344f4a21193d21c3b78719f65d40ba905458ed` |
| `run240-ether-feast-positive-gain-outcome-20261106.json` | `ec669d8543ec0a4059aba0468cde7fff53d40f20ab2310cbddf54855935a3901` |
| `209026-run240-ether-feast-positive-gain-readback-20261106.json` | `2f1ac025a032d77337985868bab2893176cdd9435424d6f2470f98a5faea3e7f` |

Run240 readback evidence is
`ev_6021fa2da55f5e5b2d170fdb6ce8cf8ef53de02a70f36647dc1246281d14ba4a`;
this source lane reused the retained result and did not launch another run.
All new captures retain complete raw/normalized output and verify owned
process cleanup. Some duplicated terminal-render snapshots were truncated.
One recorded malformed address argument exited 1; its corrected query exited
0 and supplies the table proof. Short initial PE metadata fragments were not
treated as complete functions; their logical bodies were captured separately.
