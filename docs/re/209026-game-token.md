# Build 209026: RequestGameToken

Investigation 2026-11-01, following run 46's request. Frozen executable SHA-256:
`99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`.

## Static observations through REA

Request constructor `0x140fffe40` selects Auth enum `0x13` (RequestGameToken)
and writes GameCode and AccountAlias. Caller `0x140ffa8c0` constructs the response
object with interface vtable `0x142111990`. That table binds success handler
`0x140ff75b0`.

The handler obtains the XML root, retrieves the **Token child text**, and parses
it using the same 16-byte GUID decoder used for the synthetic SSO UserId. Missing
XML or failed GUID parsing takes the error path. On success it passes the GUID
to the caller's callback at vtable offset `0x20`.

The field representation was checked explicitly: XML parser `0x140e28170`
returns a node wrapped by `0x140e28be0`, which installs vtable `0x1420b91e0`.
Its `+0x100` method `0x140e271d0` forwards to `+0x108` (`0x140e271f0`); that method
looks up a child through `+0xe8`, then returns its text through `+0x10`. Thus
Token is an element, not an attribute. Shared field getters using this method
also read child text; their mere presence does not establish which request uses
them.

## Offline runtime: run 47

A fresh synthetic Token GUID in `<Reply><Token>…</Token></Reply>` followed by LF
passed GUID parsing at `0x140ff761f` and reached the completion call at
`0x140ff7634`. The client then opened a second TCP connection to the local
observer on port 6112. It sent 16 initial binary bytes followed by 66 bytes;
the observer switched this connection from STS parsing to raw capture. The
original TLS portal connection remained open and sent `/Sts/Ping`.

This establishes the transition from the synthetic portal replies into the
binary game connection. It does not establish a completed game handshake,
credential validation, a session-token database, or character selection.
The next step is to identify and answer the binary handshake from build-specific
evidence. The attempted libc connect probe emitted no markers, so it does not
provide complete destination-call tracing; the observer establishes the local
accepted connection.

The run used Docker `--network none`, synthetic credentials and private fixtures.
Container cleanup and unchanged frozen/working executable hashes were verified;
the archive still contains 1,124,232 entries. REA verified cleanup of its owned
process group; its broader host survey remains incomplete because unrelated
process metadata was inaccessible. Raw capture bytes and token values remain
outside the repository.

## Evidence

Private bundle: `~/re/gw2/notes/209026-game-token-static.json`. REA Rizin byte
reads and bounded GNU objdump windows, plus a whole-text caller reference filter;
the target hash remains unchanged.

| Evidence ID | Observation |
| --- | --- |
| `ev_dacd33dfa26a50a60ab6adeeb753b55c1c55b6ba94d6901d226dfe776db1cbb7` | Request constructor and complete token handler. |
| `ev_d5a010313bb223aa136f19f4f736c77d84f815f2275d2b7821355639b2670adb` | Caller reference to request constructor. |
| `ev_51457786b73c72e5c65b2dd783df38b4b058ce158ebd634ed56d7e0f2889b934` | Caller builds interface vtable `0x142111990`. |
| `ev_98fc20219cd04bcf0a85168dcff6c091b1b1ddc1f643d9f6c72ddfa791980595` | Vtable binds token handler; Token literal. |
| `ev_752fda0957b3b22e37147c91d0fd26fe8e821c6e9c8bf6838309c5d00501d83e` | XML parse return path and node vtable slots. |
| `ev_4994d0f9f126618afc41ad369c03bf3a1e4f4b16878849eadb7f75fcacd6c32a` | Node wrapper installs vtable `0x1420b91e0`. |
| `ev_ef32021c1c80c214aaec948fb2411a74e5a64a811ce7bd574f4ff41eaced24b9` | Token lookup resolves child text. |

Runtime evidence: `ev_99e9cd82d4286f823e6bf2c469c655482837612a4b4b01969284674a2358532b`,
exported privately to `~/re/gw2/notes/209026-game-token-run47.json`. Run files:
`~/gw2-sandbox/runs/run47-codex-game-token/`.
