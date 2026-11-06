# Build 209026: cancellation requests and active-order correlation

Static REA, 2026-11-06, against frozen executable SHA-256
`99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`.
This extends the [predicted-order lifecycle](209026-predicted-skill-order-lifecycle.md)
and [typed78 completion/release](209026-typed78-completion-release.md) notes.
No client was launched, no archive or container was accessed, no gameplay
message was sent, and no cancellation policy or codec was implemented.

The demonstrated queued-order path sends **client23, packed opcode0x00**, in
a **new behavior container**. It does not copy the active cast's correlation.
The client-side cancellation callback for an existing order instead requires
the original behavior value and matching packed order kind. These are distinct
operations; neither server84 result0x44 nor finish0x45 can be renamed a cancel
request on this evidence.

## Exact controlled request producer

Order-component table `0x142179b90` slot **+0x90** points to complete producer
**0x141281d90..0x141281e2a**. RCX is the component. It asserts Dword+0x40
`m_isControlled`, calls `0x141023f10` to initialize a new container, allocates
exactly three packed bytes, and writes:

| Offset | Width | Proven input |
| --- | --- | --- |
| 0 | Byte | Literal0x00. |
| 1 | Byte | EDX normalized to0/1. |
| 2 | Byte | Low byte of R8D, preserved without a named enum. |

The producer resolves the bound agent through component−0x4a8 and virtual0,
then passes the container to virtual+0xa8 with **outer flags0**. That existing
[client23 carrier](209026-agent-behavior-directions.md) supplies the agent and
tick separately. There is no SkillDef key, slot byte, old behavior value or
explicit old-order pointer in this three-byte payload.

Complete initializer **0x141023f10..0x141023f55** increments the context's
counter and applies its context-type high bit. Therefore the cancellation
request's outer behavior value must not be treated as the old cast's value,
nor reconstructed by subtracting one from another request.

The older [producer inventory](209026-npc-talk-loop.md) labeled0x00
`Abort/orderTrack/reason` by size and ordering. The new caller below supports
an order-track relationship for byte1. It does **not** recover the byte2 enum
or independently prove every use of this producer is a manual skill cancel.

## Demonstrated queued-order caller

The already established local admission function is
**0x14127ecf0..0x14127ef7c**, using component+0x38 as `m_queued` and active
slots+0x28/+0x30. Its earlier owner, compatibility and queued-order gates must
pass. On the branch where queued flags+0x14 have **0x10 clear**, the tail at
**0x14127ee63** additionally requires **0x20 clear and0x40 set**.

It calls this component's virtual+0x50 with EDX1. Exact leaf
**0x141281c40..0x141281c68** selects active+0x30 for a nonzero argument and
returns1 only if that order exists and its flags+0x14 have **bit0 set**.
When true, **0x14127ee8b** calls virtual+0x90 with **EDX1, R8D0**, producing
the exact request **`[00 01 00]`** above.

The function then selects the destination active slot from queued-order
flags bit5, aborts/releases any old order in that destination slot, installs
the queued order, and invokes its virtual+0x18. The request is consequently
part of a proved queued-order replacement path. This is not proof of the
physical cancel key, movement interrupt, Escape key, or the particular route
used while casting10176. The local order and its state still matter.

## Cancellation of an existing client order

Bound-character virtual+0x80 reaches the complete order-component body
**0x14127ebe0..0x14127ece0**; its own diagnostic is
`AbortOrder behavior %S (%d)`. It requires at least two packed bytes, selects
active slot `(outer_flags >> 5) & 1`, and calls matcher **0x1412874a0**.
The previously complete matcher capture in the predicted-order note proves
behavior equality first, followed by the assertion that the packed kind
equals the existing order kind. The new capture independently rechecks the
behavior comparison; its `.pdata` fragment ends before the kind assertion.

Only after that match does cancellation clear the active slot, invoke the
order's virtual+0, clear order flag0x100, invoke virtual+0x20 with argument1,
and notify the character through `0x1411dae10` with the selected track.
For concrete typed78, virtual+0 is **0x1412851c0**, whose callback and
conditional local0x79 action are covered by the completion/release note.
This is an abort/release path, unlike ordinary duration expiry's possibly
pending completion. It is also distinct from replacement on a mismatching
incoming start.

Component virtual+0x88, **0x14127eb50..0x14127ebd7**, is a separate local
operation: it scans the two active slots for the first kind0x78, conditionally
retains a track0 match when its EDX argument is nonzero, otherwise clears the
slot and calls abort/release/track notification. It emits no request in this
body. It must not be confused with virtual+0x90, the producer above, or the
different bound-character interface's +0x88 start callback.

## Other request and limits for a server implementation

Component virtual+0x208 is **0x141284860..0x1412848e4**. This also requires
`m_isControlled` and creates a new container, but sends exactly five bytes
**`[0x28, EDX little-endian u32]`**, again with outer flags0. No typed caller
in this bounded slice establishes that Dword's meaning or binds0x28 to
manual cancellation. Adjacent0x29/0x2a producers do not supply that proof.

The supported server boundary is therefore:

- Recognize the exact0x00 request shape separately from0x23 SkillBegin and
  server84 updates; preserve its raw fields and fresh outer correlation.
- Retain the active cast's original behavior, packed kind and track as
  independent state. A fresh0x00 envelope is not an acknowledgement of0x78.
- Do not respond by echoing0x00 as the active cast, synthesizing0x53, or
  substituting0x44/0x45 for an unproved interrupt policy. The request alone
  does not establish a successful interruption, heal cancellation, recharge
  consequence, retail timing, or the server message that ends the command.

