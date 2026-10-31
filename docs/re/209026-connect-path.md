# Build 209026: why the first connection sent no bytes, and the fix

Question (task c2a832199b53): when the client connects, must our server speak first, or does
the client never notice the connection opened?

**Answer: neither. The client speaks first, as plain STS text.** It was silent because one
of its own threads died. ArenaNet's StsLib address lookup (`FUN_140e3b090`) recurses
forever when the machine's only local address is in 127.0.0.0/24, which is the case in
our `--network none` sandbox. It ran on the client's single socket-pool I/O worker and
used up that thread's 1 MiB stack. Wine killed the thread. After that, nobody read the
socket completion port: every ConnectEx completion was queued and never seen, so the
client sent nothing and gave up after its 10 s connect timeout.

**Fix (sandbox side, client untouched):** resolve the ArenaNet hostnames to loopback
addresses *outside* 127.0.0.0/24 (we use `127.1.0.1` for cligate and `127.1.0.2` for the
asset CDN). The client classifies those as type 4, not type 5, so the lookup doesn't
recurse.

With that fix (run 21):
- The client opens `cligate.101.ncplatform.net` on **TCP 6112** and sends
  `P /Sts/Connect STS/1.0` first (below).
- The port-80 connection in earlier runs was **the asset CDN patch check, not cligate**.
  An earlier note in `209026-runtime-net.md` mistook it for cligate, inferring that from
  the DNS order.

Evidence IDs are REA (Ghidra 12.1.4) results on `Gw2-64.exe` sha256 `99342078…8621`.
Runtime results are non-REA (Wine traces, gdb stack dump, our listener inside the
`--network none` sandbox), kept privately in `~/re/gw2/notes/`.

## Observed (REA, static)

**Connect path:**

| What | Where | Evidence |
| --- | --- | --- |
| BeginConnect sets a deadline of `timeGetTime() + (timeout ? timeout : 10000)`, then: `CreateIoCompletionPort(sock, socketPool.port, key=socket obj)` → bind 0.0.0.0:0 → ConnectEx (GUID `0x142584a10`). Pending connects sit in a deadline list watched by the `"SocketListen"` thread | `FUN_1409d7d20`, `FUN_1409ce350`, `FUN_1409da1e0` | ev_a4b641d4…, ev_051c153b…, ev_4e6a3d04… |
| Three I/O pools with one completion port each: 0 `Compute`, 1 `Socket`, 2 `File`. Workers run a non-alertable `GetQueuedCompletionStatusEx(port, e, 8, &n, t, FALSE)` loop | `FUN_1409cf700`, `FUN_1409cdf40`, `FUN_1409ceb90` | ev_22c90f28…, ev_51c8d644…, ev_aa6e9e25… |

**Address classification and the recursion:**

| What | Where | Evidence |
| --- | --- | --- |
| `AddressGetType` (IPv4): 169.254/16 → 6, **127.0.0.0/24 → 5**, other 127/8 → 4, 192.168/16 → 3, 172.16/12 → 2, 10/8 → 1, else 0 | `FUN_1409c0940` | ev_ea676469… |
| StsLib address lookup: a lock at `table+0x20`, then a table lookup. On a miss with type ≥ 5, it takes `s_addresses[0]` (`StsMain.cpp`, the local address list) and **calls itself**. Below 5 it returns `"Unknown"` | `FUN_140e3b090` (recursive call at `0x140e3b103`), `FUN_140e35b80` | ev_314d70b6…, ev_a05d45eb…, ev_ad46e489… |
| Entry with the global table `0x142593908` | `FUN_140e3fc80` | ev_6875a9a3… |

**STS message I/O:**

| What | Where | Evidence |
| --- | --- | --- |
| Message writer (`InetMsgOut.cpp`): start line `STS/1.0` or `HTTP/1.1` | `FUN_140dd1eb0` | ev_01aeb7ab… |
| Start-line parser: `STS/1.0` or `HTTP/1.x` | `FUN_140dd0830` | ev_01800f34… |

