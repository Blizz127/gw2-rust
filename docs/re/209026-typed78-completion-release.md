# Build 209026: typed0x78 completion and release

**Ordinary duration expiry reaches completion, but completion alone does not
normally release typed0x78.** Its concrete completion callback returns1 only
when order flag **0x2000** is set at its final test. The correlated server84
**0x45** handler is a demonstrated way to request that release: it defers
while the step is active, or releases directly after the active flag clears.
This does not recover a retail10176 packet sequence.

The source investigation continues
[ordinary command expiry](209026-skill-order-duration-expiry.md) and
[the existing skill-command path](209026-mesmer-skill-command-path.md).
No gameplay, helper, runtime, archive or deployment changes were made. A
separate raw protocol codec is described below; it sends nothing itself.

## Concrete callbacks and completion result

The proven typed0x78 table **0x142179988** has these slots:

| Slot | Target | Role in the established lifecycle |
| --- | --- | --- |
| +0x00 | `0x1412851c0` | Cancellation callback. |
| +0x08 | `0x141285740` | Start callback. |
| +0x10 | `0x141285ca0` | Completion callback. |
| +0x20 | `0x14127e810` | Release/destruction dispatch target; full body not expanded here. |

Complete completion **0x141285ca0..0x141285d9b** does the following:

1. If order **+0x14 mask0x8000** is set, it calls timed-event helper
   **0x141286a70(order,1)** and character notification **0x1411db640**,
   passing owner, target, skill, slot and current step index **+0xf4**.
2. It reads embedded animation-wrapper virtual+0x40/+0x48. Their concrete
   getters return wrapper **+0x2c/+0x30**, respectively. If the current
   unsigned step index is in that half-open range, it increments order
   **+0xec**. This counter does **not** decide the return value.
3. At **0x141285d4a**, it tests order flag **0x2000**. Clear returns0.
   Set dispatches **0x1411db540**, with the stored raw result at **+0xe8**
   and animation wrapper, and returns1. The tested flag is the state at
   this final test, after the earlier conditional notifications.

The complete caller **0x14127fe10..0x14127fefb** first selects the slot from
outer command bit5 and matches the behavior value and packed kind. After
calling the typed completion callback, **0x14127fed4 always clears order
flag0x100**. It calls **0x14127ff00** only when the returned EAX is nonzero.
Thus callback return0 leaves a matching order present but no longer active;
it is not proof that completion failed to run.

Complete release helper **0x14127ff00..0x14127ffa2** clears the component's
slot before calling the order's virtual+0x20 with EDX1, then updates the
character state and conditionally services a pending local order. This
establishes removal from the active slot and release dispatch. This slice
does not claim an observed deallocation or expand the entire destructor.

## The demonstrated server84 release route

These are the previously source-proven correlated helpers. All require the
selected existing order to be kind0x78 with a matching behavior value; the
outer dispatcher requires mask0x10. The raw outer slot bit is separate from
the order's own flags.

| Packed opcode | Exact effect relevant here |
| --- | --- |
| 0x44, `0x141286820` | Stores its result byte at order+0xe8 and dispatches `0x1411db500`. It does not directly set0x2000 or release. No success/error names are recovered. |
| 0x45, `0x1412868b0` | If order flag0x100 is set, sets0x2000 and returns. Otherwise dispatches `0x1411db540` and calls `0x14127ff00` immediately. |
| 0x47, `0x141286940` → `0x141287410` | Replaces step index+0xf4, may adjust timing+0xf0, and sets/clears masks0x4000 and0x8000 from its second raw byte's bits1/0. It does not directly set0x2000. |

Consequently, a matching0x45 during an active step can make the subsequent
ordinary completion callback return1. A matching0x45 after completion has
cleared0x100 takes the direct release route. This is a source contract for
those two relative states, **not a prescribed server timing policy or a
guessed complete packet body**.

