# Build 209026: quest-step objective binding and remaining trigger boundary

Static follow-up, 2026-11-06, to [quest-step state transport](209026-quest-step-state.md)
and [quest154 content](209026-quest154-content.md). The HUD consumes a step's
server-supplied **runtime objective ID** through a separate objective context.
This establishes a concrete state/display contract. It does **not** establish
which action completes any Great Hunt step, or that any of its six steps uses
a counter objective. No gameplay, codec, fixture or live configuration changed.

## Step state reaches the quest HUD

The exact `OblQuest.cpp` assertion reference identifies row construction at
**0x14074c550**. The function asserts `!step->IsNoUi()` and then:

1. Calls step virtual **+0x30** at **0x14074c5f2**. The previously established
   step table **0x142165f20** binds this to `0x14088e190`, returning runtime
   step **+0x14**, named `m_objectiveId` by the 545 assignment assertion.
2. Calls **0x141439d10**, which returns **game context +0x1d8**. At
   **0x14074c5fc–0x14074c607**, it passes the objective ID to that context's
   virtual **+0**, retaining the returned pointer.
3. Calls step virtual **+8** to test completion. For an **incomplete** step
   and a nonnull objective result, it constructs the widget input with that
   objective pointer, raw discriminator **0**, and raw Dword **2**. Otherwise
   the input has discriminator **1**, the UI's quest pointer and step pointer.
4. Passes this input to widget creation **0x1410708c0**, with callback
   **0x14073a050**. This trace does not assign semantics to raw discriminator
   2 or identify every objective widget kind.

Thus a quest/step row has a path even when no objective is resolved. An
objective ID is not a content species key, a step ID or a target count merely
because those fields share the Dword encoding. The exact context virtual+0
implementation/table was not independently resolved in this slice; its
receiver, ID argument, returned pointer and HUD selection are established.
The named `ObCliContext.cpp` registration/update paths below independently
establish the same context's ID-to-runtime-object registry.

The existing **544** helper replaces runtime flags and objective ID; **545**
assigns an objective ID only when the existing ID is zero. Neither recovered
binding helper creates the objective. **542** independently marks the step
complete and evaluates goal completion. No new requirement to create a
counter before completing an otherwise valid step is inferred here.

The actual HUD also uses localized-text wrapper **0x1412e38e0**: goal +0x40
at **0x14074c9a9–0x14074c9c1**, and step +0x48 at **0x14074c78c**. This
extends the earlier diagnostic-only consumer proof; it supplies no text keys.
The encrypted IDs and missing-key boundary in the content note remain intact.

## Exact visibility predicate

Step virtual **+0x18**, **0x1411ffb30–0x1411ffb66**, returns true when any of:

- `IsNoUi()`: definition +0x10 bit4, via virtual +0x20;
- runtime +0x10 bit2;
- runtime +0x10 bit1, via virtual +0x10.

The incomplete-step pass in **0x14074c8f0** calls that predicate at
**0x14074cabc** and adds the row through `0x14074c550` only when it is false.
The predicate is distinct from runtime bit0, the completion bit. No name for
the individual runtime bit1/bit2 gameplay meanings is introduced.

Quest154's retained six definitions, in array order **1, 2, 3, 6, 4, 5**, have
definition flags **0, 2, 2, 2, 2, 2**. Goal construction produces runtime flags
**0, 0xa, 0xa, 0xa, 0xa, 0xa** and objective IDs zero. Therefore the five
initial `0xa` states satisfy this HUD exclusion predicate through bit1,
while step1's initial zero flags do not. This explains an initial visibility
difference; it does not establish when a server reveals another step or the
retail order of the six actions. Content evidence remains
`ev_38781654b454545ebff182fb280ca8f64e3d035f3abc51bc5a1f78d2367232c0`.

## One supported objective family: counter state

The extracted objective schema `table-0x1422e03b0` binds the following exact
handlers. These frames have **no player index**; their leading Dword is the
runtime objective ID. This is one existing objective family, not a recovered
Great Hunt choice.

| Server ID | Wire fields after the message ID | Exact path |
| ---: | --- | --- |
| **1090** | Dword ID, String[8], Dword flags, Dword target, Dword initial, Dword raw value, Dword type-244 key, Guid, Dword type-316 key | `0x1414425e0` → `0x14143abc0` → counter constructor `0x14143e4b0`; registration `0x141439b30`. |
| **1089** | Dword ID, Dword count | `0x1414425b0` → `0x14143aa90` → `0x14143e890`. |

Dwords retain the existing unsigned varint wire convention. Decoded-memory
offsets in the handlers are not variable-length wire offsets. The two
content keys are separate from the runtime ID; 1090 resolves the type-244
key through content virtual+0x58, and the constructor conditionally resolves
the nonzero type-316 key through that same lookup slot. The Guid and raw
value are passed onward; no default or Great Hunt semantics are assigned.

For a counter allocation **C**, constructor `0x14143e4b0` establishes:

| Counter-relative field | Established source |
| --- | --- |
| +0x68 | String[8] copied by `0x1409b8660`. |
| +0x78 | Current count, initialized from the initial-value field. |
| +0x80 | Raw Dword; meaning unassigned. |
| +0x84 | Raw flags. |
| +0x88 | Start value, also initialized from the initial-value field. |
| +0x8c | Target count. |
| +0x90 | Result of the conditional type-316 lookup. |

