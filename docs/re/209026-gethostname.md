# Build 209026: GetHostname reply

Investigation 2026-11-01, following the first encrypted request captured in
[209026-login-tls.md](209026-login-tls.md). Frozen executable SHA-256 is
`99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`.
All client inspection below went through REA; raw evidence remains private.

## Static observations

- The authentication name table begins at `0x1421139e0`. Its entry at
  `0x142113a70` pairs `GetHostname` with enum 9.
- The sender at `0x140ffe2e0` selects enum 9, writes `LoginName`, and installs
  callback `0x140ffbb10` before submitting its request. This links the callback
  to GetHostname rather than relying solely on nearby strings.
- On a successful transport result, that callback obtains the reply's XML node,
  looks up `Local`, and records whether the lookup returned a node. It then
  retrieves the text of `Hostname`. A missing Hostname takes the error path.
  `Local` is tested by node presence, not by parsing its text as a boolean.
- The hostname is saved at object offset `0x8b8` and passed with the Local flag
  to `0x140ffac10`. That helper compares it to the current hostname and returns
  zero without replacing the connection when Local is present and the names
  compare equal. Otherwise it may replace the connection. The callback continues
  via `0x140fff5a0` on zero, or restarts its connection workflow on nonzero.

**Derived next experiment:** answer sequence 2 with `200 OK` and
`<Reply><Hostname>cligate.101.ncplatform.net.</Hostname><Local/></Reply>`.
The Hostname and Local fields follow the parser. The Reply wrapper follows the
existing STS convention; acceptance and the next request still require runtime
confirmation. No authentication success, ticket or character response is invented.

## Evidence

Private bundles: `~/re/gw2/notes/209026-gethostname-static-{1,2}.json`.
Commands report the executable unchanged. Bounded GNU objdump windows were
invoked through REA Rizin; a whole-text objdump reference filter located the
Hostname parser, then bounded windows established its behavior.

| Evidence ID | Observation |
| --- | --- |
| `ev_d7dc54fb0b594724ca67aeaf6c5f8b114f3b32010134d00f4869349a526245ce` | GetHostname string search. |
| `ev_8402037b89cf8bfc2e4e8b39a41fe3927a7fc636d119b252ba67e93eccd83473` | Pointer to the string in the authentication table. |
| `ev_330f4f6cc98b1d0cd09149f63c42534485856e05d2bbdd6ff550cfbe2b93761a` | Table entry with enum 9. |
| `ev_92a5619f978b8ae0b4be2a95fe78564a912f39b743b3beb77e80af18861c74f8` | References to Hostname and table registration. |
| `ev_69f2b9390f6851f45ba4b024a6d594aaa971ba1f9cd9adc0862eb4c23713787e` | Complete reply callback. |
| `ev_ab8cf89cdb6907b4a5cf3bbc41f5426b6f2b99b5ef33b9cc21cb8e445c4751d2` | Local/Hostname strings, callback reference, connection helper. |
| `ev_5b50e87e1719885a70d35472e24539801d071a7a214f6a6dba04bc0e393801df` | Enum-9 sender binds callback; XML lookup helpers. |

One exploratory `psz` command is unsupported by this Rizin version and supplies
no result. A wide `/r` search completed but exceeded the MCP response budget;
its retained Evidence `ev_5d4234fd1017203c4d7ef74fc83a5d12cb09d49bfef6b9a0c796558d7e247452`
was exported and read locally. It showed no code reference to the string itself;
the pointer table and callback linkage above resolve that indirection.

## Runtime follow-up

Run 40 uses the verified dotted certificate and guarded local trust configuration.
The observer's opt-in `--answer-gethostname` sends the proposed encrypted reply.
TLS succeeds and GetHostname arrives, but the client closes TLS and reconnects;
the breakpoint after extracting Hostname is not reached. No next authentication
request is observed. The candidate response is therefore **not yet validated**.

REA `ev_a3c9d3dd5c30f5ed56e4e4543e45d84c03b7d43af08353a1814d19b3b9577dd5`
is retained in `~/re/gw2/notes/209026-gethostname-run40.json`. Private logs are
under `~/gw2-sandbox/runs/run40-codex-gethostname/`. The bounded wrapper exited 0,
removed its container, verified unchanged executable hashes and 1,124,232 archive
entries. REA verified its owned group cleanup, with the previously noted wider
process-survey permission limitation. A following diagnostic will record callback
status and whether the missing-Hostname branch is taken.

