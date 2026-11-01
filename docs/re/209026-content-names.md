# Build 209026: encoded content-name lookup

Static REA evidence from the frozen executable, 2026-11-01. This conversion
supports private archive inspection; it is not an account or credential hash.

`0x1412e6650` checks the first base container's flags before a content-name
lookup. With bit 1 set, it calls `0x1412e4df0` to convert the requested
UTF-16 name, then hashes and compares the converted text in the name index.
The inspected frozen base container has this bit set. Type-147 palettes
provide 50 successful reference matches across ten race/sex profiles.

The converter copies the name to a 512-code-unit local buffer and splits
at the **last dot** (`0x1409b8540`). It transforms the prefix and final
component separately and joins them with one dot. Without a dot it transforms
the entire name once. Each component transformation at `0x141481860` is:

1. SHA-256 the UTF-16LE bytes, excluding the terminator (`0x141578e20`).
2. Start a 64-bit accumulator at `0xcbf29ce484222325`. Read the digest as
   eight little-endian Dwords. For each Dword, four times XOR the **entire
   current Dword** into the accumulator, multiply by `0x100000001b3` modulo
   2^64, then shift the Dword right eight bits. There is no low-byte mask.
3. Base64-encode the accumulator's eight little-endian bytes with the
   standard alphabet (`0x140d98060` / `0x140d97d10`). Keep the first five
   characters, using the six-code-unit copy bound to include a terminator.

The result is five characters, or eleven with a namespace prefix and dot.
The mixing step must not be replaced with ordinary byte-wise FNV-1a.
SHA-256 initialization and big-endian digest output were inspected; the
Base64 alphabet and grouping were inspected independently.

The Rust tooling implementation will accept at most 511 UTF-16 code units,
rejecting embedded NUL and longer input. This bounds its supported inputs
without asserting the client's behavior for an overlong or unterminated name.
It performs no content lookup and does not establish name uniqueness.

Private evidence bundle:
`~/re/gw2/notes/209026-appearance-palettes-and-keys-1.json`.

| REA evidence | Observation |
| --- | --- |
| `ev_00036a80445d330b895daeb290910dbdda7ccf48791741e0ac2911ec9c976c02` | Buffer bound, last-dot split and two component conversions. |
| `ev_2ce8e6f769a5494947ba9c8627cfd270460eaf451ce1f8ef468c4ef9c7836414` | Digest mixing, five-character result and last-dot helper. |
| `ev_a5c4856c4428f69398415df6fe7c196e3310f76f32a562e079fe1e35a5007c47` | SHA-256 initialization and Base64 dispatch. |
| `ev_6516c22ed78a8a02ac551ace0bacf2345b4ae6bc401bd15810d4e596e022a662` | Digest byte order, Base64 grouping and alphabet. |
| `ev_69206444a562909c0f6632964f0b9df37670f05a3f3c3fe0f4af2eaee8614f28` | Offline conversion resolves all 50 palette references. |
