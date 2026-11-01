# Build 209026: portal certificate acceptance

Investigation 2026-11-01, handoff step 3. **The RSA TLS path checks an issuer
trust anchor and a certificate signature, not just equality with a pinned leaf
key. It requires SHA-1/RSA certificate signatures and a matching DNS SAN.**
These are static findings; a successful GW2 handshake is still untested.

## Target and evidence boundary

REA `open_binary` confirmed the frozen `Gw2-64.new.exe` at
`/home/blizz/gw2-client/build-209026/`, SHA-256
`99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`.
Analysis uses REA. This investigation has not launched the client,
changed its memory, or changed its on-disk files.

The existing private snapshot
`~/re/gw2/notes/gw2-64-99342078ef57.snapshot` retains an inventory summary and
overview, not the certificate-validation functions. It cannot by itself answer
the acceptance question. Earlier trust-table observations and Evidence IDs are
in [209026-portal-pins.md](209026-portal-pins.md).

The Ghidra query for `0x14156db70` reached its 330,000 ms startup deadline without
returning pseudocode. That session was explicitly closed. The supported fallback
was REA `execute_rizin_command`: bounded reference searches and byte reads, then
bounded GNU `objdump` disassembly invoked through Rizin's `!` command on the same
frozen executable. This was static inspection, not target execution. Rizin's own
`pd` output contained `invalid` instructions at some valid instruction boundaries;
the conclusions below use the GNU disassembly for those functions.

Complete raw Evidence is private:
`~/re/gw2/notes/209026-certificate-step3-rizin-evidence.json`. The observed commands
report unchanged before/after executable hashes. Rizin warned that automatic
string inventory skipped large sections; explicit bounded reads/searches were
used instead. No complete string inventory or whole-program call graph is claimed.

## Established before this investigation

- The trust table contains DER Names and RSA public keys, including the
  cligate.101 identity. The STS connection setup receives this table. These are
  static observations recorded in the portal-pins document.
- Run 37 reached a TLS 1.2 ClientHello after the local server answered
  `/Auth/StartTls`; see [209026-login.md](209026-login.md).
- Neither observation proves that a self-signed certificate with a matching
  subject and a different key will be accepted.

## Local server identity (source inspection, not client evidence)

`tools/gen-test-cert.sh` generates our own RSA-1024 key (exponent 65537) and a
self-signed SHA-256 certificate. It uses a PrintableString subject with
`C=US, ST=Washington, L=Bellevue, CN=cligate.101.ncplatform.net`, and writes the
private key only to the selected ignored output directory.

`gw2_login::tls::ServerConfig::load` checks the private key's modulus length and
exponent, and searches the certificate bytes for that modulus. This is a local
configuration check, not an X.509 trust decision. The TLS implementation sends
the configured DER certificate chain, leaf first.

The earlier OpenSSL interop test establishes server-side TLS interoperability
with that test peer. It does not establish GW2 certificate acceptance.

## Observed static acceptance path

`0x14156db70` forwards to `0x14156eac0`; that constructor stores the supplied
configuration pointer. It is not itself the certificate validator. Searching for
references to the `TlsRsa/TrClient.cpp` string identified the RSA client class.
Its suite registration at `0x14158be30` installs `0x002f`, `0x0035`, `0x003c`,
and `0x003d`, consistent with the previously captured ClientHello. Its vtable
contains `0x14158b740`, which accepts handshake type `0x0b` (Certificate).

The following observations are from the complete bounded disassembly of that
handler, the certificate parsers, and their OID table:

| Check | Observation |
| --- | --- |
| Certificate framing | The handler takes a length-limited certificate list and iterates its certificates. Each certificate is parsed by `0x14158c610`. |
| Validity | Each parsed certificate is rejected if the time helper returns a value below `notBefore` or above `notAfter`. |
| Signature algorithm | The outer certificate signature algorithm must be enum **4**, mapped to OID `1.2.840.113549.1.1.5` (`sha1WithRSAEncryption`). SHA-256/RSA does not pass this check. |
| Public key | The SubjectPublicKeyInfo algorithm must be enum **3**, OID `1.2.840.113549.1.1.1` (`rsaEncryption`). Its BIT STRING is parsed as an RSA modulus/exponent sequence. |
| Algorithm parameters | `0x1417f5d10` requires an explicit zero-length ASN.1 NULL parameter and no remaining AlgorithmIdentifier bytes. |
| Bit strings | Public-key and signature bit lengths must be divisible by eight. |
| Leaf key usage | If KeyUsage exists, the decoded `keyEncipherment` bit must be set. Absence is allowed by this branch. |
| Leaf name | `0x14158c520` walks the SubjectAltName extension's GeneralNames and considers tag 2 (dNSName). It compares against the connection name supplied to the RSA client. Missing SAN returns failure; this helper has no subject-CN fallback. |
| Wildcard | The name helper recognizes a leading `*.` and compares the suffix after the first dot of the connection name. Exact DNS names avoid depending on wildcard handling. |
| Additional chain certificates | Issuer and subject Name contents must match in length and bytes. The issuer certificate needs BasicConstraints with CA true; KeyUsage, if present, must allow certificate signing. A present path-length constraint is checked against the running counter. |
| Unsupported critical extensions | The extension parser rejects an unrecognized extension when its critical flag is set; noncritical unknown extensions are ignored. |

