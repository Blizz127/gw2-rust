# Build 209026: NPC talk loop (conversation, talkable gate, interact request)

Static analysis, 2026-11-05, of frozen `Gw2-64.new.exe`, SHA-256
`99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`
(objdump plus targeted reads; no client launch, capture or replay). Field orders
match `data/protocol/209026/server-tables.xml` and `client-tables.xml`, which the
`gw2_proto::npc_talk` schema test checks. Items marked *inferred* are not proven.

**Untested in game.** The loop below is derived from client code only.

## Loop summary

1. Server 37 spawns the NPC (`IsMonster`, identity `0x20000000 + species`).
2. Server 151 `{character index, 0}`: attitude friendly (0, or 3) toward the player.
3. Server 378 `{character index, 1, 0}`: talk flag (character bit 13). The F prompt needs it.
   Don't send 590 to the NPC; it sets `char+0x1c8`, which blocks the prompt.
4. The player presses F: client 23 with behavior data `[0x14][u32 target LE]`, where the
   target is the NPC agent's +0xc id. Which server id that equals is unverified.
5. Server: 762 any active conversation, then 758 open, 760 page, 756/757 per
   option, 761 page ready.
6. Client 275 `{conversation, response}`, then the next 760 (stage+1) and so on,
   or 762 close. The client acknowledges every close with 276.

## 1. NPC dialogue: Conversation protocol (server 756–764, client 275/276), traced from the handlers

**Wire format:** serialize these fields **by ordinal**, in the order listed, using
the existing message-definition framing (strings use the normal length-prefixed
framing). The `(+0x..)` values are offsets in the client's *decoded* message struct,
where every string is an 8-byte pointer. They are evidence only, not wire offsets.
Don't hand-pack a buffer at them.

The context is game context +0xf0 (vtable `0x14223a508`) and holds one active
conversation (ctx+0x8) and one pending (ctx+0x10). The conversation object has vtable
`0x14223a6f8` and is refcounted. Strings are copied as UTF-16 into fixed buffers. Plain
versus coded string-table text is **not verified**: `TextValidateCoded(codedLink)` sits
next to this code. See the live test below.

| id | fields in order | what the client does (confirmed from code unless *inferred*) |
|---|---|---|
| **758 Open** | 1 Dword A (+2), 2 Dword convId (+6), 3 Dword stage (+a), 4 Byte (+e), 5 String[64] speaker (+f) | ctx create (vtbl+0x10): conversation +0x8=A (likely the NPC agent id, *inferred*), +0xc=convId (the key every later message uses), +0x38=stage, +0x10=byte. The String[64] is stored at +0x18/+0x20; 759's chat echo uses it as the **sender name**, so it is the NPC/speaker name. |
| **760 Page** | 1 convId (+2), 2 stage (+6), 3 timerMs (+a), 4 Float (+e, ignored), 5 Byte (+12), 6 Dword flags (+13), 7–9 Dword (+17,+1b,+1f), 10 Byte (+23), 11 Dword (+24, ignored), 12 Dword (+28), 13 String[512] text, 14 String[64] speaker, 15 String[128] | conv vtbl+0x38 (`0x14130ee30`). Sets stage=field 2, **resets ready (+0x98=0) and selection (+0x90=−1), and clears the response list**. Timer: +0x3c=timerMs, remaining = timerMs−250 when >250 (ConvTimer). +44=f5, +48=flags, +4c/+50/+54=f7–9, +58=f10, +94=f12. Unless flags bit0 is set, it builds the line object (0x588): text, speaker, optional String[128] (portrait or model, *inferred*). **Flags 0x80 also prints the text to chat as the speaker.** |
| **756 Add response** | 1 convId, 2 stage, 3 respId, 4 defId, 5 Dword, 6 Dword, 7 Dword, 8 Byte, 9 String[512] text | Only if convId and stage match the active conversation. Appends a response (0x458). Response fields: +418 respId (echoed by client 275), +448 = def resolved from defId when nonzero (game ctx+0xe0 vtbl+0x230; probably a cost/item def for ConvCost, *inferred*), +440=f5, +430=f6, +434=f7 (icon/kind, *inferred*), +43c=f8, +41c=0, +420=2. The page's +44/+4c/+50 are copied into +428/+424/+42c. |
| **757 Add response (short)** | 1 convId, 2 stage, 3 respId, 4 defId, 5 Dword, 6 Byte, 7 String[512] text | Same as 756 with +440=f5, +434=9, +43c=10, +430=0. If f6 is set, +41c/+420 come from conversation vtbl+0xc0/+0xc8; otherwise 0/2. |
| **759 Chosen/previous response** | 1 convId, 2 stage, 3 Dword X, 4 respId, 5 Dword, 6 Dword, 7 String[512] text, 8 Dword flags | Checks stage only. Sets selection=respId (+0x90). Flags 0x200 echoes the text to chat (sender = 758 speaker). **Unless the page's flags bit0 is set**, it replaces the single "previous response" object (+0x68, ConvPrevResponse, *inferred*) with that text and calls vtbl+0x108(X). |
| **761 Page ready** | 1 convId, 2 stage | If the stage matches: ready (+0x98=1), then notifies listeners. |
| **762 Close** | 1 convId | ctx destroy (vtbl+0x18) closes the active (or pending) conversation with that id. The client answers with 276. |
| **763 Update response** | 1 convId, 2 stage, 3 respId, 4 value | That response's vtbl+0x70(value), then listeners (vote count or enabled state, *inferred*; there is a ConvResponseVote widget). |
| **764 Set selection** | 1 convId, 2 stage, 3 respId | selection (+0x90)=respId, then listeners. |
| **client 275 (0x113) Choose** | 1 Dword convId, 2 Dword respId | The player picked a response; it also sets the selection locally. |
| **client 276 (0x114) Closed** | 1 Dword convId | Sent whenever the client tears down the **active** conversation (`0x14130dad0`, notify=1): **its reply to 762**, a 758 that displaces an active conversation, and UI close paths (`0x14130d9f7`, `…da63`, `…dc1d`). |

