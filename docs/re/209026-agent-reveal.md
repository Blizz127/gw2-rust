# Build 209026: collision entry and agent reveal callbacks

Static REA follow-up to [world entry](209026-agent-world-entry.md),
[scene entry](209026-agent-scene-entry.md) and the
[constructor-owned bodies](209026-entry-collision-bodies.md).
The frozen executable is unchanged; no runtime messages were sent.

Collision scene entry supplies the callback that sets
**AGENT_FLAG_REVEALED**. It can execute immediately or through the scene's
deferred event queue. This establishes the flag transition, not successful
rendering, model readiness or a complete playable bootstrap.

## Callback ownership

Base agent construction `0x141024aa0` sets R14 to whole agent +0x18 at
`0x141024ac6`, installs bound-interface vtable `0x14211aee0`, and passes
that pointer as the last argument to either collision factory. Full factory
`0x14109ca00` forwards it in R8 to `0x14109a900`; simple factory
`0x1410a5e20` forwards it in R8 to `0x1410a54f0`. Both constructors move
that argument to R9 for shared constructor `0x1410c0850`, which stores it
at secondary collider +0x58 (whole collider +0x68).

Thus the callback pointer used during entry is the bound agent interface,
not a separately allocated network object. Its relevant vtable entries are:

| Bound interface slot | Implementation | Effect |
| --- | --- | --- |
| +0x08 | `0x141032ab0` | OR 0x20 into bound +0x30: set REVEALED |
| +0x10 | `0x141032ad0` | AND ~0x20 into bound +0x30: clear REVEALED |

After changing the flag, each callback loads bound +0x20, then that
object's +0x10. If the latter is nonnull, reveal calls its virtual +0x30
and conceal calls its virtual +0x18. The constructor stores whole agent
at whole +0x38 (bound +0x20), establishing the first pointer in this chain.
The character-side effects of those final calls remain a separate trace.

## Immediate entry and exit

Shared scene entry `0x1410c1710` first sets secondary collider +0x48
mask 0x02, **COLLIDER_FLAG_ENTERED_SCENE**. After optional rigid-body
bookkeeping it calls `0x1410c3c30(scene +0x180, bound_agent)` when the
stored callback is nonnull.

When event-manager +0x00 bit 0 is clear, `0x1410c3c30` tail-calls the
callback's virtual +0x08 immediately. When that bit is set, it allocates
a 16-byte event containing kind 1 at +0x00 and the callback pointer at
+0x08, then appends it to manager +0x10, incrementing count +0x1c.
The collider-entered flag can therefore precede the agent-revealed flag.

In the non-deferred-removal branch of shared exit `0x1410c1760`, the
collider-entered bit is cleared before `0x1410c3ce0(scene +0x180, callback)`.
That function uses the same manager gate: immediate virtual +0x10,
or a queued kind-2 record with the same layout. The earlier exit gate at
scene +0x210 bit 0 can instead defer removal itself; that separate queue
has not been exhaustively traced here.

## Deferred dispatch

The opening loop of `0x1410c2730` clears manager bit 0, walks the current
+0x10 array through count +0x1c, and calls `0x1410c4180` on each nonnull
record slot. It then zeroes the count. The dispatcher switches on event
kind using the RVA table at `0x1410c430c`:

| Event kind | Branch | Callback |
| --- | --- | --- |
| 1 | `0x1410c41b9` | Stored interface virtual +0x08: reveal |
| 2 | `0x1410c41c8` | Stored interface virtual +0x10: conceal |

After dispatch it frees the event record and nulls the array slot. A
second complete helper, `0x1410c3f90`, scans the same queue but dispatches
only records whose callback equals its second argument; its kind-1/2
table entries likewise reach virtual +0x08/+0x10. This note establishes
both dispatch mechanisms without assigning an unverified frame timing
or asserting that every caller has been enumerated.

## Bootstrap consequence and limits

Server 46 normally reaches collision scene entry after agent world entry;
entry-option bit 1 skips that collision operation. Server 49 reaches the
collision operation separately. Both therefore have a traced route to
REVEALED, subject to their existing admission and entry prerequisites.
No extra reveal packet is required by this inspected route. This does not
select valid initial option bits or remove duplicate-entry assertions.

The flag is distinct from AGENT_FLAG_ENTERED_WORLD, the collider's entered
flag and the full collider's separately allocated visibility phantom.
Finally, `0x14103ea90`, called at the end of these command paths, is a
conditional serialization hook: command mask 0x400000 and a world predicate
gate a virtual +0x30 call with direction 1. Its inspected body does not
set REVEALED or itself destroy the command; descriptions of a completion
hook should not be read as proof of either behavior.

## Evidence

Private bundle: `~/re/gw2/notes/209026-agent-reveal-1.json`.
All addresses refer to frozen build 209026, executable SHA-256
`99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`.

| REA evidence | Observation |
| --- | --- |
| `ev_31e6420ced604d7307d8efc7da4ebb61544dfe71028937544429e31b7d9056ef` | Agent constructor pointer provenance; complete immediate/queued reveal and conceal helpers. |
| `ev_b40e7f6b6dca331a5a5b8447eee1318263c050c4e9ac9041383e45836fe3bd07` | Both collision factories forward the agent callback to their constructors. |
| `ev_157b75de2d2e7de9bffa638bd94694343fb3e909152851cc1608b10574047a4a` | Shared collider stores R9 at secondary +0x58; agent stores the factory-returned game interface at whole +0x10. |
| `ev_279ca1fb5cc557ea52446709a1f42505324a923f058486ef159c653fd7b16972` | Full/simple constructor argument forwarding to the shared collider. |
| `ev_712b3b4f12f88cd923192d012c270a8b74cbf6894173e3420fdb55e16934f1fc` | Complete bound-agent flag setters and downstream notifications. |
| `ev_500ec65ec6a1195d0bf137f75b981da04d125183ab2dc896338efbccfac23f3e` | Bound-interface vtable +0x08/+0x10 resolve to those setters. |
| `ev_9ac693fde3e1a795a55d8ebb992c06a43cb68cf2f06c76bcb990a519dd4b79ee` | Complete shared entry, exit callback branch and opening queue-drain loop. |
| `ev_7a64686cff402ff60c1b1c4dc4a5a27dbd6ee40019317dc4ad2faa2167ace0c6` | Complete event dispatchers, callback calls, cleanup and jump tables. |
| `ev_28809d306dcbaaf8e8691c5e3058151b5c13f5c3213ff9a02811beafe188e52a` | Raw jump-table entries confirm event 1/2 branch mappings. |
| `ev_ff972aa2a3e49118fdf3864e96b6c371b1ae40270ff6940f276001f96669563b` | Complete conditional command serialization hook. |
