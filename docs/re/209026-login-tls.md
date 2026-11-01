# Build 209026: local StartTls integration

Investigation 2026-11-01, handoff step 4. Target is the frozen executable
SHA-256 `99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`.
Runtime experiments use REA process capture around the Wine sandbox, Docker
`--network none`, the working client copy, and synthetic login input only.
Raw captures and generated identities stay private outside the repository.

## Server change and focused check

`gw2-login-observe --tls-key PEM --tls-cert DER` now answers `/Auth/StartTls`,
passes any prefetched ClientHello bytes into `gw2_login::tls`, and reassembles
decrypted STS messages. Capture files are created exclusively with mode 0600;
console output contains request paths and body lengths, not body values.
`SANDBOX_LOG_DIR` selects a separate sandbox run directory to preserve evidence.

The source-only smoke `python3 tools/test-starttls.py` passed: an OpenSSL-backed
Python TLS peer verifies the local certificate/hostname, pipelines its ClientHello
after StartTls, then sends split and coalesced STS messages. Both are decoded,
close_notify completes, and capture permissions are checked. The observer builds
and its focused Clippy check passes. These are not GW2 runtime observations.

## Run 38: hostname includes the final dot

Private run directory: `~/gw2-sandbox/runs/run38-codex-tls/`.
The RSA Certificate handler at `0x14158b740` is reached with handshake type 11.
Its connection hostname at object offset `0x70` is exactly
`cligate.101.ncplatform.net.` (including the final dot). The first test certificate
has a SAN without that dot. The handler returns 0 and the server records fatal
TLS alert 10; no encrypted login request was captured in this attempt.

The test changes only the running process's cligate.101 trust modulus, after
checking all 128 original bytes against the REA static read and checking the new
bytes after writing. The frozen and working executable files are not patched.
No validation instruction or return value is modified.

The SAN helper's exact-length comparison was already observed in step 3
(`ev_8ed39d58b583c121f1e1fbabedc5dbf6d73c0cf9dbd591ed1a1c8dd79114b636`).
**Inference:** the missing final dot explains this rejection. A following run
will use the exact hostname and observe the SAN result before claiming acceptance.

REA runtime Evidence:
`ev_37017964525c084d8bcd621bb5e6c0cae68f993be85248fbbfaee2e8b1026e30`,
exported to `~/re/gw2/notes/209026-step4-run38.json`. The wrapper exited 0 after
its three-minute bound, removed its exact Docker container, confirmed both EXE
hashes unchanged, and read 1,124,232 archive entries afterward. REA verified its
owned process group and temporary-root cleanup. Its wider process survey could
not read some `/proc/*/environ` files; this is a survey limitation, not evidence
that all host processes were inspected or attributable to the run.

## Run 39: certificate accepted, TLS completes

Private run directory: `~/gw2-sandbox/runs/run39-codex-tls-dot/`.
The fresh certificate uses `DNS_NAME=cligate.101.ncplatform.net.`; its issuer
Name retains the undotted CN, matching the embedded issuer entry. The SAN helper
return observed at `0x14158b910` is 1, and the Certificate handler returns 1.
With the same guarded in-memory local trust configuration, the Rust server
completes TLS 1.2 AES128-SHA and decrypts the first application request:

```text
P /Auth/GetHostname STS/1.0
l:91
s:2;timeout=29000
```

The XML root is `Request`, with `LoginName` (18 characters of synthetic test
input) and `Provider` (6 characters). Body values are deliberately omitted.
The request is **GetHostname**, not the hypothesized LoginStart. The observer
does not yet answer it. This proves TLS and the first encrypted STS request,
not account authentication, a session ticket, or character selection.

REA runtime Evidence:
`ev_b2c9d157a3d10473c9e9d33484c0d4c3ab6e348fd87118fa6c3fbe0b1916caa9`,
exported to `~/re/gw2/notes/209026-step4-run39.json`. The wrapper printed only
headers, XML tag names and lengths from the private decrypted capture, then
exited 0. Docker network mode was `none`, the sandbox reported loopback-only
networking and unreachable outbound routing, and the exact container was removed.
Both executable hashes stayed unchanged and the archive still has 1,124,232
entries. REA verified its owned process group and temporary-root cleanup, with
the same wider `/proc` survey limitation as run 38.

**Next:** map the GetHostname response parser through REA, document its required
fields, then implement the smallest local reply and observe the next request.
