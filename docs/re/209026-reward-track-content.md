# Build 209026: reward-track progress content

Static REA analysis and base-container census, 2026-11-01, for the frozen
client identified in [client-build.md](../client-build.md). This follows
[reward-track creation](209026-reward-track-messages.md) and the
[numeric-ID audit](209026-content-numeric-ids.md). No replay occurred.

## Specialization progress consumer

`0x141226390` clears the specialization slot's byte at component +0x70.
For a nonnull specialization whose owned character is controlled, it
requires the player's reward manager, reads specialization +0x50 as a
track pointer, and calls reward-manager virtual +0x20 (`0x141247ef0`).
It then scans that track's array at +0x30/count +0x38 with **0x30-byte
entries**, comparing each entry's first unsigned Dword to the returned
progress. It increments the byte for each threshold `<= progress` and
stops at the first larger threshold. The track pointer is dereferenced
without a null guard after the progress call. A changed byte can emit a
notification when the caller requests one.

The progress getter obtains player +0x9760 through player virtual +0x18.
For a nonnull track, it dereferences track +0x40 and uses the referenced
record's **data key at +0x28** to search progress-manager virtual +0x28,
`0x141231a20`. That searches 16-byte entries at manager +0x40/count +0x4c
and returns the matching object pointer at entry +8, or null. An existing
object supplies virtual +0x30; missing progress returns zero.

This lookup does not read the four selected track slots in the reward
manager. Assigning tracks with server 565 therefore does not itself grant
specialization progress. The [progress-message follow-up](209026-player-progress.md) maps server
511 to counter/maximum channel updates and confirms Guardian channel 4.

## Complete base census

The private helper `inspect-reward-track-content.py` visited all 32 base
containers in four bounded groups. It classified/followed local and
external pointers and checked each 0x30-byte entry's threshold field.
Source hashes accompany the private results.

- All **529** type-353 tracks have a nonnull +0x40 reference to **type 53**.
- Each has 4–801 threshold entries. Every inspected sequence is
  nondecreasing; this does not imply distinct thresholds.
- All nine tracks referenced by the Guardian specialization catalog are
  in file 1282859; their progress definitions are in file 1282842.

| Specialization data key / UID | Track data key | Type-53 progress data key | Threshold count | First / last |
| --- | ---: | ---: | ---: | --- |
| 13 / 15 | 24 | 4858 | 12 | 2 / 60 |
| 16 / 19 | 100 | 4839 | 12 | 2 / 60 |
| 42 / 57 | 103 | 4894 | 12 | 2 / 60 |
| 46 / 64 | 30 | 4890 | 12 | 2 / 60 |
| 49 / 69 | 45 | 4899 | 12 | 2 / 60 |
| 27 / 36 | 85 | 4838 | 19 | 30 / 250 |
| 62 / 76 | 417 | 9611 | 19 | 30 / 250 |
| 65 / 32 | 466 | 14185 | 19 | 30 / 250 |
| 81 / 77 | 521 | 21488 | 18 | 40 / 250 |

Correction: the original table paired four specialization rows with tracks
by list order. Joining the recorded container/offset pointers and the Rust
archive check independently establish the mapping above. In particular,
specialization key 13 / UID 15 uses track 24 and progress 4858. See the
[completion follow-up](209026-progress-completion.md) for both evidence records.

The five 12-entry sequences are exactly
`[2, 5, 8, 11, 15, 20, 25, 30, 36, 44, 52, 60]`.
Track 85 has `[30, 40, 55, 65, 75, 85, 95, 110, 125, 135, 145, 155,
165, 180, 190, 200, 210, 225, 250]`.
Tracks 417/466 have `[30, 40, 50, 60, 70, 80, 90, 105, 120, 130, 140,
150, 165, 180, 190, 200, 210, 225, 250]`.
Track 521 has the latter sequence without its first value 30.

For these Guardian records, missing player progress yields zero passed
thresholds. This is a bounded static consequence of the getter and
threshold values, not proof that a zero-progress specialization is a
playable default. The 801-entry maximum belongs to the broader track
catalog; it is not evidence that a specialization byte consumes that
particular track. Later work traces [eligibility](209026-progress-completion.md) and
identifies the [slot bytes as pin values](209026-reward-track-slots.md);
their allowed domain remains open. No initial track selection is prescribed here.

## Evidence

Private bundle: `~/re/gw2/notes/209026-reward-track-content-1.json`.
Private census files `reward-track-content-209026-{0-8,8-16,16-24,24-32}.json`
retain the individual thresholds, target locations and source hashes.

| REA evidence | Observation |
| --- | --- |
| `ev_3b885ec6649515cdb21bba09d21daeadf98cefda8b33c2de30b606a0535d4fc1` | Complete specialization threshold-count helper and reward vtable. |
| `ev_23bb63eba911c91755983a354f749d3a463f19c53dcf624992ee294a12c2ff8d` | Containers 0–7: 123 tracks. |
| `ev_737b95c2d946d1ed1ec767545013ff9ee9464dbb7e785b1f4bf107d5ddf197b4` | Containers 8–15: 92 tracks. |
| `ev_930e86396cf8da7389602026c1c3bb81d473bee1fd6b9153890883f5ef2bb323` | Containers 16–23: 269 tracks. |
| `ev_e31a5b7ea820de9b85abee7b7358f57a52fd6ba12fec23cdb9a58f4c7194bc2d` | Containers 24–31: 45 tracks; all Guardian progress mappings. |
| `ev_7db3013127435d4ce4b99a03b4a41672a1ec4a5833bbf557a83b351ce119570c` | Complete progress getter, sorted lookup and progress-manager vtable. |
