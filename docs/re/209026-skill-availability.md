# Build 209026: player skill availability messages

Static REA findings, 2026-11-01, on the frozen executable identified in
[client-build.md](../client-build.md). This follows the availability bitset
found during the [skill-palette census](209026-skill-palette-content.md).
No messages were sent and the runtime User Agreement hold remains.

## Wire layouts and effects

These protocol 0/group 0 messages address a **player index**, resolved
through character-context virtual +0x118. Skill references use content
virtual +0x230 (`0x1412e1e90`), which resolves **type 64**, and must be
nonnull. Zero is not handled as a clear-selection sentinel here.

| ID | Descriptor / handler | Fields in wire order | Effect |
| --- | --- | --- | --- |
| 605 | `0x1425d7100` / `0x14125b300` | Dword skill key, Dword player index | Sets the skill's availability bit without rebuilding or notifying in the inspected helper. |
| 606 | `0x1425d7220` / `0x14125b3c0` | Dword player index | Clears the availability bitset and the separate keyed table at skill-object +0x40. |
| 609 | `0x1425d75b0` / `0x14125b550` | Dword skill key, Dword player index | Removes that key from +0x40, clears its bit if allocated, rebuilds the owned character's skill bar and notifies. |
| 613 | `0x1425d7c20` / `0x14125b8f0` | Dword skill key, Byte flags, Dword player index | Removes that key from +0x40, sets its bit, rebuilds the owned character's skill bar and notifies. |

Registration pairs are `0x1421708e0`, `0x1421708f0`, `0x142170920` and
`0x142170960` respectively. Dwords use the existing unsigned varint
encoding. The meanings of +0x40's values remain unresolved.

## Initial loading versus live changes

605 calls `0x141228c30` on player +0x9be0. It reads the resolved skill's
key at +0x28 and calls bit setter `0x140317e70` on skill-object +0x20.
That helper grows storage to `(key >> 5) + 1` Dwords if necessary,
zeroes newly added words, and ORs `1 << (key & 31)` into the selected word.
It does not read an owned-character pointer. The inspected path therefore
supports initial bit loading after player creation and before allocating
the character's skill bar, provided the skill key resolves.

606 calls `0x141228c80`: it frees the buffer at skill-object +0x28,
zeros that pointer and the size/capacity fields at +0x30, then clears the
keyed table via `0x141227530`. Neither inspected helper rebuilds the skill
bar or clears the selected palette arrays at +0x60/+0x88. The operation
does not imply that displayed skills are immediately recomputed.

609's helper `0x141228f40` and 613's helper `0x1412292a0` both call
`0x1412269d0` to remove the key from the separate table. The former clears
the availability bit only if its word exists; the latter uses the growing
bit setter. Both then read player +0x20 (owned character), dereference
character +0x520 (skill bar), and call rebuild wrapper `0x1411f9500`
without null guards. This requires the ownership/component setup that
605's direct path does not require.

For 613, the handler extracts flags bit 0 and bit 1 into separate boolean
arguments; the setter forwards them through the listener notification
after rebuilding. Higher flag bits are unused by the inspected handler.
The boolean names and downstream listener behavior are not yet mapped.

These are availability-state mutations, not proof that every bit grants
a usable skill. The full predicate also checks slot, level, locale and
other context. Initial palette selection 612 remains a separate update
with its own immediate-rebuild dependency.

## Typed frames and focused checks

`gw2_proto::skill_availability::SkillAvailabilityFrame209026` represents
605/606/609/613 as initial-set, reset, remove and set-and-notify operations.
Player index is encoded after operation fields. The codec preserves all
raw flags and key values; it does not validate content or player lifetime.

Three focused tests and `cargo build -p gw2-proto --lib` pass. Independent
hex fixtures cover all four layouts, different key/index widths, u32
maxima, zero keys and high flag bits. Checks cover output prefixes,
concatenated-frame boundaries, every truncated fixture, IDs adjacent to
or between the supported set, and overflowing/overlong varints in every
Dword position. These are framing checks, not runtime validation.

## Evidence

Private bundle: `~/re/gw2/notes/209026-skill-availability-1.json`.

| REA evidence | Observation |
| --- | --- |
| `ev_5630ef2060fb3a3ffcba5c94273c4e7a1480edf2a83dd14a6232aae63175e114` | Bit setter/reset wrappers and mutation search. |
| `ev_6af6ede51cebdd0884d57ad6c3a8448448e666cdebbdcca09ef926004ffdd6b0` | Direct message callers and growing/zero-filling bit setter. |
| `ev_1a3e4a691d4bdbb01fdd3cae1fd2a59e93d10a2cc3f458147e737e96aa6781e4` | Function boundaries, registration pairs and complete live-change helpers. |
| `ev_e12f7f164d4bb012f13e47d41eaa2fd0e7c055114d10137058cb3adc32c4b3c2` | Hash-guarded descriptors, all four handlers and content lookup target. |
| `ev_a5ac1b27c6cfae9449f8ab75180ca82927a0dcc3214616ae9df012b0f52c0091` | Type-64 lookup, keyed-table removal and reset. |