The smallest remaining source dependency is a **typed caller of component
virtual+0x90 or+0x208 from the manual cancel/interrupt input path**, including
its argument construction. No whole-protocol or global indirect-call census
was attempted. Exact-address xrefs found the concrete table entries; local
indirect-call checks covered only0x14127eaf0..0x141284f80 and
0x1412851c0..0x141287800. Their negative results are not global absence.

The existing `AgentBehaviorFrame209026` already preserves these client23
bytes transactionally. If a separate research payload codec is later useful,
the presently supported scope is only exact three-byte opcode0x00 with two
raw bytes, documenting the producer's Boolean normalization without claiming
an enum or server policy. No such codec is added by this source slice.

## Evidence

Private complete capture bundle:
`/home/blizz/re/gw2/notes/209026-skill-cancel-request-correlation-20261106.json`,
SHA-256 `f2627d2758c9f22f9ce82448452db18acd1137aa9e7e21c6aacfb3f7b1eb3a7d`.
All15 captures exited0, were untruncated, and verified owned-process cleanup.
Early fragments, unrelated-offset candidates and bounded negatives are
retained with the successful complete bodies, not promoted to stronger claims.

| Evidence ID | Scope |
| --- | --- |
| `ev_73aad2a95c3fda8195eb1ef762059951c9061316c61b64a71db267bff6a525d4` | Complete0x00/0x28 controlled producers and named controlled-state assertion. |
| `ev_01a8bb287504cc4fd8c63ce6ef28433f1c53c0105382b9b40a7e866b2f61b4cd` | Exact component table slots and bounded named-anchor lookup. |
| `ev_7d21603fa2149e826e4ea86980febe20836bd6643b288c4bc9dd1cf6a111d378` | Complete incoming abort body, queue admission prefix and complete +0x50 leaf. Bytes beyond that leaf are not used. |
| `ev_a6557001ad8c1bf1b28a362c2aaa7dddb510b7ecc64b5ebdc1058716173f3931` | Queue tail including exact `[00 01 00]` producer call and slot replacement. |
| `ev_dde6c11ad7f872cf3b6f9a4cb7119dd51f841a11b9ba321afd7898eb36b1c7c6` | Complete distinct local typed78 abort entry +0x88 and queue tail. |
| `ev_3f43276eefa8809d7316507247bff2e344e8e30a082d23931d7857275d395fd0` | Complete new-container initializer, matcher comparison prefix and queue failure return. |

## Manual-input follow-up: typed getter, unresolved caller

A subsequent bounded frozen-PE slice did **not** close a manual-input caller
for either request producer. It added the exact character-side access path:
primary table **0x142160be0 +0x90 and+0x98** both point to
**0x1411bacf0..0x1411bacf7**, whose complete body returns **character+0x4b0**.
This is the order component, not the producer itself; the component's own
+0x90 has a different meaning.

Exact-address reference checking for this getter found four aligned table
references and no validated direct E8/E9/REX.W-LEA references. This says
nothing about indirect callers or other instruction encodings. A global
indirect-call census was deliberately not used to assign an interface from
a coincident virtual offset.

Two named leads do not close the manual-input link:

- `COrder::Cancel` resolves to complete **0x14127fcc0..0x14127fd29**. It
  takes the queued object from component+0x38, clears that field, logs,
  clears the object's flag0x100 and calls release virtual+0x20 with1.
  The body constructs no behavior request and calls no typed abort callback.
  Its name therefore does not identify a player input event.
- `AbortSkillRequestsOnDaze` has an exact reference at **0x14127f313**,
  inside the already known incoming order receiver. A named local gate is
  not proof of an input registration or the arguments to either producer.

The smallest next observation, if runtime work is separately authorized,
is a finite passive sample at **0x141281d90** and **0x141284860** during an
explicitly identified cancel input. It must first bind RCX to the current
controlled character's component/table, then retain the actual caller-site
identity and scalar EDX/R8D arguments without raw pointer output. That would
provide a concrete caller to trace statically. It would not by itself prove
server acceptance, select the interrupted cast, or establish byte2's enum.
No observer was built or launched for this follow-up.

There is **no newly proved codec or server-handler requirement**. The existing
raw client23 envelope remains sufficient, and the fresh request/original
active-order distinction above is unchanged. Manual cancellation, movement
interruption and the unproved0x28 Dword meaning remain open.

Private ten-capture follow-up bundle:
`/home/blizz/re/gw2/notes/209026-manual-cancel-input-boundary-20261106.json`,
SHA-256 `8cf98ef179619fe41e5f88a35f6dc2f0ee7287f4de403b1166c61962af38da82`.
Every capture exited0 without tool truncation. The named-string locator was
explicitly output-capped and is not a proof that other names do not exist.

| Evidence ID | Follow-up scope |
| --- | --- |
| `ev_c74877d7bae45113d7f660440070ec2523320d2963e3f20590d2885fdf684758` | Exact primary table+0x90/+0x98 getter mapping. |
| `ev_4e7a991e3380ca486a4135a96dcc89e8c73bf6a9d29a1939221ff0bd28046f5f` | Complete getter and queued-object `COrder::Cancel` bodies. |
| `ev_aba8135dac2c99ad0079320d59282a3f9e3f0689a0b42db6887a223fe16b6c81` | Getter exact-address reference coverage and bounded negative. |
| `ev_e41213c8cb3abef85866cff12f85cee6d54f76627c374d09347433bccb0a1f4e` | Validated `COrder::Cancel` literal reference. |
| `ev_e6d1b43cc766e5696acd30732c392d1ae4d2cc2facec989300c659ba5c60b647` | Validated daze-gate literal reference in the existing receiver. |
