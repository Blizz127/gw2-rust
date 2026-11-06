# Build 209026: moving existing backpack items

The frozen client supplies a request and reply for moving an existing item
between inventory slots. The server previously framed client85 but ignored it.
The bounded implementation supports whole-stack moves into empty slots and
swaps with a different item definition in the existing starter backpack.

## Observed client contract

Client85 descriptor `0x1425c6920` has these fields in wire order:

| Field | Encoding | Meaning |
| --- | --- | --- |
| 0 | Dword | Source item instance ID |
| 1 | Byte | Movement quantity |
| 2 | Byte | Destination location type |
| 3 | Dword | Destination slot index |

`0x1411f0a50..0x1411f0c18` checks the source item, its membership in the
inventory, source/destination location types and quantity. A zero quantity
argument to this **internal function** is replaced by the stack's actual
quantity, or one for a non-stackable item. This does not establish zero as a
wire request for "move all." It tail-calls sender `0x141260ac0..0x141260b67`,
which writes ID85 and the fields above. The sender asserts quantity fits in
one byte and transmits the 12-byte native message structure. Dword wire
compression follows the existing [primitive contract](209026-primitive-fields.md);
native structure offsets are not wire byte offsets.

Registration pair `0x14216f650` binds server289 descriptor `0x1425e3570`
to handler `0x141250ab0`. Its wire order is:

| Field | Encoding | Meaning |
| --- | --- | --- |
| 0 | Dword | Character index |
| 1 | Dword | Existing source item instance ID |
| 2 | Byte | Destination location type |
| 3 | Dword | Destination slot index |

Handler `0x141250ab0..0x141250b84` resolves the character's inventory and
the existing item instance, then calls `0x1411f1f30`. For an item already in
inventory, that method obtains its current slot and the destination slot,
swaps their item pointers at `0x1411f2040..0x1411f204d`, updates both items'
locations when the destination was occupied, and notifies listeners. One
server289 therefore handles both a move into an empty slot and an occupied
slot swap. It neither creates a replacement instance nor changes stack count.

The slot resolver `0x1411efc70` subtracts two from the location type. Its
table at `0x1411efeec` maps location3 to `0x1411efcd3`, which checks the
index against `m_inventorySlots.Count()` and returns that slot. Location3
is inventory; equipment and account/shared/overflow storage use other paths.

These are static observations of client behavior, not a retail server trace.
The client consumer does not establish authoritative merge, split, or
rejection policy. No new native-client runtime was launched for this change.

## Server scope and state

The observer accepts the existing connection's own backpack instance only
after startup and before its handoff/teardown, with its starter bag and tracked
loot enabled. Quantity must
equal the complete stack, destination type must be3, and the destination
must be among the20 usable starter-backpack slots. The larger array sized
by server305 is not additional usable bag capacity.

An accepted move changes only slot positions. Item instance, definition,
quantity, creation type, wallet, rewards and the monotonic loot-instance
allocator remain intact. Repeating a request after the item reached its
destination has no further effect. Invalid or unsupported requests change
nothing. Same-definition occupied slots, partial stacks, equipment and other
storage remain unsupported; their semantics are not guessed from the swap
consumer.

Exact moved slot positions travel through the existing process-local carry
on supported matching transfers/rejoins, including loot-only profiles. This
adds no durable storage or schema migration. Progress still has the existing
server-restart and disconnect limitations.

## Focused verification

An encoded85 request reproduced the previous bug: it became `Seen::Other(85)`
and produced no reply. The same regression passes with the handler installed.
Four bag-codec tests and six mission move tests pass. They cover literal wire
bytes and the extracted schemas, fragmented/coalesced input, transactional
decode errors, empty-slot moves, one-frame swaps, idempotent replay, rejected
requests, allocation of the next loot instance, native-default quest32138
identity, reward capacity, and a GUID/map-matching carry consumed only once.
The rejoin test also rejects further moves from the retired source connection.

Commands in the isolated464-based worktree:

