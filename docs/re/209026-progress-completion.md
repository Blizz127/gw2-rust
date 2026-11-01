# Build 209026: progress completion and reward-track predicates

Static REA follow-up, 2026-11-01, to [player progress](209026-player-progress.md)
and [reward-track content](209026-reward-track-content.md). Same frozen client;
no replay or initial progress value selected.

## Completion differs from numeric progress

Progress virtual +0x40 (`0x141232080`) checks all enabled channels 0–15
from definition +0x40. It calls virtual +0x38 (`0x141231f80`) for each
and returns false on the first failure, true after all pass. For data types
2/3 (COUNTER/MAXIMUM), the per-channel predicate compares the stored
unsigned Dword against **Dword +0x10 in the parameters referenced at
definition +0x38**. The parameter pointer is dereferenced without a null
guard. Type 0 tests whether its channel bit is set. Type 1 follows a
separate bit-mask parameter path; the [bit-progress follow-up](209026-bit-progress.md)
now identifies its set-bit count and independent width/completion fields.

In contrast, virtual +0x30 returns the largest enabled channel's numeric
value, as already traced. Numeric maximum, channel completion and the
specialization's passed-threshold count are separate results.

A REA helper followed all nine Guardian progress parameter references.
All have data type 2, channel mask 0x10, and these completion limits:

| Track data keys | Progress data keys | Completion limit |
| --- | --- | ---: |
| 24, 100, 103, 30, 45 | 4858, 4839, 4894, 4890, 4899 | 60 |
| 85, 417, 466, 521 | 4838, 9611, 14185, 21488 | 250 |

Each limit equals that track's last threshold. The nine track records also
have numeric kind +0x88 = 2 and scalar +0x60 = 0. The kind's general name
is not yet established by this census. The zero scalar controls behavior
after all thresholds are passed; no broader enum name is assigned.

## Reward-track eligibility paths

Reward-manager virtual +8 (`0x141247bb0`), used by a specialization
eligibility predicate, wraps `0x141247bd0` with its third argument set to
one. Reward progress notifications call the same helper with zero.
The helper asserts a nonnull track pointer. When the argument is nonzero
and track +0x88 equals 2, it skips virtual +0xb0 and +0xb8 checks.
Other cases require +0xb0 to pass and reject completed tracks when
+0xb8 is true and track +0x60 is zero.

The later keyed-state check applies in either case. For an existing
manager +0xa8 entry whose first stored state is zero, the track's +0x58
progress reference must be nonnull, present in the player manager, and
complete. Missing entries or nonzero state pass this particular check.
This describes the observed branch, not a prescription to modify state
as a way around eligibility.

Virtual +0xb8 (`0x141248a00`) checks completion of the track's primary
+0x40 progress definition. Virtual +0xb0 (`0x1412488a0`) also evaluates a
global content predicate `0x1410f56a0`, player virtual +0x3d8's methods,
level requirements for kinds other than 2, and prerequisite references.
The non-kind-2 level path directly dereferences owned character and
coreStats. The global/player predicates are not fully traced here.

Prerequisite helper `0x141247d10` dispatches on the referenced record's
type at +0x10:

- Type 353: primary progress +0x40 complete, or an optional +0x68
  counter/maximum progress object's channel 8 is nonzero.
- Type 330: record +0x34 is at most the account scalar returned through
  player virtual +8 and account virtual +0xf8. The scalar's name is open.
- Type 0: progress reference +0x48 exists and is complete.
- Other types reach the diagnostic naming `prereqType`.

These predicates explain some later callbacks but do not establish a
complete safe initialization sequence or usable default active tracks.

## Reader scope

`gw2_dat::progress_content::ProgressDefinition209026` exposes record type 53's UID,
data key, raw data type, channel mask, and classified parameters reference.
Counter/maximum limit resolution preserves the source-container index
and checks the referenced 0x14-byte extent. Other raw data types return
no counter limit without interpreting their parameter forms.

Three focused tests pass for record/reference bounds, local and external
parameters, unequal UID/key values and uninterpreted data types. The
library and `gw2-profession-check` builds pass. A REA archive smoke resolves
all 81 specialization progress definitions and their counter/maximum limits.
An independent Python join by container/offset confirms all nine Guardian
specialization-to-track mappings and corrects the earlier documentation
ordering error. No initial progress values or eligibility policy were added.

## Evidence

Private bundle: `~/re/gw2/notes/209026-progress-completion-1.json`.
Private helper: `inspect-guardian-progress-limits.py`, using previously
recorded track/progress target locations and hash-checked container reads.

| REA evidence | Observation |
| --- | --- |
| `ev_3203314fab7fb1a15a181644f53982eca73c2ce84c9ea9431df0bd7e9a351c7e` | Complete reward eligibility helper, including keyed-state branch. |
| `ev_9366ded0a75ceeebd2a301c70caf20e976d0bc8aa3d6b20e7438f028580b9606` | Wrapper mode, track eligibility checks and per-channel completion. |
| `ev_4055f3c8816500949bbf72a4913e3996dec5c83c225a3631d44dee737f0dd044` | Guardian parameter limits, complete all-channel predicate and prerequisite helper. |
| `ev_6cf752bdb287fe59bcbafc7c3434b820921f8bb1d5b21f7e2b8f55a20c58bf3b` | Rust archive smoke: all 81 limits resolved; Guardian mapping printed. |
| `ev_e1ea43da7a028151ec52987f38f825112b8c1fb701d5c0b2bea3df05a739ba43` | Independent container/offset join confirms Guardian specialization, track and progress keys. |