**Server rules (each follows from the client code above):**
1. **Order on every page is 760, then that page's 756/757s, then 761.** 760 clears the
   responses and resets ready, so a 761 sent before the responses, or kept from the
   previous page, leaves the page not ready.
2. **Never send 758 while a conversation is active.** In that case the client closes
   the old one (sending 276) and leaves the new one *pending*. Nothing on that path
   promotes it, and later messages only reach the active one, so 760/756/761 for the
   new convId are silently dropped. Send 762 for the old conversation first; with
   nothing active, 758 is promoted immediately.
3. **Treat 276 as a close acknowledgement** (drop server state for that convId), not
   as a new request. It also arrives after the server's own 762.

Minimal "talk" flow: 758 → 760 (flags 0 or 0x80) → 756/757 per option → 761 → client
275 → next 760 with stage+1 → … → 762 (→ client 276). How the player *starts* a
conversation (the interact action on an NPC, and which agent flag makes the NPC
talkable) is not traced yet.

**Live test proposed to Codex (it owns the deploy):**
- On the synthetic test character, right after world entry, send unprompted: 758 → 760
  with flags 0x80 and a short plain ASCII line plus a speaker name → one 757 → 761.
  No interact is needed.
- Flag 0x80 also echoes the line to chat, so one screenshot shows whether plain UTF-16
  renders, whether the layout is right, and whether the response appears after 761.
- If the text is garbage or blank, the strings are coded string-table text. The next
  static step is then the ConvLine widget's consumer.

## 2. Making an NPC talkable: the interaction-selection gate (server 377/378/379)

The client only shows the interact prompt (F) on an agent that passes
`AgentCanBeInteractionSelection` = **`0x1410fb2e0`** (AsContext.cpp, assert string
`!agent || AgentCanBeInteractionSelection(agent)`). It switches on agent type
(agent+0x8): 0 = character, 0xa = gadget, 0xf = item agent. Character vtable
`0x142160be0` (the same one as `209026-npc-character-foundation.md`). The character
flag word is the **u64 at char+0x178**.

**Character path**, in code order (confirmed). Let `att` = attitude from
`ctx+0x388 vtbl+0x58(agent)`, with friendlyish = att ∈ {0,3} and hostileish =
att ∈ {1,2}. The names follow the usual GW2 enum Friendly/Hostile/Indifferent/Neutral
and are *inferred*.
1. Reject if slot +0x498 is true (flag bit 24, or bit 53 plus a further check).
2. If the character is an "identity 0x3xxxxxxx" type (char+0xa0 top nibble = 3), skip
   straight to step 5; such characters never use the talk bits. IsMonster NPCs are
   nibble 2, so this doesn't apply to them. Otherwise it needs `char+0x1c8 == 0` or
   flag bit 42, else go to step 5.
3. It needs alive (`char+0x188 == 0`) or flag bit 49, **and** friendlyish or bit 42.
4. **Talkable if** slot +0x440 is true, i.e. **flag bit 13**. For IsMonster NPCs,
   when a global setting has bit 0x40 the client also requires bit 38. **Or talkable
   if** slot +0x478 is true, i.e. **flag bit 14** and not `vtbl+0x2c8(1)` bit 13.