## Observed (runtime, non-REA)

| Run | Setup | Result |
| --- | --- | --- |
| 11, 12 | DNS sinkhole → 127.0.0.1, `+server,+winsock` | The socket-pool worker (the only thread calling `remove_completion` on the socket port) dequeues a job and dies: `stack overflow 4672 bytes addr 0x6fffffc64393 … (0x7c0000-0x7c1000-0x8c0000)`, `*killed*`. The login socket is then attached to that port; ConnectEx is `PENDING` forever; the listener sees 0 bytes and a reset at 10 s |
| 13 | `/etc/hosts` with assetcdn only (cligate didn't resolve: trailing-dot name) | No overflow. The client sends `GET /latest64/101 HTTP/1.1`, `Host: assetcdn.101.ArenaNetworks.com` on port 80 |
| 14, 15 | cligate resolves to 127.0.0.1, then 127.0.0.2 | Overflow is back, so it's not a same-address conflict |
| 16, 17 | `+loaddll`, `/proc/pid/maps` | The fault is in Wine's `ntdll.dll` (base `0x6fffffc20000`): `call_seh_handlers` (signal_x86_64.c:218) with `-all`; `pf_vsnprintf_a` with `+seh`. That's where the stack ran out, not the cause |
| 20 | gdb breakpoint on Wine's "stack overflow" write; 1 MiB stack dump | Return address `0x140e3b108` (just after the self-call in `FUN_140e3b090`) appears **9270×**, ≈113 bytes/frame. Outer frames: pool worker → job dispatch → StsInetSocket (`0x14157bf17`, `0x1415806c3`) → `FUN_140e3fc80` → recursion |
| 21 | cligate → 127.1.0.1, assetcdn → 127.1.0.2 (`/etc/hosts`) | **No overflow.** cligate gets a connection on **6112** with the client speaking first (below), then `/Sts/Ping` every 30 s. The asset CDN gets the `GET /latest64/101` patch check. The launcher shows "Downloading 0KB (0KB/sec)" while it waits for that answer |

The first message on cligate:6112 (run 21; 284 bytes, no `s:` header, so one-way):

```
P /Sts/Connect STS/1.0
l:251

<Connect>
<ConnType>400</ConnType>
<Address>127.0.0.1</Address>
<ProductType>0</ProductType>
<ProductName>Gw2-64</ProductName>
<AppIndex>1</AppIndex>
<Epoch>815182995</Epoch>
<Program>101</Program>
<Build>1004</Build>
<Process>32</Process>
</Connect>
```

The asset CDN patch check, on port 80 (an authCookie value appears in the request):

```
GET /latest64/101 HTTP/1.1
Cookie: authCookie=access=/latest/*!/manifest/program/*!/program/*~md5=…
Cache-Control: no-cache
Pragma: no-cache
Host: assetcdn.101.ArenaNetworks.com
Connection: keep-alive
Content-Length: 0
```

## Inferred

- On a real machine `s_addresses[0]` is a LAN or public address (type < 5), so the
  recursion stops after one step. That is why this never shows up for normal Windows or
  Proton players. It is triggered by an offline host with only loopback, not by Wine.
- The launcher waits on the patch check before anything else (UI shows Downloading 0KB).
  The STS conversation probably continues (`/Auth/…`) once the patch step is satisfied or
  skipped. Not confirmed.

## Unknown, and the next steps

- The `/latest64/101` response format. Next: find its parser in REA, then serve a
  response that reports build 209026 as current.
- What the client sends on cligate after `/Sts/Connect` and `/Sts/Ping` once the launcher
  moves past patching (auth flow, StartTls or TLS, and the pinned RSA keys).
- Whether `/Sts/Ping` needs a reply. The client sent two over 60 s without one and kept
  the connection open.
