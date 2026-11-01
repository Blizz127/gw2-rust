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
public value and generator are derived from this use; run 48 confirms the DH
arithmetic against an independently computed shared value, as described below.

Importer `0x14156fbd0` copies ascending little-endian u32 limbs and removes zero
limbs at the high end. Exporter `0x1415707e0` returns the limb pointer and count
multiplied by four. Thus this path uses **little-endian** integer bytes.

Constructor `0x140fefb00` clears a 20-byte buffer at object +0x118, then copies
at most 20 bytes from the first operation's result. It sends the second result
as `u8 0, u8 total_length, little-endian integer bytes`, where integer length
is a multiple of four and at most 64. Run 47 directly confirms type 0 and length
66, with 82 total captured bytes.
Its preceding 16-byte preamble contains build 209026 at offset +4; other preamble
fields remain opaque. The probe observer checks that build field and uses the
observed fixed preamble length.

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
existing `rc4_key` structure; run 50 independently verifies the resulting
decryption against the client plaintext. The
existing helper's arbitrary-length folding is not confirmed by this path.

## Local experiment design

Use an ephemeral private server exponent and substitute only its public value
in the offline client's process memory, guarded by the original full parameter
block. Retain the frozen prime/generator. Compare the client's computed shared
prefix and initial RC4 state against the local implementation without printing
keys. Answer the type-1 reply and capture subsequent decrypted client bytes into
private files. No instructions or validation outcomes need changing. Preserve
TLS synthetic fixtures and isolate the binary listener with `-authsrv`.

## Offline runtime: run 48

With `-authsrv 127.1.0.2`, the binary connection reaches a separate loopback
listener while portal TLS remains on 127.1.0.1. A GDB probe at `0x140ff4480`
checks the full original 136-byte block, then writes only our 64-byte public
value at +72 and verifies it. The prime, generator and version are unchanged.
At `0x140fefc4c`, the client public value and its stored shared prefix are read;
Python's `pow(client_public, own_private, p)` produces exactly the same low
20 bytes. This confirms the DH roles and byte order for this client.

The Rust observer sends a type-1, length-22 reply with a fresh random block.
The client reaches `0x140fee9e6` after initializing its cipher and sends 57
bytes. A private decoded capture is created with mode 0600, but a search for
the synthetic token in standard GUID encodings does **not** match. Decryption
was therefore initially unverified.

## Exact plaintext verification: runs 49–50

Run 49 stayed at the login form, with no StartTls or game exchange; its plaintext
comparison was unavailable. Screenshots showed entered synthetic credentials
without a submitted login. Run 50 added UI input settling and armed the RC4
input breakpoint only after game cipher initialization, avoiding unrelated early
calls. These changes address probe/input timing; no client behavior was patched.

The probe at `0x140ff4dc0` selects the outgoing cipher context (connection object
+0x234) and captures its input buffer into a private file. **All 57 decoded bytes
match all 57 bytes in the client buffer exactly.** The DH prefix also matches
again. The synthetic Token GUID is absent from that first plaintext in both
standard UUID byte representations. Its meaning awaits message-schema analysis;
absence of a token here is not evidence of a cipher failure.

Run 50 verifies the client-to-server cipher and key derivation for this build.
Server-to-client compressed frames, message replies and character selection
remain unverified.

Network isolation, container cleanup, frozen/working executable hashes and
archive count 1,124,232 were verified. REA owned-process cleanup is verified;
its unrelated-host-process survey remains incomplete. No character-selection
or server-to-client game-message response has been demonstrated.

Implementation: `gw2-game-observe` accepts an explicit local parameter blob and
private exponent, requires a loopback listener, validates key-record lengths
and public values, sends the 22-byte response, and captures bounded decrypted
traffic in exclusive mode-0600 files. `session_secret` now returns the exact
20-byte XOR input; four focused DH tests, the observer build and crate clippy
pass. The full workspace suite was not rerun.

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

Additional evidence:

- `ev_23c89113a6d92ae0e65ff3a77c96eb6a229c7497ae47136592d271b6fd7eb27a`:
  run-47 raw metadata, processed through REA; no raw values printed.
- `ev_759929af7ed260bd52dc284b441df963e815866f8f8fc4cd91cc1dec95389927`:
  run 48, private export `~/re/gw2/notes/209026-game-handshake-run48.json`.
- `ev_959382d03a203ebeb6e67a2268edafb305203982b0f2777483841a017f1c07df`:
  RC4 input-buffer signature and state update, supporting the next probe.

- `ev_a5c2b3ab2b94e2eb5d348cec61010ed8dd04a0afeb3d43db597dff2c95cfd245`:
  run 49, no login exchange; no plaintext conclusion.
- `ev_2106636f8c4dcc2567f8f53688d609e304e64a1cff397a4e6ce417c6701e6550`:
  run 50, exact 57-byte plaintext match; private export
  `~/re/gw2/notes/209026-game-handshake-run50.json`.
