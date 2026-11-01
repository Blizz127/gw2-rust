# Build 209026: pinned portal (cligate) TLS identities, the portal's "baked server key"

Target: `Gw2-64.new.exe`, sha256 `99342078…8d621`.

**Follow-up 2026-11-01:** [209026-certificate-check.md](209026-certificate-check.md)
maps the RSA Certificate handler. These Name/key pairs act as issuer trust anchors:
the last certificate's issuer Name selects a key for signature verification.
The older leaf-pinning interpretation below is superseded. Our local certificate
also needs SHA-1/RSA signing and DNS SAN; a modulus change alone is not sufficient
with the original fixture. No runtime trust change or accepted handshake is claimed.

**REA evidence:**
- `read_bytes` 0x14230b1c0 (1792 bytes): `ev_5e942e1c11c0fad8a2860a67ba01d8b9e7525811ed764838c0399043a4c46b03`.
- The cligate strings come from `search_strings` (`ev_d635d76e…`).
- `xrefs` to `0x14230b490` returned no direct references (`ev_337e492a…`): the hostnames are
  inline fields of a table, not standalone strings.

**Cross-check (non-REA):** the same table parsed from the frozen file (file offset
`0x23097c0`) with a format-aware DER scan. Output (private): `~/re/gw2/notes/cligate-pins-209026.txt`.

## Observed

A table of pinned server identities in `.rdata`. Each entry is a DER X.509 Name
(`C=US, ST=Washington, L=Bellevue[, O=ArenaNet, Inc.], CN=<host>`) followed by an
`RSAPublicKey` (1024-bit modulus, e = 65537):

| CN | RSAPublicKey at | modulus sha256 (prefix) |
| --- | --- | --- |
| cligate.102.ncplatform.net | 0x14230b2d0 | 87beb349dfffc02f |
| cligate.103.ncplatform.net | 0x14230b3c0 | c7bdd1b59786361d |
| **cligate.101.ncplatform.net** (the host the client contacts) | 0x14230b4b0 | 6ce966979e2e3874 |
| cligate-pvg.112.cnportal.net | 0x14230b5c0 | 3ae3244e04ce977c |
| cligate.113.cnportal.net | 0x14230b6d0 | 8aef9fc2b994111d |
| cligate.112.ncplatform.net | 0x14230b7c0 | 34e1dae899358d31 |
| GeoTrust SSL CA - G2 (RSA-2048) | 0x14230b8a0 | — |
| cligate.111.cnportal.net | 0x14230ba30 | 93f7749782192f02 |

## Inferred

- The portal connection to `cligate.<n>` (TCP 80 at runtime) is authenticated against these
  **pinned RSA-1024 keys**. This is the "server key baked into the exe" for the
  portal/login step. It changes when ArenaNet rotates keys, so it is pinned per build.
- **For our server:** generate our own RSA key and, at launch, write its modulus over the
  pinned modulus for `cligate.101` (`0x14230b4b7`, 128 bytes) **in the running process's
  memory only**. The exe on disk is never edited. Point `cligate.101.ncplatform.net` at our
  host. We don't need the real private key.
- The game connection's DH parameters (ascalon `dh_params.py`: 136 bytes in `.rdata`,
  starting `u32 1, u32 4`) are a separate key for the game server, still to be located.

## The trust table and who loads it (REA)

- A masked pointer search (`ev_60d22e63…`) found a `.data` table of `{ptr, len}` pairs at
  **`0x14261ff20`** (`ev_6c2c0c5a…`):
  - `0x14230a450`/4000: unknown, maybe a certificate bundle.
  - `0x14230afe0`/125: the StartCom Certification Authority Name (`ev_f5acb264…`).
  - `0x14230b060`/526: an RSA-4096 key, probably StartCom's.
  - Then (Name, RSAPublicKey) pairs for cligate.102, .103, .101, .112 (ncplatform),
    cligate-pvg.112 and .113 (cnportal), the GeoTrust SSL CA - G2 (key length 0x10e, RSA-2048)
    and cligate.111 (cnportal).
  - Then `{1, 0xc020}` and later a pointer to `0x14230beb0`.
- `xrefs` to `0x14261ff20` (`ev_147edaa9…`) give 8 sites in `FUN_141585490` and its
  neighbour (`ev_1bf2aae1…`).
- **Observed** in `FUN_141585490` (`ev_3c0a59cfe9ccae330da4c22c1a49618e54d10c3c2852d11a54679f51813876d5`):
  - It calls `FUN_14156db70(1, connType, &PTR_PTR_14261ff20, 0, 0, …)` for connTypes
    **0x47 'G', 0x50 'P', 0x44 'D'**, and for a caller-given `connType` when flag 0x800 is set.
  - It asserts `"connType > MAX_BYTE"` with source path
    `…\Arena\Services\StsInetSocket\StsInetListen.cpp`.
- **Inferred:** the portal is the ArenaNet **STS** stack (StsInetSocket). Each STS
  "connection type" is keyed by one byte, and 'P' matches the first byte of
  `P /Sts/Connect STS/1.0`. Each gets this trust store (CA roots plus pinned cligate RSA keys)
  for its secure (`StartTls`) phase. Our existing `gw2-login::sts` codec is on the right track.

## Historical unknowns (superseded in part)

- At the time of these observations, the portal record format on TCP 80 was unknown. The 2012–13 notes describe STS/1.0 with
  `/Auth/StartTls` and then TLS-like records with SRP. Whether the client speaks first is
  still unsettled: at runtime it sent 0 bytes in 10 s under both Wine 10.0 and GE-Proton 11.
- The runtime connection and STS framing were subsequently observed on TCP 6112:
  [209026-login.md](209026-login.md).
- The RSA Certificate handler at `0x14158b740` directly consumes entries
  `0x14261ff30..0x142620050`; see the certificate-check follow-up above.