The constructor starts with low packed-byte flags, adds0x4800, and optionally
0x8000; the inspected constructor/start/0x47/event-loop instructions do not
directly set0x2000. The callback therefore supplies no automatic release
merely because the action's last index or the command duration was reached.
This bounded set is not a census of every possible flag writer or reentrant
notification consumer; it does not prove0x45 is the only possible lifetime
ending mechanism. Cancellation/replacement and owner teardown are distinct.

## Start, cancellation and animation notifications

Complete start **0x141285740..0x1412858dc** sets0x1000 once and initializes
timing **+0xf0** from the clock plus an animation-wrapper getter. With0x4000
set, it dispatches a one-time **0x1411db4b0** notification if0x800 is still
set, then clears0x800. It dispatches **0x1411db5f0** for the current step,
derives the step's timing and event interval, and calls
**0x141286a70(order,0)**. The constructor's0x4800 enables these initial
notification gates without a preceding0x47.

The embedded wrapper at **order+0x88** has table **0x14217cce8**. Its
initializer **0x14129a1a0** resolves the selected action into wrapper+8,
explicitly named **`m_animationData`** by an assertion, and retains the skill
definition at wrapper+0x38. Its step accessors require the chosen index to
fit wrapper+0x54 before indexing the eight-byte entries at wrapper+0x48.
The timed event accessor **0x14129a100** reads a nine-byte record from
animationData+0x30, bounded by its Dword count+0x2c.

Complete helper **0x141286a70..0x141286cf3** iterates the event interval
**order+0xfc .. order+0x100**. With argument0, a future event stops the loop
and schedules the next time through **0x141023990** on order+0x40. Argument1
bypasses that future-time check. The event type byte selects notification
helpers; types0,1,2,3–5,7–8,9–10,12,13 have distinct dispatches, while6/11
and out-of-range types pass through without a selected notification. The
loop directly sets flag0x400 in type0 and may clear flag0x1 in types3–5.
It does not directly write0x2000. These are client animation/event dispatches,
not an established server-authoritative healing calculation.

Complete cancellation **0x1412851c0..0x141285346** always ends with character
notification **0x1411db470**. A conditional branch can temporarily construct
a local kind0x79 order: it requires `(flags & 0x1401) == 0x1000`, a successful
owner-bound virtual+0x678 predicate, and the exact unsigned timing comparison
`(order+0xf0 - clock) >= 40`. This is a local cancellation path, not a server
reply to synthesize. The downstream predicate and kind0x79 state machine are
outside this slice.

## Exact10176 content and remaining execution boundary

The [resolved animation records](209026-mesmer-heal-animation-data.md) prove
primary token **0x0000b309585954f7**, alternate token
**0x000000215a908ef7**, variant0, and a zero skill-body+0x70 flag word. These
keys are already resolved in frozen pack184788, not an unresolved lookup.
The primary variant has one segment and six ordered events:
`(4,0,0)`, `(5,867,0)`, `(1,1067,0)`, `(3,1067,0)`, `(2,1067,0)`,
`(0,1350,0)`. Its type-0 quantity derives action+0x10=1350, while the
execute-event quantity1067 supplies the independently traced
[cast tooltip](209026-mesmer-cast-tooltip.md). Neither is automatically a
server duration. The known primary action's +0x2c/+0x30 are both0, so its
unmodified completion range is empty and the +0xec increment is skipped.

The current
research0x78's packed byte12=0 selects the primary branch and does not set
0x8000. Unless later changed, completion consequently skips its conditional
event flush/`0x1411db640` branch; this is not a claim that the skill should use
another raw byte. Initial start still has the constructor's0x4000/0x800 gates.

That identifies the source data expected by the explicit selector. It is not
an observation of a particular runtime order, its modifiers, current step or
subscribed event consumers. The tooltip's locale0 path also selects the
primary pair, but that separate consumer does not prove server order state.
This slice cannot prescribe a complete0x47 sequence, declare0x44 mandatory or
choose its result byte, or prove a retail effect sequence. The0x2000 release
decision is independent of the chosen action range. A diagnostic1000ms command
duration does not equal the primary execute or type-0 quantities and is not
evidence that every animation event has fired when command completion runs.
The next missing execution boundary is the actual selected order state and
the consumers of these notifications, not token resolution. No heal amount,
recharge, retail duration or fabricated predicted0x53 acknowledgment follows.

