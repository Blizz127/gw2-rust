# Build 209026: character setup frame subset

This groups five previously traced protocol 0/group 0 messages needed to
express character setup without depending on a private XML schema at
runtime. It adds no inferred wire fields or default gameplay values.

| ID | Fields in wire order | Source trace |
| --- | --- | --- |
| 151 | Dword character index, Dword attitude | [Core-stat messages](209026-character-core-stats.md) |
| 170 | Dword character index | [Player population/finalization](209026-character-population.md) |
| 282 | Dword character index | [Inventory and finalization](209026-character-finalization-dependencies.md) |
| 443 | Dword player index, Dword character index | [Player identity and ownership](209026-agent-creation.md#player-identity-and-ownership) |
| 444 | Dword A, Dword B, Dword character index | [Controlled-character assignment](209026-character-control.md) |

All Dwords use the established unsigned varint encoding. The exact roles
of 444's A/B fields remain unnamed: equality selects the observed
controlled-character path, while unequal fields select a different path.
They must remain explicit, rather than being silently filled or equated by
the codec. The other messages also have stateful prerequisites; their
representation does not itself enforce ordering or validate a bootstrap.

`gw2_proto::character_setup::CharacterSetupFrame209026` now represents
these five layouts with explicit fields and transactional decoding. It
preserves raw u32 values, including unequal A/B and values outside known
gameplay constraints, and never emits messages automatically.

Three focused tests and `cargo build -p gw2-proto --lib` pass. Hand-authored
hex fixtures verify every layout, field ordering, unequal/equal A/B,
distinct varint widths, zero and u32 maxima. Additional checks verify
append prefixes, exact concatenated boundaries, every truncated fixture,
neighboring IDs and overflowing/overlong varints at every field position.
No client replay occurred.