The signature verifier at `0x1417f5950` applies the RSA public operation, parses a
PKCS#1-style signature block and DigestInfo, hashes the certificate's encoded
TBSCertificate with the SHA-1 implementation, and compares the digest. The SHA
constructor at `0x1415773c0` initializes the five SHA-1 state words and references
`Crypt/CptSha.cpp`. This is separate from the TLS 1.2 SHA-256 PRF used by our server.

## How the embedded trust table is used

After processing the supplied chain, the handler walks **nine 32-byte entries**
from `0x14261ff30` up to (excluding) `0x142620050`. Each entry contains a Name
pointer/length followed by an RSA public-key pointer/length. The earlier
`0x14261ff20` pointer/length pair is outside this loop.

The comparison is with the **issuer Name of the last supplied certificate**.
On a match, the handler parses that entry's RSA public key and verifies the last
certificate's signature with it. No matching entry, malformed RSA key, or failed
signature leads to failure. The leaf public key is retained separately for
encrypting the premaster secret after certificate acceptance.

For cligate.101, the matched table entry is at `0x14261ff90`: Name pointer
`0x14230b450`, length 90; key pointer `0x14230b4b0`, length 140. The 90 Name bytes
are the **contents** of the Name SEQUENCE, starting with the first RDN SET (`31
0b ...`), not a complete outer `30 <length>` wrapper. They encode C, ST, L and CN
as PrintableString, in that order. Display-equivalent Names with different DER
string types or ordering need not match.

**Derived consequence for the local emulator:** a correctly encoded self-signed
leaf can use the same key for leaf encryption and issuer verification, but that
key must be an explicitly configured local trust anchor in the isolated client.
Changing subject text alone cannot establish trust. The stock embedded key cannot
verify a certificate signed by our unrelated private key. No trust change was
performed in this investigation.

## Requirements for our next local test identity

1. RSA public key with `rsaEncryption` plus NULL parameters. RSA-1024/e=65537
   remains our chosen local compatibility shape; this investigation does not claim
   the validator only accepts 1024-bit keys.
2. A currently valid certificate signed using SHA-1/RSA, with NULL algorithm
   parameters, and a DNS SAN matching the connection hostname. The expected portal
   name from prior captures is `cligate.101.ncplatform.net`; the exact string passed
   to the TLS constructor still needs runtime confirmation, including any trailing dot.
3. For the intended single self-signed certificate, issuer Name contents matching
   the selected local trust entry byte-for-byte, and the corresponding own public
   key configured in that sandbox entry. The frozen executable remains untouched.
4. KeyUsage can be omitted, or include `keyEncipherment`. Avoid unneeded critical
   extensions. A multi-certificate test would additionally need the CA/chain checks
   above; it is not necessary for the proposed single-certificate experiment.

**Current generator mismatch, reproduced locally:** running the existing script
into a fresh ignored temporary directory and inspecting only public certificate
metadata with OpenSSL produced `sha256WithRSAEncryption` and **no SubjectAltName**.
The temporary identity was removed by the test harness. This is a source/fixture
check, not client runtime evidence. Those two fields need correction before a GW2
handshake experiment; the existing subject encoding and RSA shape can be retained.

## Evidence index

All IDs below refer to the private REA bundle named above.

| Evidence ID | Observation |
| --- | --- |
| `ev_cd0af071b7d811c9810bad7981f3f54c9f931bf13bb44d255ee38b0f51049332` | Setup wrapper at `0x14156db70`. |
| `ev_92dd88544c93a5a3af65b823663765cf75532317f21555506c56d342d38cd41e` | Constructor stores configuration at object offset `0x58`; its attempted string-search command failed and supplies no search evidence. |
| `ev_335bbb60249dbb64d81cdcf19e157bc60e1687f39f6c50062b5b93ea844adee6` | Bounded references to `TrClient.cpp` and RSA class vtable. |
| `ev_5f409108e62fcf06f29aa31b59c18051c0a28ddbc86ef4c7475d6e98235f2038` | Complete Certificate handler `0x14158b740..0x14158bc40`. |
| `ev_8ed39d58b583c121f1e1fbabedc5dbf6d73c0cf9dbd591ed1a1c8dd79114b636` | SAN matcher and outer certificate parser. |
| `ev_e390e4ddd29948ecff2476c8ccb3959ee5498bd407b1339b142372d7a77572a3` | TBSCertificate field mapping and AlgorithmIdentifier parser. |
| `ev_3cb8308e74d83165a1c1373f7dfa8ca47f186e0104f0367659218abb744b8422` | Extension parser, OID enum table and RSA signature verification. |
| `ev_4d7eb7670494172d45a4c08e920f0e9f311892a2f10cf2b5ae4d6f26feceaab1` | OID bytes, trust-entry pointers/lengths and SHA-1 constructor. |
| `ev_f95f94f6a3e2edc9d6bb23328455c2c7cb0f16f8c671624a3ac21df11c99359a` | Name contents, sequence parser, suite registration and key-exchange sender. |
| `ev_0a08b53d8dcb8f76381ef4d4f707d3f76cb7c334a7cf89c29c66cbf3f1a348ee` | AlgorithmIdentifier NULL-parameter and enum checks. |

## Remaining validation

The certificate acceptance checks and local identity requirements are mapped for
this RSA handler. Its linkage to the captured flow is inferred from the class,
handshake type and identical registered suites; static disassembly does not prove
that a live connection executed it. The exact constructor hostname, an accepted
local certificate, Finished exchange, and decrypted login messages remain to be
observed in the isolated sandbox. Step 4 is not complete.
