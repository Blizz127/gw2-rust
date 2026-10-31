# Reference packfile schema: build 208039 (NOT our build)

`packfiles-208039.xml` is copied unchanged from valaphee/ascalon
(`asset/packfiles.xml`, commit `03fb60b`, MIT, Copyright (c) 2026 Valaphee).

It describes the chunk struct layouts of six packfile types (`mapc`, `MODL`, `txtm`,
`MFST`, `ARMF`, `cntc`): 52 chunks and 372 versions, with field names. A chunk's
on-disk version picks the struct. Ascalon derived the file statically from client
build 208039's embedded schema tables (`research/packfiles.py`).

Packfile chunk versions are explicit and old versions stay readable, so these
layouts likely carry over to our build better than message ids do. That is
**inferred, not confirmed**. Our build's schema gets regenerated through REA and saved
as `packfiles-<build>.xml`.

Layout rules used by `gw2_dat::pfread` (ascalon `asset/src/packfile.rs`):
- Structs are packed, with no alignment padding.
- Pointers are 4 bytes, or 8 when the PF header flag `4` is set. Each one holds a
  signed offset relative to its own slot (0 = null).
- `ArrayPtr` = `u32 count` + pointer. `List` = `ArrayPtr` of pointers.
- `CharPtr` / `WcharPtr` / `FileName` / `FileRef` point to NUL-terminated
  8-bit / UTF-16 strings.
