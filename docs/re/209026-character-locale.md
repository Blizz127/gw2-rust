# Build 209026: agent state and character skill locale

Static REA follow-up, 2026-11-01, to [player skill palettes](209026-player-skill-palettes.md).
No runtime replay or bootstrap default was added.

## Exact state conversion

Helper `0x14122acc0` takes agent state in ECX and previous character locale
in EDX. Its 15-entry RVA table at `0x14122ad28` selects:

| Agent state | Returned locale |
| --- | --- |
| 0, 1, 2, 3 | 0 |
| 4, 5 | Previous locale, unchanged |
| 6, 9 | 1 |
| 7, 8, 10, 11, 12 | 3 |
| 13, 14 | 2 |

Other unsigned state values reach the fatal diagnostic naming `agentState`.
The table and branches establish numeric relationships; they do not
establish names such as land, swimming or flying for these values.

The [palette getter](209026-player-skill-palettes.md) uses its +0x88 array
for locales 1/3 and its +0x60 array for other selectors. Thus state-to-array
selection depends on the previous locale for states 4/5. An emulator must
not replace that branch with a constant if reproducing this conversion.

## Constructor and state callback

Character construction at `0x1411d2e13..0x1411d2e33` calls agent virtual
+0x660, then the conversion helper with previous locale **zero**, and
stores the result at character +0xd8. Base-agent vtable `0x14211a7a0`
resolves +0x660 to `0x141027400`, which reads byte +0x1b8. The base-agent
constructor `0x141024aa0` initializes that byte to **1** at `0x141024bd1`,
before invoking the registered character factory at `0x141024ef6`.
These writes support locale 0 / the +0x60 palette array during construction.
This is not proof that intervening or later collision callbacks cannot
change state, nor a runtime observation of a finished character.

The second direct caller is callback `0x1411dd090`. Its receiver is the
character's +0x48 interface (its final dispatch recovers the whole
character by subtracting 0x48). It reads previous locale at interface
+0x90, obtains the new agent state from event +8, and runs the conversion.
There is an additional correction: when the computed locale is 1 or 2,
event Dword +0x0c is 1, and the agent virtual +0x458 result minus **100.0f**
is greater than Z returned through agent virtual +0x1c8, it stores locale
zero. The wrapper targets are `0x141027490` and `0x141027520`; they forward
to collision virtual +0x260 and +0xe0 respectively. The physical meaning
of this comparison is not fully traced here.

If the resulting locale differs from the previous value, the callback
rebuilds an existing skill bar (null checked) through `0x1411f9500`, then
notifies character listeners through `0x1411d3dcc`. It subsequently
performs further callbacks even if locale did not change. This is another
reason palette selection cannot be reduced to a one-time creation byte.

## Skill-content filter

Entry predicate `0x141212fe0` passes the same locale to `0x1411be3c0`.
The latter reads type-64 skill flags at +0x38:

| Locale | Required skill flag |
| --- | --- |
| 0 | Bit 26 |
| 1 | Bit 28 |
| 2 | Bit 29 |
| 3 | No flag required by this particular predicate |

Other locale values reach the `charLocale` fatal diagnostic. Locale 3's
unconditional return in this helper does not skip the entry predicate's
level, availability, slot, profession or optional-reference checks.
The numeric mapping narrows initial palette investigation without proving
that any candidate Guardian skill is eligible.

## Evidence

Private bundle: `~/re/gw2/notes/209026-character-locale-1.json`.

| REA evidence | Observation |
| --- | --- |
| `ev_a07a207b2a7cafd66ee32996c57cdc4791e54911491dc64f3f9caeca13552c1a` | Complete state converter, unwind ranges and branch bodies. |
| `ev_89b9c77fd04d1c348a5af86c5922f0f58620286cd3f1c7134e7fdd8384cdf5a9` | Exact 15-entry table, two direct call candidates, skill flag predicate and entry call. |
| `ev_6c4470c502109f67fcf7aee46085de277213e0997d94ffddfc15610aa3bbd629` | Constructor call and complete state callback with conditional rebuild. |
| `ev_dbdb9925daff9b1342a814401ccba216bb94ddbe837404cf3f52e4c52737a4df` | Base-agent vtable installation and character callback pointer. |
| `ev_38d3e185c1194243a9cadb554fe53052e63f9bad25f2ac1fbc0e0ef2a82ef681` | Agent vtable targets, 100.0f constant, initial state byte and aligned constructor call. |
| `ev_d8e450fdc8576aaf51f6fc2f9d2303227fcbe6d50347a052413f66b910b9b964` | State-byte getter, collision forwarding wrappers and later factory call. |
