# Build209026 typed78 result notification consumers

The exact result0x44 notification reaches character-listener slot **+0x368**.
The already installed ordinary-view listener and the constructor-established
skillbar interface both bind this slot to a no-op. This closes a different
edge from the [finish+0x370 investigation](209026-typed78-finish-listeners.md),
but does not identify result-byte success/error names or a gameplay trigger.
No whole-client listener census or absence of other consumers is claimed.

Static REA, 2026-11-06, frozen executable SHA256
`99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`.
Only that frozen PE and retained evidence were read. No runtime, archive,
packet, server implementation, configuration or publisher state changed.

## Typed result delivery

The retained complete handler **0x141286820..0x1412868aa** selects an existing
order from the outer slot bit, requires kind0x78 and matching behavior value,
and reads the raw byte from the exact two-byte packed0x44 payload. It stores
that byte, widened, at order+0xe8 and calls **0x1411db500** with the owner,
order+0x60 object, skill reference and raw result. This handler does not itself
set the0x2000 release flag, apply health changes or choose recharge.

Complete notification **0x1411db500..0x1411db53e** dispatches through the
character's collection at+0x1e0. Its thunk **0x1411d3f94..0x1411d3f9c** loads
the listener table and tail-jumps through **+0x368**. Complete collection
dispatcher **0x1411d0900..0x1411d0add** establishes these callback arguments:

| Argument | Source |
| --- | --- |
| RCX | Current listener |
| RDX | Owner character |
| R8 | Original order+0x60 object |
| R9 | Reference to the same skill definition |
| Fifth Dword | Raw result from the0x44 payload |

The dispatcher preserves that result; it does not translate it to a named
success/failure enum. Its listener-safe traversal and callback invocation
must not be interpreted as acceptance of the skill's gameplay effects.

## Exact known receiver tables

The constructors/registration in the existing finish-listener note establish
the two receiver tables used for this bounded check:

| Receiver | Exact +0x368 table entry | Target |
| --- | --- | --- |
| Installed ordinary AvCharBase listener, table0x142141558 |0x1421418c0 |0x140163990 |
| Skillbar+0x20 interface, table0x142165158 |0x1421654c0 |0x140163990 |

The complete target is **`ret 0x0`**. This is a no-op with zero stack-byte
adjustment; it does **not** mean that EAX is set to zero. Neither selected
callback examines the raw result or any other argument. The skillbar's
subscription to this particular character collection remains unproved, as
in the earlier note; this table observation does not invent a subscription.

These concrete consumers therefore cannot supply the missing result-byte
meaning, heal moment, cooldown policy or chain advancement trigger. The raw
result remains stored in the order and is forwarded again by the separately
traced finish route. Other dynamically registered listeners or other readers
of that stored field remain outside this slice.

## Remaining dependency

There is no new server behavior justified by this result. A meaningful next
edge requires a demonstrated non-no-op result consumer or another typed reader
of the stored result, with its receiver identity established. Without that,
authoritative execution timing/result semantics require an independently
sourced execution specification or a separately authorized known-good
accepted-skill trace. The existing raw codecs, tooltip values and animation
quantities cannot replace that evidence. This investigation did not request
or perform runtime access and adds no diagnostic option.

## Evidence

Private complete three-record bundle:
`/home/blizz/re/gw2/notes/209026-typed78-result-listeners-20261106.json`, SHA256
`f75dcdc837cefffa45ba5e2ef8d2f9fa9ffcc80da577cce8225c5476aecfa4e2`.
All new captures exited0 with nontruncated output and verified cleanup of
their owned process groups. This is not a host-wide process audit.

- `ev_ab5088d7b3ca96ada8c06a7adbe3a90d670ee4bc49e4ec928770493e884638ea`:
  exact+0x368 thunk and complete argument dispatcher.
- `ev_193ee0ea294d1cdc776519072311aa0e16e16740af747e79816255214fcafa56`:
  exact two receiver-table entries, with adjacent finish entries retained.
- `ev_75f6af188cd01413c31ac87abd74abf1ccb358c30d0960a276d1fce5a1a5adf3`:
  complete no-op body.

Handler/notification evidence is reused from the existing command-path bundle:
`ev_14e6f473422745215a2f8659accac9333330648cd731f2bc72deff3b2112927e`
and `ev_8fa8d4c9fae7a544e2680b6447b20e2830012d291450e0bd4f7572bd54b5c8b9`.
Constructor/registration provenance remains in the linked finish-listener and
character-view notes. No broad virtual-offset or callback search was run.