## Lossless research codec scope

`crates/gw2-proto/src/skill_order_update.rs` adds only typed packed payloads,
reusing the existing raw server84 carrier. Opcode0x44 is exactly two bytes;
0x47 is exactly three, retaining every raw flag bit. The dispatcher
**0x1411dd4d0** requires at least one byte, and its exact0x45 branch
**0x1411de188** forwards directly to **0x1412868b0**, which reads no further
payload bytes. Thus the one-byte0x45 form is source-supported. This typed
codec recognizes only that minimal form and rejects trailing bytes instead
of discarding them; it does not claim the native handler rejects extensions.
Extended/unknown forms remain representable by the existing raw carrier.
No admission flags, correlation values, result enums, step policy, timing,
default sequence or sends are supplied by the codec.

Focused validation: `cargo test -p gw2-proto skill_order_update --
--test-threads=1` passed4 tests. They check literal payload/outer wire bytes,
all256 result bytes and65536 step/flag pairs, truncation and extension
rejection, concatenated carrier frames, unchanged carrier input on truncated
frames, and agreement with the frozen server84 schema. These are source-code
tests, not runtime evidence that a particular order accepts an update.

## Evidence

Frozen executable SHA256:
`99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`.

New private complete bundle:
`/home/blizz/re/gw2/notes/209026-typed78-completion-release-20261106.json`,
SHA256 **`bde90fb73167bdcca6798e9941c554abee36d82ab1ae5026d2f7bcb867676301`**.
All9 captures exited0 without truncation and verified owned-process cleanup.

| Evidence | Supported observations |
| --- | --- |
| `ev_31c39508bdd5cd98ff120dfdc15880cac2a9f876b457fb44b2212ee7d577a074`, `ev_d188216f12bf605dbac3ff2eeaa4f5fbd0e266b1a53b9d2689881f6482038efe` | Concrete typed/animation slots and decoded14-entry event jump table. |
| `ev_ca41d7f4750866dc0deb76941f9116661df234919f547dca7eed7c63fc1bbeb2` | Complete typed0x78 completion and its0x2000 return gate. |
| `ev_7867b8d41bb333320e60b16f45fae7e7418f876eb55b2aa9bd0626f4096511b7` | Complete start, cancellation, timed-event loop and component completion caller. Bytes after0x141286cf3 are table data, not code. |
| `ev_e52a14e917a7e09eeec7b67257e6f83a8a28b3fe459f9c4a723c644a375f8bf5` | Complete slot-release helper and animation-wrapper constructor; bounded initializer fragment with named animation data. |
| `ev_6c434f25381b71d65fae518057a4cfc172e758a58996d318918f0d3c3b4f43cb` | Exact range getters, step accessors and nine-byte event lookup. |
| `ev_5495403d8c738b5391810dd96bdb72cdd4e1102b8451f42da894d76b912a3d00` | Exact0x44/0x45/0x47 selector-table entries and direct handler branches. |

Reused constructor/server84 Evidence:
**`ev_b15d022689140fcdd8a8517a7417f13300f8eff1397f29afc61c9714b478c279`**,
**`ev_14e6f473422745215a2f8659accac9333330648cd731f2bc72deff3b2112927e`**,
**`ev_8fa8d4c9fae7a544e2680b6447b20e2830012d291450e0bd4f7572bd54b5c8b9`**,
and dispatcher minimum-length Evidence
**`ev_3f2a66e598c3972168c449554d0d69045fba7b20a2e5829429a30024ac0e3794`**,
in `/home/blizz/re/gw2/notes/209026-mesmer-skill-command-path-20261105.json`,
SHA256 `afb5c36095bf22529d0ab1a49e41d9b9585c5c5de5ec40d3a172f7b5c3501d1e`.
Initial `.pdata` fragments and partial destructor inspection remain explicitly
bounded; no global flag-writer census, notification-subscriber traversal or
runtime observation is claimed.
