# Build 209026: first network behaviour, observed at runtime (non-REA)

**Method:** runtime observation of the frozen client in the offline sandbox. This is **not
REA evidence**: it comes from our own DNS sinkhole and listeners plus Wine's
`+winsock` trace. Static confirmation through REA is still pending (the Ghidra import is
running).
- Raw logs (private): `~/re/gw2/notes/run3-dns.log`, `run3-capture-port80.log`,
  `run4-dns.log`, `run4-winsock.trace`.
- Setup: `sandbox/run.sh client 209026` with Docker `--network none`, `--dns 127.0.0.1`,
  `gw2-dns-observe` answering 127.0.0.1 for every name, and `gw2-login-observe` on
  127.0.0.1 ports 80, 443, 6112 and 6600.
- Client: the prepared working copy, `Gw2-64.exe` sha256 `99342078…8d621`, no arguments.

## Observed (runs 3 and 4, 2026-10-31 ~20:03–20:08Z)

1. **Name lookups:**
   - the container's own hostname
   - `assetcdn.101.ArenaNetworks.com` (via `WSAAsyncGetHostByName`)
   - `cligate.101.ncplatform.net.` (also via `WSAAsyncGetHostByName`, with a trailing dot)
2. **Exactly one TCP connection** in a 120 s run:
   - The client creates a socket with `WSASocketW(AF_INET, SOCK_STREAM, TCP, OVERLAPPED)`,
     `FIONBIO`, binds it to `0.0.0.0:0`, then calls
     `ConnectEx({127.0.0.1, port 80}, send_len 0)` → `status 0x103` (pending). That is the
     sinkholed address of `cligate.101.ncplatform.net`. The DNS log puts the lookup 2 ms
     before the connect.
   - Our observer accepted the connection and received **0 bytes**.
   - After about 10 s the client set `SO_LINGER` 0 and called `closesocket`, which the
     observer saw as `Connection reset by peer` after 10,024 ms.
3. **No connection to `assetcdn` was attempted** in the window, although the name was resolved.
4. The UI meanwhile shows the launcher with "Connection error(s) detected. Retrying…".

## Inferred

- `cligate.101.ncplatform.net` is the client gateway, the portal/STS entry point.
  **Today it is reached on TCP 80**, not the 6112/6600 of the 2012–13 notes. (Port 80 is
  observed; the role is inferred.)
- Redirecting by **hostname** works without client changes: the client honours the
  system resolver. A runner can map `cligate.101.ncplatform.net` (and `assetcdn…`) to our
  server through DNS or hosts, so `-authsrv` may not be needed for routing. Whether
  `-authsrv` still exists remains to be checked through REA strings.
- The 0-byte, 10 s pattern means one of two things: (a) the client waits for the server to
  speak first, or (b) the client never saw the overlapped `ConnectEx` complete under Wine
  and timed out. **Unknown**; REA and the next run settle it. A test is to have the
  observer send a probe byte, or to trace `+iocp`/`+sync`.

## Follow-up experiments (2026-10-31 ~20:45–20:50Z)

- **Wine control probe** (`tools/probes/connectex-probe.c`, our own program, built with
  `zig cc` for x86_64-windows-gnu). It runs in the same image and user setup and mirrors the
  client's socket sequence: overlapped `WSASocketW` → `FIONBIO` → `bind(0.0.0.0:0)` →
  `ConnectEx` → IOCP.
  - **Wine delivers the `ConnectEx` completion** (`connect completion: got=1`).
  - The probe's follow-up `setsockopt(SO_UPDATE_CONNECT_CONTEXT)` **does** appear in the
    `+winsock` trace. The client's trace never shows that call.
  - Wine's `+winsock` channel does not trace `WSARecv`/`WSASend` at all, so the missing
    recv/send lines in the client trace prove nothing on their own.
- **Run 5, greeting:** the observer sent 18 bytes (`STS/1.0 200 OK\r\n\r\n`) right after
  accept. **The client's behaviour did not change:** 0 bytes sent, reset after 10,027 ms,
  the same `SO_LINGER` + `closesocket`.
- **Revised inference:** "the server must speak first" is now unlikely, because the client
  doesn't even react to data. The likely explanation is that the client never handles its
  connect completion under Wine and gives up on a 10 s connect timeout. Its completion
  mechanism (IOCP thread, thread-pool IO, event or alertable wait) is not identified yet.
  **REA should find the cligate connect path**: xrefs from the `cligate`/`ncplatform`
  strings to the socket code.

## Unknown

- The protocol on cligate:80: STS/1.0 text, TLS-first, or something new.
- The asset CDN protocol and when it is first contacted. It is probably HTTP patching;
  the launcher must keep that suppressed or sinkholed.
