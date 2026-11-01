# Build 209026: payload-free player startup frame

Static REA reconfirmation, 2026-11-01. Server registration pair
`0x14216fe30` contains descriptor `0x1425c81a0` and handler
`0x141255010`. The complete terminated descriptor has only Header 431:
there are no payload fields. Its decoded wire frame is exactly the two
bytes **`af 01`**. This is framing evidence, not proof that sending it
at any particular point is safe for the client's game state.

The [initial handler trace](209026-reward-track-messages.md) follows its
call to context startup `0x1411b7700`. Later findings establish direct
[wardrobe and component prerequisites](209026-wardrobe-startup.md),
[player-tail work](209026-player-startup-tail.md), and
[context subscribers](209026-startup-subscribers.md). The
[content-direction listener](209026-content-direction-startup.md)
additionally requires a reported quest manager, supplied by server 538
for an existing player, before this event reaches that listener.
Listener coverage and the complete bootstrap sequence remain unresolved.

The typed `PlayerStartupFrame209026` in
`crates/gw2-proto/src/player_startup.rs` appends/consumes only this two-byte
frame, leaving following stream bytes intact and leaving input unchanged
on truncation or a different ID. It does not change the observer, send
packets, or validate component state. The client stays stopped at the
existing agreement hold.

Validation: `cargo test -p gw2-proto --lib player_startup_` passed both
focused tests, and `cargo build -p gw2-proto --lib` passed. Tests cover
literal bytes, prefix preservation, mixed 538/431/538 stream boundaries,
truncation and wrong-ID transactional rejection. No client startup smoke
was performed.

Private bundle: `~/re/gw2/notes/209026-player-startup-frame-1.json`
(1,922 records). Executable SHA-256:
`99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`.

| REA evidence | Observation |
| --- | --- |
| `ev_47f1da4e0cbdc65b872500eb60818b19fe924ea11aafed76819abc34ac42e44d` | Complete header-only descriptor through its terminator. |
| `ev_c52586ca4342f24f3f6662b1182cd31074afb9e0bf9b183e7ed5087637ee4742` | Direct registration descriptor/handler pair. |