```text
cargo test -p gw2-proto --lib bags::tests::
cargo test -p gw2-net --bin gw2-game-observe mission::tests::backpack_move -- --nocapture
cargo build --offline --release --target x86_64-unknown-linux-musl -j 4 -p gw2-net --bin gw2-game-observe
```

The static musl build passes. Both auth/world `--check-config` startup checks
pass against the unchanged464 saved-mode profile, with31 story arguments and
an owned empty temporary store; no listener is started and the temporary root
is removed. Private logs and build-input hashes are in
`/home/blizz/re/gw2/notes/backpack-move-20261106-checks/`. These are offline
protocol/state checks, not a native UI drag observation or a deployment receipt.
The startup receipt is `backpack-move-257b98b-preflight-20261106.json` in the
same private notes root. Implementation commit: `257b98b`.

## Published increment

The sole publisher integrated257b98b and its source note on the quest168/169
base as `c2ae0e8`. Kit `/home/blizz/gw2-rust-c2ae0e8-backpack` went live at
2026-11-06 **20:11:16Z**. Root independently read it back at
**20:14:38.978691Z**: all current/rollback manifest hashes matched, the four
owned listeners were healthy, saved mode remained enabled, the85 story
arguments matched the pinned profile, and CDN2130706950 readback matched.
The record metadata check found the existing two private records; it did not
read their contents. Rollback `/home/blizz/gw2-rust-e1348e3-chain` is retained.

The1970192-byte deployed game binary SHA256 is
`e6fdd1226de8d577b3affa68361415957639523f0a797fd2180de315a9908149`.
Only the game binary and BUILD differ from that rollback. Effective auth and
character-store source match464; the held53a8989 behavior remains reverted.
The publisher reports combined85-argument startup and focused build checks;
root's live readback does not convert the unobserved native drag into a pass.

Private verification: `c2ae0e8-backpack-release-verification-20261106.json`;
full REA capture: `209026-c2ae0e8-backpack-release-readback-20261106.json`,
`ev_19be849ff0e7adc99ee4a009474512f6bee6a69423ef9239498d1d602496f354`.
The capture exited0, was untruncated and verified owned-process cleanup.
Normal quest placements and saved-character story traversal remain unproved;
this release does not complete the full retail-fidelity goal.

## Private evidence

Executable SHA256:
`99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`.
All captures are static REA Rizin observations, exit0, output untruncated,
with unchanged before/after executable hashes. The string-inventory size
warnings do not affect the selected instruction/data reads; no global
string-search completeness is claimed.

Private files under `/home/blizz/re/gw2/notes/`:

| File | REA evidence | Coverage |
| --- | --- | --- |
| `209026-backpack-move-source1-20261106.json` | `ev_0fe8a3f4f0db81014067da80cb0d2fc880cd050ddab4d4f72448d91ba98af0be` | Complete request wrapper and289 handler; complete existing-item swap branch. |
| `209026-backpack-move-source2-20261106.json` | `ev_222fa1bc704de4ca9dc98bea74c1ee5b853ae1eff4037cf3e8de256b1a27307c` | Sender prefix; inventory-slot resolver. |
| `209026-backpack-move-source3-20261106.json` | `ev_0468b07de5ce0b6551768962e71bea09e42b56c9d0f1fcb8079021a9a753b25f` | Sender continuation through return; location jump tables. |
| `209026-backpack-move-source5-20261106.json` | `ev_c1039f813aba9475af3b2a9459d72dec1b3505aef6bfb330aea10a41e91a7e60` | Complete85 descriptor. |
| `209026-backpack-move-source6-20261106.json` | `ev_5c94cdc60dfe19439442848ae4e6dced172a92223c5e1cf46494550a8cecc23b` | Exact289 registration pair. |
| `209026-backpack-move-source7-20261106.json` | `ev_fd74b92e3c161053d1adfd6e87521d9d5a84fe545bdef914dc32906c2cdbb9b9` | Complete289 descriptor. |

Some bounded reads extend into neighboring functions or stop within unrelated
branches. Only the complete relevant ranges identified above support these
claims. Source4 and the unrelated table slice in source5 are retained as
investigation history, not as proof of289's registration.
