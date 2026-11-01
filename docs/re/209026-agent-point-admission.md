# Build 209026: point magnitude check before command execution

Static REA follow-up to [server 86 position updates](209026-agent-position-update.md)
on the same frozen executable. No client replay occurred.

The server-86 constructor's `0x141030750` query is a **raw coordinate
magnitude predicate**, separate from rectangular scene bounds and collision
queries. It neither resolves nor validates the Point3 transform tag.

## Scene pointer and exact predicate

Agent-world initialization **`0x14102f680`** calls `0x141095210` and stores
its return at world **+8**. That accessor jumps to `0x140cd8310`, which
requests module index **2** from the current map's virtual +0x130. This
is the same collision scene traced in [ray construction](209026-ray-query-construction.md).
Scene constructor `0x1410b2710` installs vtable **`0x14212e240`**.

`0x141030750` forwards the input point through world +8 virtual **+0x2d8**,
whose entry at `0x14212e518` is **`0x1410bc510`**. The complete function:

1. Reads only input XYZ, converts to `[X/32, Y/32, -Z/32, 0]`.
2. Squares each SIMD lane and sums the lanes with shuffles/adds.
3. Returns the result of **scene float +0x4b8 > squared magnitude**.

The comparison is strict (`comiss` followed by `seta`). Equality and
unordered comparisons return false. With a finite positive scene limit,
infinite/NaN coordinates therefore fail this predicate. No map-bound
minimum/maximum, current agent position, transform tag or world lookup is
read by this leaf. It is a distance from the raw coordinate origin, not a
distance from the agent or a collision test.

The scene constructor writes float bits **0x4cbebc20 (100,000,000)** to
+0x4b8 at `0x1410b2cd8`. If unchanged, an exactly axial point at map
coordinate 320,000 has squared collision magnitude 100,000,000 and fails
the strict comparison. This illustrates the constructor value, not an
assertion about configured live limits. Scene virtual **+0x2c0**,
`0x1410bc7c0`, replaces +0x4b8 with the square of its input float +0x2c;
it also writes the square of input +0x28 to scene +0x4bc. The source and
live value of that configuration input remain open.

Server 86 adds **command flags +0x50 mask 0x40000** when this predicate
returns false. The incoming flag byte cannot directly supply this bit.
The codec intentionally preserves raw floats without imposing the scene's
configurable limit. A successful wire decode does not prove acceptance by
the command scheduler or a meaningful destination.

## Bounded command-admission follow-up

The complete submission path `0x14102fcf0` binds the agent and world,
drops a missing-agent command unless flag 0x8000 permits it, and drops
commands for bound interfaces marked +0x30 mask 4. There are additional
world-mode/agent-state gates. After clock bookkeeping, incoming flag byte
bit 7 selects immediate command virtual +8; otherwise bound-agent virtual
+0x180 receives the command.

For the base-agent interface, +0x180 maps to **`0x141026390`**. It performs
queue bookkeeping and inserts the command into the list at interface
+0x48, then calls `0x141032760` to ensure world scheduling. The complete
common command virtual +8 (`0x14103e880`) eventually calls bound-agent
+0x160, **`0x141025e80`**, which invokes command virtual +0x10. These
traced bodies do not directly test 0x40000. The optional character
admission callback at game-component virtual +0xa0 maps to the always-true
stub `0x1402406f0` in character interface vtable `0x142161670`.

This does **not** prove that 0x40000 is ignored: queue advancement, other
component types, diagnostics and generic flag consumers are not exhausted.
Bounded disassembly searches for direct bit-18 tests and immediate masks
found constructor writers but no confirmed consumer. Those searches do
not cover all possible compiled tests. The flag's downstream effect remains
explicitly unresolved; no invalid-point fixture is sent.

## Evidence

Private bundle: `~/re/gw2/notes/209026-agent-point-admission-1.json`
(1,363 records, 27,696,657 bytes).

| REA evidence | Observation |
| --- | --- |
| `ev_13a131bdb7cde9be91dcf5b164d90a37ed9c94ba80bbc8737e22378850961709` / `ev_df87d210eb0d26bda6ea594436c7ed42ab62b52900a0c2d38c3a1e5b6a7f4b64` | Query wrapper and submission path. |
| `ev_4e2f703caf527f3cae0a06ddfd6b7738cc9f7daed03f272c2cd68f13fe1340f5` / `ev_e93f3d895eda4441fc6a5d1903438dcbf9717cff51b6742fa3c78b59082ac85a` | World +8 initialization and scene accessor. |
| `ev_3823ca424be4bb974ae5453ec01c43aafbe0fb691f8c46d623d628bb74e72b29` / `ev_a32bce38e6a5043da6da47e4e166e3f69637589e941ecadaa2eaeb6a7651c9b8` | Scene predicate vtable/body and bound-agent queue insertion. |
| `ev_f97fc93a25be2f4812ebd8b8297717edea0a30414f76605e4fce2d470550b6d2` / `ev_226432d4503a3c9893e70db260db4b4206bff9dab18acc9e0d724d3ba216623b` / `ev_e7a94b24e55728e486fde6835a054ea425dd81eac059ee2eb041eeff0fdae558` | Constructor threshold, setter and setter vtable slot. |
| `ev_7c6a67154a4b37a5263e6be021fb4601bcd54c9139ed594e8d70587e221749e6` / `ev_b466d72020d981e005a3efca252ed2b24c62d94e84d38c02ac92297626423409` / `ev_34762fbb5f4afd5ed5a8ac5746fe711074098ecdbbcc62e4f65d4598828b70ed` | Common executor, base-agent start and character admission entry. |
| `ev_aac163345dba11a943032dd9eadf0dadd8c3230889fbf7bfbc4e059e8f63ca7c` / `ev_5bca53638ec2eff1278e1753afb1ecbd4d40d044db3492d716357fa0a705b891` | Bounded direct-test searches; not exhaustive consumer analysis. |
