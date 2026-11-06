# Build 209026: authored quest-step state transport

Static source follow-up, 2026-11-05, to [quest154 content](209026-quest154-content.md)
and [quest initialization](209026-quest-lifecycle.md). The client has a
source-supported **per-step completion and snapshot path**. Raw codecs are
implemented below; server state and gameplay integration remain future work.
This is a bounded progression foundation; it does not identify the retail
action that completes any Great Hunt step.
No gameplay policy or runtime fixture changed in this work.

## Content and server integration

Quest **154**, goal **1**, directly references map **379**. Its six type-343
definitions begin at **1282830:4146312**, stride **0x60**. Definition +0x14
contains IDs **1, 2, 3, 6, 4, 5** in array order. These are IDs, not a
proven encounter sequence. Definition flags +0x10 are **0, 2, 2, 2, 2, 2**;
the already traced goal constructor initializes runtime flags **0, 0xa,
0xa, 0xa, 0xa, 0xa**, with runtime +0x14 zero. Content evidence:
`ev_38781654b454545ebff182fb280ca8f64e3d035f3abc51bc5a1f78d2367232c0`.

The server integration inspected at repository HEAD
`a2fda6b2ff8f671a1010d0101e0182665d948df0` remains unchanged:
`crates/gw2-net/src/bin/game_observe/mission.rs` stores only the configured
quest ID, initial byte and goal ID, then emits 527/532/528 once after player
readiness. It sends no 542/544 frames and stores no per-step progress.
CharacterStore persists the initial creation request; its
durable format does not contain quest progress. Existing heart/kill fixtures
do not supply Great Hunt triggers.

## Exact message consumers

All six handlers resolve the leading **player index** through character
context virtual +0x118, assert that player exists, and pass its quest manager
at player +0x9820 to the helper below. Each helper searches runtime steps by
**definition +0x14 == supplied step ID**, not by array position. The extracted
server schema supplies these fields after the common player Dword:

| Server ID | Remaining wire fields | Handler → helper | Established effect |
| ---: | --- | --- | --- |
| **542** | Dword step ID | `0x141258e00` → `0x1412017e0` | Set runtime step flags bit0; evaluate goal completion and notify listeners. |
| 543 | Dword step ID, Byte | `0x141258e70` → `0x1412019e0` | Normalize nonzero byte to true, set/clear runtime flags bit2, notify. Its UI/gameplay name remains unassigned. |
| **544** | Dword step ID, Byte raw flags, Dword objective ID | `0x141258ee0` → `0x141201ad0` | Replace runtime flags at +0x10 and objective ID at +0x14; no notification or goal-completion evaluation in this helper. |
| 545 | Dword step ID, Dword objective ID | `0x141258f50` → `0x141201bd0` | Require existing runtime objective ID zero, then assign it. Assertion names `m_objectiveId`. |
| 546 | Dword step ID | `0x141258fc0` → `0x141201c90` | Clear runtime bit0; if bit3 was set, also set bit1; notify and update the manager's step collection. |
| 547 | Dword step ID | `0x141259030` → `0x141201de0` | Clear runtime bit1, notify, and conditionally update other consumers. Do not assign an unsupported UI name to this operation. |

Dwords use the existing unsigned varint encoding; Byte is one wire byte.
The decoded-memory offsets used by the handlers are not varint wire offsets.
The packet contains no quest ID: its step ID applies to the current active
quest/goal, making server-side state validation necessary.

## Completion, restore and prerequisite limits

For 542, helper `0x1412017e0` requires an active quest, `!IsComplete()`, an
active goal and a matching step. At `0x141201898` it ORs runtime +0x10 with
1. It then completes the active goal through `0x1412006a0` when either all
steps have bit0 set, or any completed step's **definition** flags contain
`0x40`. Quest154's six definition flags contain no `0x40`; therefore its
inspected path requires all six runtime completion bits. The existing
lifecycle trace establishes that completing its sole goal marks the quest
complete and removes the active goal/steps. A subsequent completion message
must not be treated as harmless repeated refresh.

The step table **0x142165f20** independently binds virtual +0x8 to
`0x1411ffa30`, returning runtime flags bit0. Virtual +0x30 binds
`0x14088e190`, returning runtime +0x14. The assignment helper's explicit
`!(*currStep)->m_objectiveId` assertion names the latter field; it is not an
enemy count or content key established by this trace.

