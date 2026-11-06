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

### Quest 170 integration: independently verified `633fff8`

The sole publisher deployed `633fff8d1c321b6d5f60b64e2e95be31373e5dea`
as `/home/blizz/gw2-rust-633fff8-q170` at 2026-11-06 **21:06:38Z**, retaining
`/home/blizz/gw2-rust-c2ae0e8-backpack` as rollback. Independent readback at
**21:18:48.203942Z** passed all current and rollback manifest sums. Exactly
`BUILD`, `bin/gw2-game-observe`, `data/mission-profile.conf` and
`deploy/gw2-server.sh` changed; no manifest rows were added or removed.

| Current artifact | SHA256 |
| --- | --- |
| Game binary, 1,995,432 bytes | `13f1e64e39d7087a449f1a26efaaa790fa1c38034f87988d3c532a7e87d42bb3` |
| `SHA256SUMS` | `bb3ae562d97a466dcbd74f8aef5b1c0ad392fd4ed17409335371f8b8825da7ad` |
| `BUILD` | `a3032dac77ef7be424fcc1feaa65620fd8e830d353e01fff7c0c12d508074ef7` |
| Mission profile | `bc6eb6eb0c384808b55c507239800adfd671d8c3d5e2143f5f2094c519969bf1` |
| Supervisor | `97b0c470cd5898c86d6f2409f53cf5c996615c168394d0d0073d7b0fdba219d4` |
| Unchanged 27-byte sword-chain skills | `7a8f8c7be4e52f646aec90b0edfaf66feceb6a800d52354cf78fad8f9809d7b1` |

The 102 effective story arguments match exactly: the prior 85 are preserved,
with map 400, five NPC entries and eleven hostile entries added. All prior
asset bytes and the eight root-manifest rows are unchanged; served
`209026 0 0 2130706950 508` and the checked leaves match. Four owned listeners
were healthy, with zero humans and zero bots at the readback instant. Saved
mode, the 0700 store, two 0600 record metadata entries and the existing backup
hash passed. Record contents were not read, and names were not emitted. Ether Feast and
sword-chain mode remain enabled; legacy autoattack, echo, sword2 diagnostics,
movement-log override and story seeds remain absent.

Exact source comparison against `c2ae0e8` finds byte-identical `mission.rs`,
`auth.rs`, `saved_population.rs`, `npc_talk.rs` and `population.rs`. The normal
quest 154 Thora/trophy/Ido handlers and saved/auth record handling therefore
have no source delta in those files; this does not establish native execution
equivalence. The separate current-kit auth/world no-listener checks both pass
with 102 story arguments and a removed empty temporary store.

The publisher reports quest 170 steps 1–5 in `laneb20-q170` and step 6 in
`laneb21-q170-ally`, using prepared characters, fixture placements and seeds,
with a binary byte-identical to this release. Those reports do not prove
normal placements, the invented/unprobed Svena starter or handoff from 168/169,
the saved-character route, species 5900 or the Grechen variant. Dialogue is
reconstructed; Armund, Styrr, Eir, Garm and portals are omitted from this
quest 170 reconstruction. Carry remains process-local, items are unimplemented,
and 171 is recorded as next but is not playable.

Private verification: `633fff8-q170-release-verification-20261106.json`,
SHA256 `bb4a311cbe448a4bd4cf714c1132d27e5e831d544a7fb3e4951c3b76e2aedae8`.
Complete REA capture: `209026-633fff8-q170-release-readback-20261106.json`,
SHA256 `93499a12e995d8d6ba429c9590403a1539ec75ba2b78072ee1241be498d51126`,
Evidence `ev_404bccb46ea738dfd2735309790a302f0d0d8697a13c65aa5c7117ece091349a`.
It exited 0 without truncation and verified cleanup of its owned process group.
One unverified process-environment observation returned EACCES; it was not
attributed or touched. The existing undeployed standalone 8192 leaf dependency remains an
asset-completeness limitation. No deployment, client run or store mutation was
performed by this verification.

### Quest 171 integration: independently verified `cb8622a`

The publisher reports `/home/blizz/gw2-rust-cb8622a-q171` live since
2026-11-06 **22:43:06Z**. Independent readback at **22:53:36.573457Z** verified
that selected kit and rollback `/home/blizz/gw2-rust-633fff8-q170`, including
all current and rollback manifest sums. The deployment timestamp itself
remains publisher-reported. Exactly `BUILD`, `bin/gw2-game-observe`,
`data/mission-profile.conf` and `deploy/gw2-server.sh` changed.

| Current artifact | SHA256 |
| --- | --- |
| Game binary, 2,002,128 bytes | `de7cd57486f10b01ddf2e40b0d6b4b522401500882cbd05426b39666edff6cf3` |
| `SHA256SUMS` | `fbaea53014f031472a1caf8b43a2f53f84dd3370cbda42ba94f52698397f001d` |
| Mission profile | `bb82e99ce4c8eccc880409bbe535f811b471c00d1a634332f27e6f97d21aa2f1` |
| Supervisor | `d79f03b5a7b9102c84d6a076faf42ee37a7a18cfb0471aa5e5c2f27eb0479f96` |

All 126 effective story arguments matched: the prior 102 plus map 407,
four NPC entries and nineteen hostile entries. Four owned listeners were
healthy, with zero humans and zero bots. Saved mode remained ON; the 0700
store, two 0600 record metadata entries and unchanged activation-backup hash
passed without reading record contents or emitting names. The 27-byte skills,
Ether Feast, sword chain and normal combat defaults were preserved; story
seeds and diagnostic overrides remained absent. All asset bytes were unchanged;
the served CDN950 root and checked small leaves matched. The launcher
owner-only hold persists and was untouched.

The publisher separately reports prepared, seeded quest171 fixture smokes
in `laneb22`, `laneb27`, `laneb23`, `laneb24` and `laneb26`, with a binary
byte-identical to this release. This readback did not inspect or replay those
runs. Normal placements, the Valda starter on map28, handoff from170 and the
saved-character quest171 path remain unproved; neither native quest171 success
nor normal-path equivalence follows from the deployment checks.

Private files under `/home/blizz/re/gw2/notes/`:
`cb8622a-q171-release-verification-20261106.json`, SHA256
`dd25bac5fa2a4971ced60853d0213e5a19f064a4c06e975d62c40d91727a76bb`;
complete REA `209026-cb8622a-q171-release-readback-20261106.json`, SHA256
`b083cf1d540185ebd5e4d831d38882c41aa75aee593d406665f1256181e97751`,
Evidence `ev_bca100757c3799e0b8eff938fd068e7ee3a157ead02a12d05446b0c534fd6a62`.
The capture exited0 without truncation and verified owned-process cleanup.
Two unverified process-environment observations returned EACCES; the processes
were not attributed or touched. The existing standalone8192 asset-dependency limitation remains.
No client, sandbox, archive, deployment or store mutation occurred.

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