Run 41 reaches callback `0x140ffbb10` with status **0**, then takes the
missing-value branch at `0x140ffbbc6`. The reply therefore reaches the callback,
but the current XML lookup does not find Hostname. This narrows the next check
to the returned node and its lookup method; it does not establish whether the
wrapper or the field representation needs correction. REA Evidence
`ev_aff6dd8facefd17a1c0e9d70040668e1094cb30c06bcd38efe86b6f60fbcad90`
is retained in `~/re/gw2/notes/209026-gethostname-run41.json`. Cleanup, hashes
and archive count pass again; no next authentication request is observed.

Run 42's diagnostic attempted to read the returned XML node at callback address
`0x140ffbb99`. GDB reported an access at address zero and ended the batch early.
This is a **probe error**, not a client crash claim. The next probe guards null
and records the reply object's XML method address before that method is invoked.
The wrapper still removed the container and verified executable hashes/archive
count. REA `ev_a0fa90f3815b1376c4c6bd29482ec8b6152eed1bc05547d40b6db6c1b90ff24d`
and the private run42 `client.out` are retained in
`~/re/gw2/notes/209026-gethostname-run42.json` and
`~/gw2-sandbox/runs/run42-codex-hostname-xml/` respectively.

## XML bodies require a final LF

The corrected run43 probe identifies reply vtable `0x1420afd20`, XML method
`0x140dcc1b0`, and a null node result. REA inspection of that exact method shows
the cause: before parsing a body it requires a nonzero length, then compares
the **last body byte with `0x0a`** at `0x140dcc24c`. If that test fails, it returns
null. On success it temporarily replaces the final LF with NUL and invokes the
XML parser. Our candidate ended at `</Reply>` and therefore never reached parsing.

Static Evidence `ev_00e1c4dd56a018cac03daea7990c50e153747981cbc4b0ff51ffa6c8d3011fc2`
and return-path Evidence
`ev_703f25ddbe9139ab6c555403397da67fa72d9ff923fd6cdf3adf5ada599f2f95`
are exported in `~/re/gw2/notes/209026-gethostname-newline.json`. That bundle also
retains run43 Evidence
`ev_067033c788f3a479b1b3df19e3e1870d6ff467dd0349baf17192207be4898b48`:
status zero, exact reply vtable/XML method, null node, missing-value branch;
container cleanup, hashes and archive count pass.
**Correction to test:** append LF to the GetHostname reply XML and include it in
`l:`. No content-type header or alternative XML wrapper is justified by this
failure. StartTls previously succeeded without LF because its reply did not need
this XML field lookup; that observation does not establish that all STS bodies
can omit their terminator.

## Run 44: corrected reply accepted

With the same XML plus final LF, the callback returns an XML node, reads the
expected dotted hostname, and observes Local present. The client then sends
`P /Auth/StartSsoLogin STS/1.0`, `l:253`, `s:3;timeout=29000`, over the same
TLS connection. Its Request has six children:

| Field | Text length in this synthetic run |
| --- | --- |
| CID | 36 |
| Provider | 6 |
| LoginName | 18 |
| Scopes | 18 |
| Language | 2 |
| Password | 32 |

Values are omitted. The Password field's encoding and authentication semantics
are not established by its length. No real account or credential cache was used.
This is an observed **StartSsoLogin** path, not proof of the older SRP flow.

REA Evidence
`ev_370f25feb8798c57399e99cff8c49ddf1fbd2f4a503e070d483748bba4d7b20b`
is exported in `~/re/gw2/notes/209026-gethostname-run44.json`; private logs are in
`~/gw2-sandbox/runs/run44-codex-hostname-newline/`. The wrapper exits 0 and verifies
container removal, unchanged executable hashes, and 1,124,232 archive entries.
REA verifies its owned process group cleanup with the same wider process-survey
limitations. The focused observer build, TLS StartTls/GetHostname smoke, Clippy
and diff check pass. Next: map the StartSsoLogin reply, without claiming account
authentication or character selection yet.