544 likewise requires an active **incomplete** quest, an active goal and an
existing step. Its raw byte replaces the full runtime flags Dword without
normalizing bits. It can restore known state after fresh 527/532 construction
and before the existing 528 notification. Merely restoring all completion
bits via 544 does **not** execute 542's goal-completion effects. Completed
quest restoration needs its own policy; it must not replay 532 on an
already active goal or send further step operations after goal destruction.

## Raw codecs implemented; integration remains proposed

`crates/gw2-proto/src/quest.rs` now exposes `QuestStepFrame209026` with a
common `player_index` and `QuestStepOperation209026::{Complete, Restore}`.
`Complete { step_id }` encodes **542**; `Restore { step_id, raw_flags,
objective_id }` encodes **544**. All unsigned Dword and Byte values, including
zero and the maximum values, are preserved without normalization or content
validation. Encoding appends one frame; decoding consumes exactly one frame
and leaves input unchanged on every error. No defaults, quest selection,
ordering, duration, triggers, persistence or runtime integration were added.

`cargo test -p gw2-proto --lib quest::tests` passes **9 tests**, including
four new step tests. They cover seven literal wire fixtures, full-width
player/step/objective IDs, every raw flags byte, stream boundaries,
independent decoding through the extracted character schema, every truncated
prefix, neighboring message rejection, and overflow/unterminated varints in
each Dword position. Existing lifecycle/report tests remain unchanged and
pass. This is protocol verification, not client gameplay playback.

The next integration would add validated in-memory state for an existing
quest154/goal1 with its six exact IDs. A transition must reject unknown IDs or inactive/
completed goals before output, and duplicate completion must not issue a
second transition. On a fresh connection, known incomplete state can be
restored between the existing 532 and 528, without selecting the goal twice.
Persistence schema changes and completion/reward policy are separate work.

Future integration checks must cover exact step-ID validation, duplicate
completion and reconnect ordering. A client experiment may explicitly
complete one chosen step to verify transport/UI behavior, but such a scripted
experiment is not an authored encounter trigger. Do not bind these operations
to generic wolf kills, heart56 completion, a timer, sector entry or a guessed
NPC conversation.

The **remaining blocker for the first retail progression transition** is an
authentic action→step-ID rule, plus any required objective-instance state.
The six-step array/sector links do not establish that rule. The
[text-key follow-up](209026-quest154-inline-key-followup.md) still has no
keys for the encrypted title/objectives. The
[species-link audit](209026-great-hunt-species-link-audit.md),
[mission-point trace](209026-norn-map-mission-points.md) and
[entry geography](209026-norn-entry-geography.md) still do not establish
retail NPC placements, entry selection, encounter membership or tutorial
bounds. This transport increment changes none of those limits.

## Evidence

The frozen executable SHA-256 was asserted before each bounded query:
`99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`.
Only the exact schema-selected handlers, their direct helpers and the
previously established step table were inspected. No archive or game client
was opened, and no broad code scan or Ghidra import was used.

| REA Evidence | Scope |
| --- | --- |
| `ev_f5191874f9a8309f54f321d94a1589fd3102abcdd5b2454ff48f4c6169dc62d9` | Complete handlers542–547 and direct helper arguments. |
| `ev_96951d789bb2bd71135793ce808c37f10e531431cb56dfc693517bb6f375fb65` | Complete six helper bodies, named prerequisites and state mutations. |
| `ev_1644b70c4d0cd263d337262b6624c7847881c6b1a6db14f55ad10e5bb94d82a1` | Exact step-table targets/getters and retained schema fields. |

All three captures exited zero with owned-process cleanup verified. The
helper-body capture marks rendered-terminal truncation; all raw and normalized
bytes/frames were retained. The step virtual+0x18 sample stops at its first
return, so no complete semantics are claimed for that multi-branch predicate.
Unrelated inaccessible `/proc` entries remain capture limitations.

Complete private bundle:
`~/re/gw2/notes/209026-quest-step-state-contract-20261105-evidence.json`,
SHA-256 `32071b055f19cef3bf4ae59fdd0e412f1dd4b16c6a267a98786fefc9798b5c48`.
No raw third-party data is included in the repository.
