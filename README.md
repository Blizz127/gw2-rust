# gw2-rust

A Guild Wars 2 server emulator in Rust for the private, offline Banshee Realm.
Work in progress; nothing runs yet.

- Client reverse engineering goes through REA: see [docs/REA.md](docs/REA.md).
- Runs offline and firewalled only. Never use a modified client on a real
  ArenaNet account.
- Reference: [valaphee/ascalon](https://github.com/valaphee/ascalon) (MIT).
  GPL-licensed projects are for reading only; do not copy their code.

Licensed under the MIT License (see [LICENSE](LICENSE)).

## Layout

| Crate | Purpose |
| --- | --- |
| `gw2-net` | Transport: RC4 key derivation, LZ4 server frames |
| `gw2-proto` | Message primitive encodings (varints, strings, GUIDs) |
| `gw2-dat` | `Gw2.dat` archive reader and decompressor |
| `gw2-api` | Importer for the public GW2 web API (cached under `data/`, not committed) |

Status: [docs/STATUS.md](docs/STATUS.md). Client request: [docs/CLIENT-NEEDED.md](docs/CLIENT-NEEDED.md).