Constructor assertions require flags bit3 clear (`!IsScaling()`); for flags
bit8 clear the target must be nonzero, and for bit8 set the start value must
be nonzero (`IsCountingDown()` names that bit's behavior). The base-object
constructor at `0x1414395e0` lacked a recovered `.pdata` bound and was not
interpreted here.

Registration **0x141439b30** requires the ID not already present, inserts
an entry into the context array at **+0x10/count +0x1c**, and stores the ID
at entry +0 and the objective-interface pointer **C+8** at entry +8. Entries
have stride **0x10**. The counter uses primary table **0x1422e0f38** at C,
and objective-interface table **0x1422e0fa8** at C+8. The latter's virtual
**+0x158** is `0x140976b30`, returning receiver minus8, so a snapshot of an
objective-interface pointer must not read counter offsets without adjusting
that pointer.

For 1089, helper `0x14143aa90` asserts a nonzero ID, an existing registry
entry and nonnull objective, then calls objective virtual+0x158 and asserts
the result `objectiveCounter`. The update writes C+0x78 and validates:

- Counting up: start ≤ count; count ≤ target unless flags bit6 is set.
- Counting down: count ≤ start; target ≤ count unless flags bit6 is set.

The bit6 test is the exact primary virtual+0 leaf **0x14143e820**. All
comparisons above are unsigned. `0x14143e890` then calls the notification
helper `0x1414399c0` on **C+8**; this slice does not trace every registered
listener. It does not establish that reaching the target completes a quest
step or supplies an authored trigger.

The repository already has a restricted `RenownFrame209026::CreateCounter`
and `SetCounter` encoder in `crates/gw2-proto/src/renown.rs`. Its plain-counter
flags and empty/zero auxiliary fields are the existing heart fixture policy.
They must not become Great Hunt defaults without corresponding evidence.

## Remaining action-to-step dependency

For each actual Great Hunt transition, the missing source contract is still
the authored **action/actor/condition → step ID** rule and, if the step uses
one, its objective kind, content fields, target/start values and runtime-ID
binding. A protocol create/update/bind path can carry known state; it cannot
recover that rule. Neither sector links nor array order, generic wolf kills,
heart56 completion, arbitrary counts, timers or UI visibility supply it.

The bounded diagnostic and navigation consumers inspected here read
completion/visibility or marker state, not a demonstrated authored-action
rule. Their partial fragments are not evidence that such a rule is absent
from every shipped asset or code path. No archive was opened. The next
useful source input would be a specific authored per-step rule/objective
reference or authentic state establishing that rule; there is no justified
new encounter trigger to wire from this slice alone.

## Evidence and coverage

Every query asserted frozen executable SHA-256
`99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`.
REA ran read-only hash-bound Python/objdump helpers, not the game. The one
exact-string xref pass validated 14 references to `OblQuest.cpp`; it was
limited to its stated instruction encodings and did not enumerate all
indirect consumers. No Ghidra import, working archive, container or client
runtime was used.

| REA Evidence | Observation |
| --- | --- |
| `ev_edc35ee0811ca8ff99c543d975bff69704158b8c018dbdb6b04347ef7dd17305` | Exact `OblQuest.cpp` reference anchors. |
| `ev_f664f7414fdf58ee6dc8f97e12e8e6f7a045b5f94a4cafffad6a46657cd34ae1` | Quest HUD row creation, goal/step iteration, text and listener fragments. |
| `ev_eb79fe185a11c27c74812fc7ebf5b547e2ff2373c887b27af8177ea9dfe3905a` | Context getter, ID registration and complete step visibility predicate. |
| `ev_5c7fdd10956687a122d39234a7d597268ae346aa3158d3770b9cbe0c4a24ab45` | 1090 handler and creation/registration wrapper. |
| `ev_e806ad5f9c94c0c9a2411fb461460e4da9463260e0a8edb555bfa40015ea4063` | 1089 handler, counter constructor and count validation. |
| `ev_a7d2f8d4d66835c5797c0be4f543c8dc07eeed4d7c28e35687aab678a6b13162` | Typed counter lookup, update and notification helper. |
| `ev_7d09d67c864dfded996139a103cc64f83327e398224fe8d86315e623bb595344` | Constructor-established counter table slots; `.pdata` leaf-bound limitation. |
| `ev_c68ef3b3016db1cc8314596563f3d2a26d8cf4058e3e9c454aae23fe651f69ff` | Exact 32-byte leaf windows; interpreted only through each first return. |

Complete captures, including the diagnostic/navigation reads and limitations:

- `~/re/gw2/notes/209026-quest154-objective-ui-boundary-20261106.json`,
  SHA-256 `0ba15b623f38124f796785280c2e711271aab2e74cc0647b7387a957acc4e704`.
- `~/re/gw2/notes/209026-quest154-objective-counter-leafs-20261106.json`,
  SHA-256 `21bf4cf20e99d17407d6e68750bcf3c12d4ed1a310518b76c4adee6379218cc2`.

All captures exited zero and REA verified their owned process-group cleanup.
The xref capture omitted some cumulative rendered-terminal frames; all 16
raw frames/2872 bytes and normalized frames were retained. Some captures
report unrelated `/proc/.../environ` inspection permission failures in their
residual process observations; those limitations remain in the bundles.
Raw source data and disassembly remain private. No runtime or gameplay
validation is claimed, and no code tests were necessary for this note.
