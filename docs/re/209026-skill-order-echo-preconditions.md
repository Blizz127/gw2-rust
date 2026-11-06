# Build 209026: research skill echo has two execution preconditions

2026-11-06, static REA plus review of our Rust source. The original research
echo used a zero duration and copied a request flag byte that does not permit
creating a new order for an already controlled character with an empty slot.
Neither setting is justified by the old session merely remaining connected.
No new client run, archive access or deployment occurred in this source slice.

## Original echo reviewed

At review baseline `08ccd7307a0a954e173255896b0e58d9644cc94b`, the
`Seen::Skill` branch in `crates/gw2-net/src/bin/game_observe/mission.rs`
copied the client23 agent index, remote tick, command flags and behavior value
into server34, and constructed this packed0x78:

| Field | Original research choice | Source status |
| --- | --- | --- |
| Packed +0 | 0x78 | Concrete active-skill order kind. |
| Packed +1 | 0 | Order flags, separate from outer command flags. No retail10176 default recovered. |
| Packed +2 | Request skill key | Type64 content lookup;10176 is the intended observation scope. |
| Packed +6 | Request slot | Skillbar slot; ordinary player request observed at0. |
| Packed +7 | 0 | Stored at order +0xf4; zero is not a recovered retail10176 value. |
| Packed +8 | Request target, or fixture character index if target0 | Agent lookup through141024060. Absent target→self remains an explicit placeholder policy. |
| Packed +12 | 0 | Bit0 adds order flag0x8000; bit1 selects action variant. This selects the first branch, not a proven retail convention. |
| Server34 trailing Word | 0 | **Not valid for the ordinary kind2 execution path below.** |

The echo is separate from health, recharge and the server84 correlated
0x44/0x45/0x47 updates. The damage/NPC-cast match arms precede this arm when
their research options are enabled; the planned isolated skill observation
disables those unrelated behaviors.

## Duration is checked before order start

Server34 handler **0x1410333e0** creates command kind2 through
**0x141035880**. That constructor ORs the received byte with **0x801000**,
installs command table **0x14211c290**, and stores the decoded trailing Word
at command **+0x70** as a zero-extended Dword.

The concrete execution chain is:

1. Kind2 table +8 → **0x14103e880**. For a bound agent passing its admission
   checks, this calls bound-agent virtual **+0x160** at **0x14103e9e0**.
2. Ordinary base-agent bound table **0x14211aee0**, +0x160 → common start
   **0x141025e80**. At **0x141025f92**, common start invokes command virtual
   **+0x10**, before its character order-start callback at **0x141025fc2**.
3. Kind2 +0x10 → **0x141039a20**, which calls its bound agent's virtual
   **+0x190**. Ordinary bound table +0x190 → **0x141032c00**.
4. The complete helper begins with **`[command+0x70] != 0`** at
   **0x141032c0a**. The assertion explicitly names **`command->m_durationMs`**.
   There is **no control-state conditional** around this check.

For a nonzero duration, that helper adds the duration to world **+0x1d4**
and passes the result to **0x140d4ba90**, then forwards duration and behavior
data to the game-bound interface's virtual+0xb0 if present. The established
[agent clock](209026-agent-command-clock.md) identifies world+0x1d4 as elapsed
milliseconds. No further timing or retail animation-duration formula is
inferred from that leaf call.

Therefore a nonzero duration is a source-proven precondition **if the command
reaches this ordinary execution path**, for a player as well as an NPC.
The old NPC assertion did not prove packed0x78 had already been constructed:
this check runs before the character order-start callback. Likewise, the old
player run's lack of assertion does not validate Word0.

A positive **u16** diagnostic duration can satisfy this particular precondition.
For example, the root-selected1000ms is an analyst-selected experiment value.
The predicted order's separate local1000ms command does not establish that
value as10176's retail cast, animation, healing or recharge duration. No
automatic duration or retail timing default is proposed here.

## Copied request flags do not admit a new controlled order

The complete receiver in the
[prediction lifecycle](209026-predicted-skill-order-lifecycle.md) proves:

- At **0x14127f393**, outer flags mask **0x10** set selects the path that may
  create a missing order through the factory.
- When that mask is clear, **0x14127f3a7** calls whole-character virtual
  **+0x350**, the `IsControlled()` getter established by
  [character control](209026-character-control.md).
- If controlled and the selected active slot is empty,
  **0x14127f3b5 → 0x14127f429** returns without invoking the factory.

The previously observed SkillBegin header had flags **3**, so copying it
unchanged omits this admission bit. A different behavior value can first
cancel/clear an existing predicted order at **0x14127f03f–0x14127f050**; the
controlled-empty-slot condition then still prevents creating a new0x78.
Other command-lifecycle work may already have removed that old order, so this
is a conditional source explanation, not a reconstruction of run126.

ORing **0x10** while preserving the request's other bits satisfies this
specific factory-admission gate. The bit also bypasses a conditional
pre-dispatch filter at **0x14103e8d5**, whose other branch may query the
game-bound interface virtual+0xa0 and destroy a rejected command. It must
therefore remain an explicit research admission choice; **0x13 is not a
recovered universal retail reply constant**.

## Correlation, upstream rejection and completion limits

The producer allocates separate behavior containers for predicted0x53 and
SkillBegin. Their values must not be equated or derived by subtracting one.
The echoed request value can become the new0x78's copied behavior value, but
this source does not prove the retail server chooses that policy. Matching
an existing order first compares the behavior value and then asserts equal
packed kind; echoing0x78 is not a typed acknowledgment of predicted0x53.
No standalone0x53 acknowledgment is added.

