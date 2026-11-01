# Build 209026: endurance deficit bookkeeping

Static REA follow-up to [health/endurance](209026-character-health-endurance.md)
for the same frozen executable. This identifies local bookkeeping separately
from the current-points field. No client launch or packet replay occurred.

## Fields and gates

The constructor `0x141222b40` copies current points (+0x10) to tracked
balance (+0x20), clears tracked deficit (+0x24), and derives flags +0x30
bit 0 from the named `CharEnableEnduranceDeficitTracking` option. The
literal is at `0x14216af58`; the option lookup receives second argument 1.
This trace does not read an owner's configuration or establish its runtime
value. The option-change callback `0x141223180` receives the component's
+8 interface and updates interface +0x28, the same whole-component +0x30.

Primary endurance vtable `0x14216adc8` has:

| Virtual offset | Function | Observed operation |
| --- | --- | --- |
| +0x50 | `0x141222dd0` | Predicate on the projected tracked deficit. |
| +0x68 | `0x1412230c0` | Consume one segment in tracking fields only. |

Both return without the operation when flags bit 0 is clear. Both also
consult setting descriptor ID 42 (`0x2a`), expected content type 64. If
an owner exists, they call owner +0x40 interface virtual +0x50, then that
returned object's virtual +0x58 with the looked-up skill pointer. A true
result suppresses the predicate/consume operation. The full meaning of
that owner-side predicate remains untraced. For the established character
+0x40 vtable, its +0x50 getter is `0x140400100`, returning interface
+0x90 (whole-character +0xd0).

The type-64 lookup is concrete: descriptor `0x1425e7550` is initialized at
`0x14020cb81` with ID 42, expected type 64, and these four little-endian
GUID Dwords:

```text
d6b817e8 485a1bd0 c5c61fb2 87db251b
```

A hash-checked archive helper searched types 64 and 150 in all 32 base
containers. Exactly one matching record exists: file **1282857**, content
offset **4804808**, direct type **64**, UID **69789**, data key **783**.
The lookup does not need an intermediate type-150 setting on this frozen
archive. No human skill name is inferred from the numeric identifiers.

Generic resolver `0x1412dde50` accepts either the requested direct record
type or a type-150 record whose declared value type matches. For a setting
wrapper, these consumers use `0x1412e3d60`: variant tag 64 returns its pointer
at +8, otherwise null. Missing/wrong record paths are not treated here as
proof that an absent exemption is safe in every caller.

## Arithmetic and reset behavior

Let `B` be Dword +0x20, `D` Dword +0x24, `S` segment divisor +0x1c and
`M` effective maximum +0x18. After the gates above, +0x50 returns:

```text
M == 0: false
otherwise: f32(i32(wrapping_u32(D - B + S))) / f32(u32(M)) > 0.5
```

`0x141924c58` contains float bits `0x3f000000`, exactly 0.5. The numerator
uses 32-bit subtraction/addition and signed `cvtdq2ps`; the denominator is
zero-extended before conversion to float. The comparison is strictly
greater, so equality at one half returns false. The zero-maximum path
compares 0.0 with 0.5 and also returns false.

The consume operation computes `R = wrapping_u32(B - S - D)`, then branches
on R's sign bit. If clear, it stores B=R and D=0. If set, it stores B=0
and D=wrapping_u32(-R). It changes neither current points +0x10 nor either
maximum. In ordinary small nonnegative ranges this records the shortfall
after subtracting one segment; extreme Dwords retain the observed 32-bit
wrap/sign behavior rather than mathematical unbounded subtraction.

For illustration only, B=50, D=0, S=50, M=100 gives predicate false and
one consume produces B=0,D=0. A further predicate is still false at
exactly 0.5; another consume produces B=0,D=50, after which the projected
ratio is 1 and the predicate is true. These are arithmetic examples,
not selected bootstrap values or confirmed action timing.

Wire setters 219/220/221 reset B to the resulting current points and D
to zero, as already traced. The local signed-delta adjuster
`0x141222ce0` also resets them after changing current points. Its percentage
wrapper `0x141222ca0` delegates to that adjuster. No separate clock-based
recovery is established here. These findings do not rename S as a dodge
cost or prove which input/action calls consume.

## Evidence

Private bundle: `~/re/gw2/notes/209026-endurance-deficit-1.json`.

| REA evidence | Observation |
| --- | --- |
| `ev_1c8d8af8364a81af7e93f970be8b1951631013c95a733beac8b65a3905d80c7e` | Primary and adjacent +8-interface vtable entries. |
| `ev_b8631090cbca68b54507bce734a2aac78f9833b30c5d2be8886d22cfb6a088b7` | Complete predicate, option callback, direct setters. |
| `ev_0c65a15005573d07a9ea3e6e98552a985ca11f5a647062440931da23e190f75a` | Complete consume operation, constructor and local delta/percentage paths. |
| `ev_2198196271c4e323c336b65e74a934c09ed9bc2f217fd0c4ff30f44f612aa5ac` | Half-threshold bytes. Only the first command produced output; no string or call-search result is claimed from this record. |
| `ev_7e8653fa0a0787bfd2c70906222d61b1d59c7fa2ce70780944f8c5e8b1fb8dfc` | Named option bytes. |
| `ev_2b755272870554a1401a37e1ba47a515fd44902117f7544e807df572c460020e` | Character +0x40 interface's +0x50 getter entry. |
| `ev_f8f55a9f1052013931e71b02dfc8155e1766a45261397aa03436540b24bb73bf` | Getter and complete generic direct-record/setting lookup. |
| `ev_e7525b7c5f801d9d0230f1c54c126d58ac1f6781870ce20891ea82d1cd2623d5` | Descriptor initialization and variant pointer reader. |
| `ev_78c70bfcdf3d36b230792695636b1626d81143bcc63c4365f6d42ea7d949927b` | All-32-container GUID resolution to the unique type-64 record. |
