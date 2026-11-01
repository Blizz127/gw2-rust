# Build 209026: binary game handshake

Investigation 2026-11-01, following run 47. Frozen executable SHA-256:
`99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`.
All client reads below were made through REA.

## Parameter block and outgoing exchange

A bounded `.rdata` search for `(u32 1, u32 4)` produced 722 candidates. Only
`0x142111220` had at most three zero bytes in the following 128 bytes (it had
none). This heuristic alone was not treated as proof. Getter `0x140ff45b0`
returns that address and writes length 136; its caller passes the pointer/length
to connection constructor `0x140fefb00`, which calls `0x140ff4480`.

That function requires length 136 and version 1, imports 64 bytes at +8 and
64 bytes at +72, and imports the u32 at +4. The two large values and generator
are used in two calls to the same large-integer operation `0x141570b90`, with
a common generated exponent and modulus. Their roles as DH prime, server
public value and generator are derived from this use; a runtime arithmetic
comparison is still pending.

Importer `0x14156fbd0` copies ascending little-endian u32 limbs and removes zero
limbs at the high end. Exporter `0x1415707e0` returns the limb pointer and count
multiplied by four. Thus this path uses **little-endian** integer bytes.

Constructor `0x140fefb00` clears a 20-byte buffer at object +0x118, then copies
at most 20 bytes from the first operation's result. It sends the second result
as `u8 0, u8 total_length, little-endian integer bytes`, where integer length
is a multiple of four and at most 64. Run 47's 66-byte raw chunk is consistent
with this record; a direct byte/header cross-check is pending. The preceding
16 bytes have not yet been mapped.

## Server reply and session key

Receive dispatcher `0x140fefe50` reads a two-byte type/length header and dispatches
types 0–2 through table `0x142110df8`. Type 1 calls `0x140fee7f0`, which requires
connection state 2 and total length 22. It XORs the reply's 20 payload bytes
with the 20 bytes stored at object +0x118, then calls cipher setup `0x140ff5050`
with length 20 and sets state 3. The cipher state is copied for the second
direction, giving independent states with the same initial key.

This corrects the earlier provisional implementation: the input to key derivation
is **the first 20 little-endian shared-value bytes XOR the 20-byte reply block**,
zero-padding a short shared value. It is not the full 64 bytes XOR a repeating
block. Cipher setup performs five SHA-1-style steps and feed-forward, followed
by the 256-byte RC4 key schedule. For exactly 20 input bytes this matches the
existing `rc4_key` structure; a runtime key-state comparison is pending. The
existing helper's arbitrary-length folding is not confirmed by this path.

## Next experiment and limits

Use an ephemeral private server exponent and substitute only its public value
in the offline client's process memory, guarded by the original full parameter
block. Retain the frozen prime/generator. Compare the client's computed shared
prefix and initial RC4 state against the local implementation without printing
keys. Answer the type-1 reply and capture subsequent decrypted client bytes into
private files. No instructions or validation outcomes need changing. Preserve
TLS synthetic fixtures and isolate the binary listener with `-authsrv`.

No game handshake or message schema has yet been validated at runtime.

## Evidence

Private export: `~/re/gw2/notes/209026-game-handshake-static-1.json`.
REA Rizin reads and GNU objdump through REA; no executable writes.

| Evidence ID | Observation |
| --- | --- |
| `ev_f39e9c0cd588aba97e8e6cd8e3cde444eb634dbdfd1b1daf777d5a2fd816e51b` | Header search. |
| `ev_3564fc58ec3ffa0aec4f2c62b95e63be04717ce64299331f37f35c606cc57cb2` | Candidate reads and unique dense block. |
| `ev_b373c15cec244dea18619f5b7ce3136acbd34741c3e305dbb62f79a9dba82824` | Getter references block. |
| `ev_fea2431c78f3567fa36cfce6def7b2953057ebb7043bc7f6b5982b02f67165aa` | Getter callers. |
| `ev_aef232d4517bef0fa51e04689b33cff24e33333a6c855e5c35fb9a216ca232b2` | Connection constructor and outgoing record. |
| `ev_203900b56fe4854c0d98b685b0673ddc915ddc1bf3f2d4dcba9a10ef4a2a7dae` | Parameter imports and integer operations. |
| `ev_a4eff60fc2e33b1ac57911accff4c8f1df92135dc7be2d8dc2ab3fe1299ccd27` | Little-endian integer import. |
| `ev_ae5ee17c0f5505e9197ffffb3ab0c36bbf028eb6be30fa88a191672047df1be0` | Limb export. |
| `ev_9d2f183b0ab644b6266dde002438b117b7de2082c6eeb6aca50cf40bd79cfd6c` | Receive dispatcher. |
| `ev_08ad9996759cd7674d22b53c33f9e78f86de477980b0d20da8c6f07febd91f33` | Dispatch table. |
| `ev_3d0b931fbd02c9e861a5d8581bc6d50be38fa60b6545a28b470612ca62a55450` | Type-1 response, 20-byte XOR, state transition. |
| `ev_d6befa846a4f7e10ca7233c800ad3c28d7254eaee4ef6bff8b6ebb6cb2ca9517` | Key derivation and RC4 schedule. |
