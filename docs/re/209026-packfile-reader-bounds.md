# Packfile reader bounds during build-209026 content work

Source inspection and synthetic checks, 2026-11-01. These findings concern
the local Rust reader, not behavior inferred from the shipped client.

Before extending archive content inspection, two focused regressions exposed
problems in `pfread::read_chunk`:

- The reader replaced `Limits::max_elements` with the greater of that limit
  and twice the input length. Padding therefore defeated an explicitly
  smaller caller budget. A seven-element synthetic payload still decoded
  with budgets of zero, one or six.
- A 64-bit pointer containing `i64::MAX` at a nonzero slot overflowed signed
  addition before the target bounds check, panicking in a debug build.

The correction honors the supplied element budget and checks the
pointer addition and conversion before reading its target. Larger deliberate
reads can supply a larger budget explicitly. No new format semantics or
client behavior are established by these changes.

The five focused reader tests and `gw2-dat-pf` build pass. A frozen Queensdale
archive smoke decodes 19 chunks, including terrain and typed collision, but
the 13,256,968-byte `octm` chunk now correctly exceeds the default four-million
element limit. This initial complete-file command did not pass. The CLI now
offers `--max-elements N` for an explicit larger budget; its default remains
unchanged.

With an explicit 16-million element budget, the complete Queensdale command
passes all 20 chunk decodes, including `octm`. Four CLI smoke cases reject
missing, malformed, negative and overflowing budget arguments before opening
an archive. The correction changes reader limits only; it does not validate
all schema semantics or placement transforms.

Private REA bundle: `~/re/gw2/notes/209026-packfile-reader-bounds-1.json`.
`ev_36f424d31721f78126e2a5a23d4fdc7642f419b50e9c332275a66899bf7349e3`
records the default-budget rejection;
`ev_5cfd17d4762a952a3d21462e5b483a8417c578b9f57b9a5d89791dff17c203c8`
records the explicit-budget success. No client process was launched.
