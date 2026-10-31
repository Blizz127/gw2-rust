# Public protocol notes (second-hand, 2012–2013 era)

**Provenance:** these are public blog notes, summarized by a scout agent on 2026-10-31.
They are not REA observations and not verbatim quotes, and they describe clients from
around build 16247–17368 (2012–2013). Treat every line as a hypothesis to confirm with
REA on our frozen build.

Sources:
- poke152, "Portal connection" (2012-12-21): https://poke152.blogspot.com/2012/12/portal-connection-time-to-dig-into.html
- poke152, "Game connection" (2013-03-08): https://poke152.blogspot.com/2013/03/game-connection.html
- puppt/GW2Emu and nomelx/GW2SEX: their READMEs hold only a title. No source was read.

## Portal / auth (STS)

- The client opens with an HTTP-like STS request, `P /Sts/Connect STS/1.0`, carrying an
  XML body (`ConnType 400`, `ProductName gw2`, `AppIndex 1`, `Program 101`,
  `Build 1002`).
- Then `/Auth/StartTls`, then TLS-like records: content type, version `03 03`, u16 length.
  The record types are 0x14 ack, 0x15 error, 0x16 auth phase and 0x17 data.
- Authentication is SRP-6a (RFC 5054 group, g = 2) with a modified x hash. After an
  encryption switch, separate send/recv keys come from a master key. The key expansion
  was never documented.
- Ports per the blog index: portal 6600, game 6112.
- **Today:** the launcher/Steam/Epic sits in front of this. Whether `-authsrv` still
  exists is **unknown** (REA probe 4 in STATUS.md).

## Game connection

- Diffie–Hellman with a **512-bit prime and g = 4**. The server's public value is
  precomputed and baked into the client, and the prime changes every build.
- The client sends version info, then its public value A. The server replies with a
  20-byte block. The RC4 key comes from the DH secret XOR that block, through a hash.
  That hash matches what ascalon implements as `rc4_key`: one SHA-1 round over the
  folded input (see `crates/gw2-net`).
- Message type ids change between client builds.

## How this lines up with ascalon (inferred)

ascalon's `dh_params.py` looks for a 136-byte `.rdata` blob that starts `u32 1, u32 4`.
8 header bytes + 64 bytes (512-bit p) + 64 bytes (server public value) = 136. That fits
"g = 4, 512-bit prime, server public baked in".

**Implication (inferred):** our server holds its own private key y. In memory, the
client gets our `p` and `g^y mod p` written over the baked server public value (or
the whole 136-byte blob). Then we derive the same session key.