5. Fallbacks: hostileish plus slot +0x110 (a combat-target path), or slot +0x118
   (flags bit1 set, bit32 clear) plus not alive plus friendlyish. After that come
   player-side checks.

**Attitude source: server 151 (already sent by Codex's population code).** AsContext+0x388
is game ctx+0x98, and its vtbl+0x58 is `CharClientContext()->GetAttitudeToward(agent)`;
the WorldMapModel switch at `0x140974073` names it with that assert string. Server
**151 {Dword agent, Dword attitude}** (`0x14124b300` → setter `0x1411da150`) writes
`m_attitudeTowardControlled` at char+0xc0, bounded by `AFFINITY_ATTITUDES` = 4; see
`docs/re/209026-character-core-stats.md`. That this getter returns char+0xc0 for
characters is strongly implied but not traced.
Value labels (*inferred* from two client uses that agree):
- The interact gate groups {0,3} as non-attackable and {1,2} as attackable.
- The world-map switch has separate cases for 0 and 1 and a shared case for 2/3.
- Both fit GW2's order **0 Friendly, 1 Hostile, 2 Indifferent, 3 Neutral**.

**char+0x1c8 (the gate's step 2):** the character constructor zeroes it (`0x1411d2ad3`).
Its only setter, `0x1411e16d0`, is reached from **server 590 {Guid, Dword agent,
Dword value→+0x1c8, Byte, Byte}**. Owner/claim-like (*inferred*). Don't send 590 to a
talk NPC; with it unset, step 2 passes.

**For a talkable NPC send 151 with 0 (or 3).** Codex's current placeholder `ATTITUDE = 0`
already qualifies. Send 151 after server 37 (the handler ignores a missing character).

**Which server messages set bits 13 and 14** (both setters assert `IsMonster()`, so
they only apply to NPC characters with identity `0x20000000 + species`):

| id | fields in order | effect |
|---|---|---|
| **378** | Dword character index, Byte **talk**, Byte | Byte 1 sets/clears **flag bit 13** (`0x1411e1750`). Byte 2 sets/clears bit 20 (`0x1411e1df0`). |
| **379** | Dword character index, Byte **interact** | Sets/clears **flag bit 14** (`0x1411e17d0`). |
| **377** | Dword character index, String[128] (name, via `0x1411e1e60`), Word flags, Optional{Qword Qword Qword}, Optional{Float, Dword×4, Float} | The full NPC flag set. Word flags: 0x1→bit 14, 0x2→bit 18, 0x100→bit 56, 0x200→bit 57, 0x4/0x8/0x10/0x20/0x40/0x400→other setters (unnamed). |

The agent Dword is resolved through game ctx+0x98 `0x1411b4500`, the same character
lookup as the other Char messages.

**Minimum for a talkable friendly NPC (*inferred from the gate*):** spawn it as an
IsMonster character with friendly or neutral attitude, alive, with `char+0x1c8` unset.
Then send **378 {character index, 1, 0}** (or 379 {character index, 1}). Without bit 13 or 14 there is no
F prompt, whatever the conversation messages say. **Send 378/379 only after server 37
has created the NPC.** The handlers resolve the agent through `0x1411b4500` and assert
`character` when it's missing, and the setters assert `IsMonster()`.

**Client → server when F is pressed: client 23, packed opcode 0x14.** The producer is
confirmed from code. The link from the F key to order slot +0x148 is *inferred* (it
wasn't traced), and the live test below settles it. This builds on `docs/re/209026-mesmer-skill-command-path.md`, where client 23
behavior data is a packed buffer led by an opcode byte and sent through base-agent
virtual +0xa8 with outer command flags 3.
- **The producer** is order-vtable `0x142179b90` slot +0x148, function `0x141282d90`
  (ChCliOrder.cpp). It allocates 5 bytes: **`[0x14][u32 target agent +0xc]`**. If the
  target is a gadget whose gadget vtbl+0x70 is true, it sends **`[0x15][u32 target]`**
  (InteractChannel) instead. For an NPC character that check is 0, so the opcode is
  always 0x14. The target id is the same agent +0xc id Codex's 0x23 skill request uses.
- **Opcode names.** The enum is *not* the schema list order: opcode 0x04 is a 1-byte
  producer, but the 5th schema entry (EnterGlideMode) is 18 bytes, and 0x04–0x11 holds
  14 opcodes against 6 schema entries. Field types are 5=u8, 10=u32, 14=Point3
  (12 bytes), 22=Guid. Names backed by independent evidence: **0x14/0x15** (gadget-channel
  branch plus adjacency), **0x1f Pickup** (`itemAgent` assert), **0x23–0x27** (they
  reproduce Codex's lengths). The rest are size-plus-order guesses (*inferred*):
  - 0x00 Abort (3 bytes: op, orderTrack, reason)
  - 0x01 Charm (5)
  - 0x12/0x13 Finisher/FinisherCustom (5)
  - **0x14 Interact (5), 0x15 InteractChannel (5)**
  - 0x1f Pickup (5); its producer asserts `itemAgent`
  - 0x21/0x22 Resurrect/ResurrectCustom (5)
  - 0x23 SkillBegin (10)
  - 0x24 SkillBeginFacingPoint (22), 0x25 SkillBeginTargeted (22), 0x26 SkillBeginTargetedRotation (34), 0x27 SkillBeginTargetedTransform (38)

  The 0x23–0x27 sizes reproduce Codex's independently traced lengths. Names for the
  1-byte opcodes are not assigned here. Signal schema list: descriptors at
  `0x142175bd0`…`0x142176c00`. Producer table: `~/re/gw2/notes/209026-mesmer-re-handoff-20261105/packed-producers.tsv`.
- **Live-test check:** log the decoded client 23 payload with Codex's
  `decode_client_frame`; no plaintext capture is needed. Expect byte 0 = 0x14.
  Compare bytes 1–4 with both the server-37 agent index (a Word) and the character
  index Codex assigned for 151/378 (a Dword). Whichever matches is the id the server
  should use to find the NPC on interact. If there's no F prompt after 378, suspect
  attitude or `char+0x1c8` (below).
- **Server side:** on client 23 with packed `[0x14][agent]` for a talkable NPC, start
  the section 1 flow (762 any active conversation first, then 758 …). Whether retail
  also sends a server 84 order notification for interact isn't established; skip it
  until a client symptom shows it's needed.
- A producer side effect: it also calls `0x1411db060(char, 0, 0, 1)` and target vtbl
  +0x2d8 locally before sending. Those are client-side presentation, *inferred*.

## 3. Server wiring on branch `claude/npc-talk-loop` (handoff to Codex)

Opt-in and off by default; deployed services are unchanged unless the flag is added.

```text
gw2-game-observe ... --mission-preamble --mission-map-key KEY <population flags> \
    --mission-talk-npc SPECIES,X,Y,Z [--mission-talk-npc-speaker NAME]
```

- **Spawn.** `MissionPhase::respond` appends `TalkNpcPlan209026::spawn_frames` to the
  AgentsLoaded batch, after 1036/431/83 and any skills/quest. The frames are 37 (type 0,
  identity `0x20000000 + SPECIES`), 183 coreStats (level 1/1, profession 0, zero
  appearance; placeholders), 282 inventory, 151 (0), 256 (alive), 170, 46 (flags 0) and
  378 `{2, 1, 0}`. The NPC is agent and character index **2**; it refuses to spawn if
  that collides with the player's character index. It works with both
  `--mission-population` and the saved-population (`--character-store`) path, because
  it attaches to `MissionPhase` itself.
- **Interact.** `MissionPhase::push` decodes client 23 behavior data with
  `InteractRequest209026`. `[0x14][2]` becomes `Seen::Interact`, and the reply
  is `TalkSession209026::on_interact`: 758, a 760 page (flags 0, no chat echo),
  757 Continue/Goodbye, then 761. Every other client-23 payload stays
  `Seen::Other(23)`; the log prints its leading opcode and length.
- **Replies.** Client 275 Continue sends the next page, and Goodbye (or the last
  page) sends 762. 276 only clears state. Interacting again while a conversation is
  active sends 762 first.
- **Text** is placeholder, marked `[test]`. Retail lines are server data.

**Choosing SPECIES (open):**
- No tutorial species key is established.
- 3172 resolves to the name `Eir` but has a null direct model field, so it may be
  invisible.
- 8013 (`Juvenile Boar`) and 1544 (`Branded Minotaur`) have direct models
  (`docs/re/209026-species-plain-name-coverage.md`).
- A visible species is the better first smoke test. Place it a few map units from
  `--mission-spawn`.

**Live smoke (Codex/owner; not run).** Expected log lines, in order:
1. `Mission phase: spawning talk NPC agent 2 (species …) with 378 talk flag`
2. Walk to the NPC and press F: `Mission phase: client 23 interact Interact { target: N }`.
   - `N == 2` confirms the target is the server-side index. It can't tell agent index
     from character index, which are equal here (`population.rs` sets them equal); that
     only matters if they ever diverge.
   - Any other N appears as `not the talk NPC`. Then the server must map N to the NPC.
3. The dialogue window opens, the line shows in chat, and Continue/Goodbye produce
   `conversation reply Choose …` lines.

No F prompt means the gate failed (section 2); check the NPC appeared first. A
`client 23 opcode Some(0x..)` line on F means the key sends a different request
than producer `0x141282d90`.

**Crash risk on the first F (checked).** Every conversation handler dereferences the
context at game ctx+0xf0 without a null test. That context is registered in the same
global context-factory table as the character context: slot 0x1e (×8 = +0xf0,
`0x14130de60` → factory `0x14130de90` → `0x14130d780`), next to the character context's
slot 0x13 (= +0x98, `0x141242c70`). The uniform registration means it exists whenever
151/378 work. That's strong static evidence, not a runtime check, so do the first
smoke in a sandbox run anyway.

**Verification on the branch (2026-11-05):**
- `cargo test -p gw2-proto --lib`: all pass (9 npc_talk tests).
- `cargo test -p gw2-net --bin gw2-game-observe`: 87 passed, 3 ignored. That includes
  the spawn-after-BeginPlaying, interact → conversation, 275/276 and raw-23 tests, and
  the existing every-client-message dispatch test.
- Clippy on gw2-proto/gw2-net is clean for the new code. The remaining workspace errors
  are pre-existing, in untouched files: `gw2-map-probe.rs:386`, `creation_scene.rs:26`
  and `saved_population.rs:364`.
- Not run in game. Nothing deployed.

## 4. Sandbox smoke results (local, Docker network=none, synthetic account)

The runs are copies of Codex's run118 Norn Mesmer flow on map 379, with
`--mission-talk-npc 8013,-3120,-25600,-3742`.

| Run | Change | Result |
| --- | --- | --- |
| 121 | First version (no coreStats) | Client stuck on the loading screen. Assert `ChCliCharacter.h:565 "m_coreStats"` when the NPC entered the world. **The NPC needs coreStats** (Codex's foundation note flagged that consumers might need it). |
| 122 | + 183 coreStats, 282 inventory | **In world.** The NPC renders as a friendly green-named Juvenile Boar, and the client shows **"Greet [F]"**, so the interaction gate passes with 378. Pressing F sends **client 23 `[0x14][2]`**: target 2 = the server agent/character index. The server opens the conversation, then the client asserts `CtApi.cpp:141 "TextValidateCoded(codedName)"`: the 0x80 chat echo needs a **coded** speaker name. |
| 123 | Page flags 0 (no chat echo) | **The dialogue window opens without an assert**, with two response rows (heart icons, i.e. Continue/Goodbye). The text areas are blank: **the dialogue strings must be coded text**, not plain UTF-16. |
| 124 | Coded text via `coded_literal` (plain template 46217 `%str1%`) | **Full loop in game, no asserts.** The window shows speaker `TalkTest`, the line `[test] Well met, traveler.` and the responses `Continue.`/`Goodbye.`. Clicking Continue sends **275 {1, 1}** and page 2 renders with only Goodbye. Clicking Goodbye sends **275 {1, 2}** and the server sends 762. The client closes the window and acks with **276 {1}**, and the "Greet [F]" prompt returns. |

**Coded text format** (`TextValidateCoded` `0x1410dcca0`, grammar `0x1410e7350`):
- The first unit, with bit 15 clear, must be at least 0x100: a text-ID token (bit 15 =
  continuation).
- Then parameters: 0x107–0x10c take literal units directly, 0x10d–0x112 take nested
  coded text, and 3 (`CONCAT_LITERAL`) starts literal units of at least 0xa, ended by 1.
- Showing arbitrary text therefore needs a text ID whose template is just a parameter.
- Retail dialogue would use the NPC's real text IDs. Most English strings are encrypted
  (marker ≠ 0) and need server-supplied keys. About 4% are plain.

**Coded literal text (used since run 124):** `[0x8101 0x3689][0x107]<UTF-16 units>[0x0001]`.
That is text ID 46217, whose plain English template is `%str1%`, with STR1 set to the
literal. A scan of all 1,176 English string packs found **207,236 plain (keyless)
strings**, 80 of them pure `%str1%` templates (`plain-texts.py`/`plain-texts.json` in
`~/re/gw2/notes/209026-mesmer-re-handoff-20261105/`). Retail lines would use the NPC's
real text IDs. Only the plain ~4% render without server-supplied decryption keys.

Still open: the 0x80 chat echo with coded text, timers and costs (756 definitions),
and real retail dialogue/NPC data.

## 5. Hostile NPC and the first decoded attack/skill requests (run 125)

`--mission-hostile-npc 1544,-3000,-25600,-3742` spawns agent 3 with attitude 1 and no
378 (same 37/183/282/151/256/170/46 spawn). The client shows a **red "Branded
Minotaur"** nameplate, confirming that attitude 1 is hostile. Non-interact client-23
payloads are now logged in hex. With Tab, key 1 and key 6 pressed:

| Order | Behavior value | Payload | Reading |
| ---: | --- | --- | --- |
| 1 | 0x80000006 | `29 05` | Opcode 0x29 with byte 5. Unnamed; precedes the cast. |
| 2 | 0x80000007 | `2a 00` | Opcode 0x2a with byte 0. Unnamed. |
| 3 | 0x80000009 | `23 00000000 c0270000 00` | **SkillBegin 0x23**: target 0, skill **10176** (0x27c0, the heal Codex selected), slot byte 0. Matches `docs/re/209026-mesmer-skill-command-path.md` exactly. |
| 4 | 0x8000000a | `29 00` | Opcode 0x29 with byte 0. |

- The behavior value is `0x80000000 | sequence`, a per-request counter.
- Outer command flags are 3 on all of them.
- Key 1 sent nothing; the weapon slot is locked on the current skill bar.
- The heal used target 0 even after Tab. Whether Tab targeted the minotaur isn't
  established from the screenshots.
- These are the 15/23/15-byte frames Codex recorded only by length in run 116.

The server sends no reply yet. Executing the skill (the 0x78 order, server 84
notifications and recharge) is the open step in Codex's skill note.

## 6. Skill execution hypothesis: server 34 + packed 0x78 (run 126, inconclusive)

**Command flags of every buffer-carrying agent message** (handler → constructor
→ command kind → constant ORed into the flags):

| ID | Command kind | Flags OR | Bit 12? |
| ---: | ---: | --- | --- |
| 32, 55 | 15 | 0x8010000 | no |
| **34** | **2** | **0x801000** | **yes** |
| 35 | 3 | 0x8801000 | yes |
| 37, 38, 40 | 5, 6, 7 | 0x18080 | no |
| 42 | 9 | bit 16 | no |
| 52, 53, 54, 56, 66, 79, 80 | 12, 13, 14, 16, 26, 35, 36 | 0x8010000 | no |
| 57 | 17 | 0x8b01000 | yes |
| 58, 59 | 18, 19 | 0x8301200 | yes |
| 60, 69 | 20, 29 | 0x301000 | yes |
| 62 | 22 | 0x901000 | yes |
| 63, 65, 68 | 23, 25, 28 | 0x101000 | yes |
| 67 | 27 | 0x8301000 | yes |
| 81, **84** | 37, 41 | 0x12000 | **no** |
| 85 | 42 | 0x8112000 | no |

Codex's executor trace says order start needs **bit 12 set and bit 17 clear**. None of
these sets bit 17. Server **34** (`0x1410333e0` → `0x141035880`, kind 2, vtable
`0x14211c290`) is the plain behavior header plus one Word (stored at command +0x70).
It is the simplest bit-12 carrier.

**Run 126** (`--mission-skill-order-echo`) answered the client's SkillBegin
`[23][0][10176][0]` (value 0x80000009, tick 1799, flags 3). The answer was server 34 with
the same agent, tick, flags and value, data `[78 00 c0270000 00 00 01000000 00]`, and
Word 0. The client **accepted it without an assert or disconnect**, but **nothing visible
happened**: no cast, no cooldown, health unchanged.

Candidates for the next trace:
- the executor's other completion guards for kind 2;
- whether order +8's agent identifier uses the 37 index space (`0x141024060`);
- packed order-flags gates (masks 0x10/0x4);
- the client's own predicted order (kind 0x53) for the same value.

`skill_order.rs` provides the codecs (SkillBegin parse, 0x78 builder, server 34).

### Why run 126 did nothing: the controlled-character 0x78 gate (`0x14127ef80`)

The order-start entry on the character's order component (+0x4b0) checks, in order:
1. **Packed order flags.** Bit 0x10 rejects the order. An existing order in slot
   `(outer flags >> 5) & 1` is compared via `0x1412874a0` and kept or cancelled. Bit 0x4
   takes the alternate path.
2. **Controlled character** (order component +0x58 bit 0, the local player): for a
   packed **0x78**, the skill definition is resolved by the key at +2. The order is
   **dropped unless skill definition +0x38 has 0x80000**. That is the same mask the
   client's own request producer tests. Heal 10176 evidently lacks it, so the server-34
   0x78 order for the player was discarded before creation. For skills without that
   flag, the client keeps its **own predicted order (kind 0x53)**.
3. **Predicted-order replacement:** a packed **0x7a** order arriving for slot 1 (outer
   flag bit 5), while slot 0 holds a kind-0x53 order whose action lacks flag 0x8000,
   cancels and replaces that predicted order.

**0x7a layout** (factory entry `0x14128145f`, 0x68-byte object, vtable `0x142179a70`):
exactly **11 bytes**:
- `[0x7a]`
- `[order flags]`
- `[u32 agent]`, resolved through `0x141024060`
- `[u32]`, stored at +0x40
- `[byte]`

Its gameplay role isn't established.

**Consequence:** for the player's own predicted skills, the server answer is not a 0x78
start. It is most likely the order notifications Codex traced (server 84 opcodes
0x44/0x45/0x47, keyed by behavior value), which Codex found require an existing kind-0x78
order. A 0x78 start for the player only applies to 0x80000-flag skills. For NPC (not
controlled) casters, 0x78 via a bit-12 carrier such as server 34 remains the candidate
for making enemies cast. Untested.

### NPC casts via server 34 (runs 127–128)

`--mission-npc-cast SKILL` (with `--mission-hostile-npc`): on each player SkillBegin the
server sends server 34 to the hostile NPC (agent 3). The command flags are 0, with its
own value counter starting at 0x40000000, and the packed 0x78 order is
`{flags 0, SKILL, slot 23, 0, agent 3, 0}`.

- **Run 127 (Word 0):** the client asserts **`AgBase.cpp:822 "command->m_durationMs"`**.
  The check (`0x141032c0a`, `cmp [command+0x70], 0`) reads command +0x70, which is
  **server 34's trailing Word**. So that Word is the command duration in ms, and a
  non-controlled agent's command must have one. The 0x78 order itself was accepted far
  enough to be executed.
- **Run 128 (Word 1000):** no assert and no disconnect; the command is accepted. A cast
  animation isn't clearly visible, because the NPC stands behind the player from the
  camera. That needs a better camera placement, or a hostile skill with a visible effect.

The skill used (10176, the Mesmer heal) is a placeholder. NPC skill content is server data.

### Run 129: targeting works; NPC models don't render

With the minotaur placed off-axis (`-3000,-25300,-3742`):
- **Tab targets it.** The client shows the full target frame: `Branded Minotaur`, level 1,
  a 100% health bar, description `Throws Foes` (from content), and a red target marker.
  Even so, the heal's SkillBegin still carried target 0.
- **No model renders**, for either the minotaur or the boar in runs 122–124: only the
  nameplate and marker. **Missing model files are ruled out:** model 19830 (species
  1544, 173,976 bytes) and 26012 (species 8013, 403,476 bytes) are present in both the
  frozen and the working Gw2.dat. Remaining candidates (*untested*):
  - dependent assets (textures, skeleton);
  - the NPC's **zeroed 183 coreStats appearance** acting as an appearance override (the
    model loader `0x141166a80` considers description/appearance overrides before the
    direct species model; `docs/re/209026-npc-character-foundation.md`);
  - a view or reveal prerequisite.

  The coreStats appearance is the first to test, for example by sending 182/183 only
  where required, or by tracing the override test.
- **Run 130 (runtime trace, run-local runner copy):** model-loader points fired once, at
  06:36:13Z, just after `STATE_READY_WAIT`, i.e. with the AgentsLoaded batch that carries
  the NPC spawn:
  - route entry `0x141169ff0`;
  - loader `0x141166aef`;
  - `0x1411098f0` = 0;
  - default branch `0x141166c0b`;
  - **direct species branch `0x141166db4`**.

  So the NPC most likely selects its species model directly, and the appearance-override
  theory is unlikely. The trace prints only first hits, so the attribution is by timing.
  Invisibility therefore lies downstream of model selection: the asynchronous model load,
  skeleton or animation, or LOD/culling. That is untraced.
- With no model, an NPC cast can't be judged visually. Run 129 again showed no assert
  for the server-34 cast with Word 1000.

### Run 131: the NPC passes view creation, model selection, scene entry and REVEALED

Per-stage probes on object pointers, from a run-local runner copy (each distinct
pointer prints, so the two agents separate):

| Stage | Player (13:31:30, population) | NPC (13:31:45, AgentsLoaded batch) |
| --- | --- | --- |
| View factory `0x14110f150` (rdx = character) | hit 1 | **hit 2** |
| Species model route `0x141169ff0` | not hit (appearance composite path) | **hit 1** |
| Collision scene entry `0x1410c1710` | hit 1 | **hit 2** |
| REVEALED set `0x141032ab0` | hit 1 | **hit 2** |
| REVEALED clear `0x141032ad0` | — | — |

So the NPC gets a view, selects its species model, enters the collision scene and
becomes REVEALED, and it is never concealed. **The drop is after model selection:**
completion of the species model's asynchronous load, or render/LOD/culling of the
loaded model. `0x140cf7000` turned out to be a generic loader helper (thousands of hits),
so it can't isolate the NPC. The next probe should sit on the species branch's own
completion path, from `0x141166db4` through `0x14130c150` to the model component stored
at view +0x820 (`docs/re/209026-npc-character-foundation.md`).

### Run 132: the NPC's species model loads (non-null handle)

Species branch probes (`0x141166df0`, `0x141166eea`, `0x141166eef`) at the NPC spawn
(13:36:19Z):
- species +0xf0 = `0x29d08c10`, non-null and not the empty-name constant `0x1419244e8`;
- it is passed as the filename;
- species +0x120 = 0;
- the loader returns handle **`0x68bcc7a0`**.

A second handle (`0x65f71a70`) follows at 13:36:57Z, near the Tab/skill input. So the
model file resolves and a model object exists. **The invisibility is in the view/render
layer, after the model object is created.** Server 37's eight parameters aren't a scale
(`docs/re/209026-agent-dimensions.md`), so their zeros are unlikely to be the cause.

Remaining candidates:
- view visibility state: character flags from 377's Word flags, never sent, or view
  hide/alpha;
- the model component attach to view +0x820 not completing;
- species +0x120 = 0, purpose unknown.

The next probe is the model handle's attach to the view, and a check of the view's
visibility or alpha fields against the player's view.

### Runs 133–134: view-object diff (player vs NPC)

**Method:** a run-local runner copy captures each ordinary character view as its
constructor installs vtable `0x142149138` (`0x1411589c5`, `rbx`). It dumps each view's
0xc00 bytes 20 s after the NPC's view appears.

- **Both views have a model component at +0x820**, similar fill density, and plausible
  world-position floats near +0x830: about −3200 for the player, about −3020 for the NPC
  near its spawn.
- The player's view also holds a count of 2 at +0x3a0 followed by three
  player-position triples, and nonzero +0x0f4..+0x104 (1, 0x40, 0.25, −1). The NPC's
  are zero. Those fields look player- or control-specific (*inferred*).
- **Run 134** added a server **86** position update for the NPC. The NPC's +0x3a0
  stayed 0 and it stayed invisible, so missing position samples aren't the cause. The
  86 frame was reverted.

**Conclusion so far:** the NPC has a view, a valid model handle, a model component,
REVEALED and a position. The remaining cause is inside model-component rendering
(pose/animation, material/skin, LOD). That's the next trace.

### Likely root cause: the creature models reference files absent from this Gw2.dat

A file-reference scan of the two NPC models uses the encoded `(low, high, 0)` tuples,
id = high×0xff00 + low + 0xff00ff01. The present references are what they should be:
- model **19830** (Branded Minotaur) references textures (ATEX 314887, 2331955, 65282)
  and two more MODL files (19818; 19819, 2.3 MB, likely skeleton or animation), all
  present;
- it also repeatedly references **2257612** and **2492707**, next to its texture
  references;
- model **26012** (Juvenile Boar) references the same **2257612** and **2492707**.

**Both files are missing from the frozen `build-209026/Gw2.dat` and the working copy**
(`gw2-dat-cat` returns nothing). A retail client streams missing files from ArenaNet's
asset CDN. Here:
- no manifest names them;
- the sandbox CDN logged no request (`cdn.log` has only `/latest64/101`);
- offline-only rules forbid fetching them.

So these creatures probably can't render with this archive. That is consistent with
every other stage (view, model handle, model component, REVEALED, position) succeeding.
Their type isn't known, since they're absent; material or shader is *inferred*.

**Workaround in progress:** scan species models for ones whose references are all
present locally, then spawn one of those as the test NPC
(`~/re/gw2/notes/209026-mesmer-re-handoff-20261105/model-refs.*`).