Common command end independently selects cancellation/completion from command
flags, as detailed in the prediction note. Receiving server34, constructing
a0x78, dispatching its start callback, returning from its completion callback
and releasing it are distinct observations. Duration alone does not prove
that every one of those transitions occurred or that health/recharge changed.

There are legitimate rejection paths before ordinary execution: network
submission **0x14102fcf0** can discard a command for an absent/removing agent
or under its exact world/base-agent flag predicates; command virtual+8 has
additional filtering. Its normal scheduled branch goes through bound-agent
+0x180 → **0x141026390**, appends the command and forwards scheduling to
**0x141032760**. The latter scheduler was not expanded here. No successful
player path that preserves ordinary kind2 execution/order-start while bypassing
the duration assertion was demonstrated. Other concrete agent bindings and
all dynamic admission predicates were not exhaustively enumerated.

These two findings support a narrowly opt-in research correction: explicit
nonzero duration plus the0x10 admission bit, preserving other request fields.
They do not recover a complete10176 reply sequence. Remaining uncertain fields
include packed flags/+7, target policy, action-selector convention, timing,
possible correlated updates and numerical healing/recharge. The
[passive lifecycle observer](209026-skill10176-lifecycle-observer.md) is intended
to report the actual typed paths; its code remains unchanged by this note.

## Evidence and repository changes

New complete private bundle:
`~/re/gw2/notes/209026-skill-order-echo-preconditions-20261106.json`, SHA-256
`1eccfd19c84017222bb12f3bf98e7312412d67c8b54ac1c4ee4014a4eeb0a0b8`.
All12 captures exited zero without output truncation and verified their owned
process-group cleanup. The pinned executable hash was asserted before every
read: `99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`.
Partial `.pdata` fragments were followed only across their demonstrated
continuations; extra displayed table words beyond kind2 slot+0x30 are not
assigned semantics.

| REA Evidence | Contribution |
| --- | --- |
| `ev_e9566369105500cd35d654f8c092bd8d361154c2f637565e50f2cbcf747033e4` | Complete duration helper and named nonzero assertion. |
| `ev_cdded77507c6ffb8622562ea9d64bbe4958c5f5e4a136f11a7f5b7a60ad4c589` | Concrete kind2 table slots. |
| `ev_ad185e6af754df6e0aeb775469056f74178d1a3bd92e2a96d15cdcc33592eda4` | Server34 construction/submission and kind2 execution dispatch. |
| `ev_8423a71345143c60fdef4e4561d61cf94f9cbd73e36f323ccbca328dbc1f0ff1`, `ev_79725761533252bc3396f57bae9a5bb6b4ed24f5b53f507024f5a3b1be347659` | Common command execution continuations through bound+0x160 and return/rejection. |
| `ev_61238a482ee0f2fa8499cb4e7291669b802c184b3e1d22715ca6498061bcbb5f` | Ordinary bound-agent+0x160/+0x190 identities. |
| `ev_cd0460aaf753f7af47b6f4da7fa7ca1c6fc10f549abf6d9e4e743096a2ae894f` | Complete network submission and pre-scheduling rejection predicates. |
| `ev_ee97a55bd727cf0bb727a5765a9ff6aef8d12b0b62ab44727960ebefefcb57bf`, `ev_e4083eadb2ffc4eb95ddf0ef05a554092350c44397ba25ded30c2ea8c9942d4f`, `ev_b22294ec0bab1adb70b5646a4555fc02aa02bfc66489c4360e86659291f07acd` | Bound+0x180 command insertion, through its scheduler handoff. |

The previously retained source records
`ev_54000555b438c2da6bade948530ff48c153e19c7fb5d9fff23e63669f2e76dac`
and `ev_6c3141ed929f205e324fa150de423ade000c755cbcc8105d9e652c27b9b2522d`
establish the complete common-start order and controlled factory gate.
The new work corrects only the Word documentation in
`crates/gw2-proto/src/skill_order.rs` and historical interpretation in the
[NPC talk-loop note](209026-npc-talk-loop.md); raw codec behavior still preserves
Word0. Separate CLI/server work owns research-option validation and reply policy.
No codec tests were rerun for this comment-only change.

## Explicit diagnostic option correction

The observer now requires `--mission-skill-order-echo DURATION_MS`, with an
explicit integer in1..=65535 and no default. `NonZeroU16` carries this input
through the mission state. The opt-in response preserves the request header
and packed fields, adds the demonstrated0x10 admission bit, and writes the
selected duration into server34's Word. Normal deployment profiles do not
enable this experiment. No healing, recharge or standalone0x53 reply is added.

Eight focused tests pass: three echo/config rejection tests, one no-listener
saved-profile preflight, two shared creation/combat cases, the10176 damage
exclusion regression, and the no-echo-without-opt-in check. The shared combat
test now sends attack10170 instead of the explicitly excluded healing10176.
A literal frame test checks the outer admission bit and little-endian duration,
and another checks preservation of nonzero request fields and existing flags.

Both debug and isolated static musl release builds pass. The release artifact
is `/home/blizz/scratch/gw2-skill-order-echo-20261106-target/x86_64-unknown-linux-musl/release/gw2-game-observe`,
1,582,288 bytes, SHA-256
`52cc0108825effb3d00a0a4561be8ab10e1f7b98742dd82802325117073f0ea2`.
Static-binary missing/zero/out-of-range argument checks also pass.
This is a tested research input correction, not a native-client cast result
or a deployed-build claim.
