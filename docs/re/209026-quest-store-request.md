# Build 209026: conditional quest startup store request

Static REA follow-up, 2026-11-01, to the
[quest startup subscriber](209026-startup-subscribers.md). This identifies
an asynchronous request on the category-failure branch. No request was
executed, no store action was taken, and the agreement hold remains intact.

## Request ID rather than a lookup result

The earlier +0xc8 service query forwards a type-68 definition's GUID to
context service **+0x1f8**, virtual **+0x1a0**. Registration
`0x14026cf50` binds factory slot **0x3f** to thunk `0x14026cf80`, which
reaches `0x140258980`. The factory allocates 0x6f8 bytes and calls
constructor `0x1402544b0`; the constructor installs primary vtable
`0x1419256e8`. Entry `0x141925888` (+0x1a0) is **`0x14025d230`**.

That method increments service Dword +0x1c8, masks it with **0x1fffffff**,
and repeats if the result is zero. It stores the resulting ID, passes the
GUID and ID to `0x14023d710`, then returns the ID. Thus, on normal return,
the result lies in **1..0x1fffffff**. It is not a stored content value and
is not an indication that the asynchronous operation completed. A bounded
constructor read observes +0x1c8 initially written zero; the first request
would be 1 if no intervening request changed that counter. The complete
constructor and all other counter users were not audited.

`0x14023d710` jumps to `0x140245ca0`, which allocates a 0x10-byte callback,
installs vtable `0x1419248c0`, and stores the request ID at callback +8.
It passes that callback with the GUID to `0x140ff9b90`. That helper
allocates a 0x58-byte request object and sets:

| Whole-object offset | Value |
| --- | --- |
| +0 | Vtable `0x1421117d0` |
| +0x10 | Request interface vtable `0x1421117d8` |
| +0x28 | Supplied 16-byte GUID |
| +0x38 | 0xffffffff sentinel |
| +0x40, +0x48 | Null |
| +0x50 | Request-ID callback |

It then calls `0x140ff6450`. This builds a request through `0x141007550`
and passes the request interface to the object at `0x14289a4e0`, virtual
+0x68, with R9D=30000. This call argument is observed; the concrete global
service and its timeout interpretation have not been re-traced here.

When this chain returns normally, the quest callback's final consumer
therefore receives a **nonzero** request ID. Its previously documented
zero-input early return does not apply to this reached path. The consumer
can still skip an existing non-null entry for that ID; otherwise it calls
`0x1402ee5a0`. The request submission and consumer tracking remain distinct
from operation completion and zone readiness.

## Named requests and response continuation

Initial builder `0x141007550` calls `0x141007650` with selector **2**,
then opens/closes the XML body. The common builder looks up selector 2
in the table at `0x14289b4b0`. Initializer `0x1402043e0` supplies three
pairs from `0x142114080`; selector 2 points to **`GetLimits`** at
`0x1421140c8`. Selector 1 is `Purchase`, but this initial request supplies
2. This initial chain is a read/query path; a later
[item-result follow-up](209026-quest-store-item-followup.md) traces a
conditional, separate zero-TotalPrice Purchase builder.

Request interface +0x20 points to `0x140ff6e60`. Its inspected body reads
XML data through its second argument's virtual +0x58 and constructs a
second request. It has branches for an explicit numeric value, a zero
GUID, and a nonzero GUID. The object constructed above has numeric
sentinel 0xffffffff and the supplied definition GUID, so its nonzero-GUID
branch copies that GUID into the second object and reaches
**`0x141002030`** to build the next request.

That builder supplies selector **1** to `0x1410029e0`. The common tail
looks up the selector in `0x14289b010`; initializer `0x140204010` supplies
two pairs from `0x142112cc0`. Selector 1 is **`GemStoreSearch`** at
`0x142112ce8`. The builder puts the supplied GUID into a one-element
array and emits fields through XML helpers. Exact XML field names,
namespace construction and response requirements remain to be documented.
The [callback follow-up](209026-quest-store-callbacks.md) traces internal
item/package events and the request-ID error event. The
[dispatch trace](209026-quest-store-dispatch.md) identifies receiving
handlers, and the [tracker trace](209026-quest-store-tracker.md) establishes
conditional error cleanup; successful empty-response cleanup remains open. These static request names do not establish an observed
wire exchange, successful response, purchase, or a required zone-load wait.

The practical bootstrap finding is conditional: when the quest subscriber
is registered, the earlier empty-refresh conditions hold, the frozen
content resolves, category 4 fails, and the +0xc8 query gate remains open,
this chain reaches an asynchronous store query. An empty quest object
alone does not eliminate that query. Runtime values and acceptance remain
unverified; no live account, credential cache or external service was used.

## Evidence

Private bundle: `~/re/gw2/notes/209026-quest-store-request-1.json`.
The service-registration helper is a byte-candidate filter; the reported
site was then disassembled from its unwind boundary. Unaligned leading
bytes in one bounded constructor read are not used. Broad builder reads
include allocation code but do not constitute a complete serializer audit.

| REA evidence | Observation |
| --- | --- |
| `ev_082801fa0a659931856e2a40aa611a6ea2b14d9a5a634c610272b62701f4e6e7` / `ev_8eca0eca617d48277ac3134579a0160e27f603aed979cf4e18c3e7041fc689eb` | Slot-0x3f candidate and validated registration. |
| `ev_9801f8fc8896f7166e735c02ec7532f40a4f33cac5efcacd22df5cd7314a4b6c` / `ev_05fb0b99e797006cd707407a8e946bb709f3d44b481ba2602089f38c539220df` | Factory thunk, allocation and constructor call. |
| `ev_6e4c837743827c79017c5a9d006faef5477156560db6b901d0c10d9eaab8d92f` / `ev_a31e9c510056fd3e2c72592f8f4b1cd78a86c70d6dcf259a989547accde73fb3` | Primary table installation and +0x1a0 entry. |
| `ev_474757cfefe873b385c3e294b8b85c498978e99ffb29e1391f23a5695f177bbf` | Complete request-ID generation and return. |
| `ev_2726a27dd6f509655f607bc85d88ce24d82996a2fdef651f3997ca149065de7a` | Sender thunk and initial counter write. |
| `ev_875b23aac0ea7b9dad2217dcab66f26291f9e36cb4f442032a3e6ec2a5b86d8e` | Callback allocation and ID field. |
| `ev_f9eda7bd58cb2434a0bf31da0172484670e354501f545d12121c87b76741d760` | First request-object constructor and submission helper. |
| `ev_5e5fb732ef1c12ec915606b3a11e5f0289a0f6faaefeb7c291afc87ff7df5102` | Initial builder call and global-service submission. |
| `ev_200ce1818094c0058e0f635dec06d412b7edb65e3dc6acb77e5ac74554b7deb4` | Request interface method entries. |
| `ev_cd4eca094fd5fa750670173e1cab6c79ab7e40fadac1d718e931099d98d61bf9` / `ev_9c6e745e00327eadfbb28bb87be28b3b12465d1c36750a0cbc77938b2171e565` | Numeric/GUID branches, XML response iteration and next builder selection. |
| `ev_3e24ae08ee7f36bf29f5b95d84e69af15423e9e61dace8c33a50515782298aa1` / `ev_9de1bc98cf29b90e571830e1dcf412346e6e03aff281e074ab10acf421136d36` | Initial selector 2 and GUID-array search builder selector 1. |
| `ev_263b19dc02cdedb92c64fe15f3e0cd99b051dd2f616fbfa4397c729fbdd414ed` | Common builder selector-table lookups. |
| `ev_90fc2f26f58c446d1daeedc266828144f679780af0cfe547ef29bd6d5610f1aa` / `ev_5ed10ec80ac8e3bb6c2227cf761bd18af8675687a52fc126ef21d14de3d890ae` | Named-table initializers and selector/string pairs. |
| `ev_c60a7643e55815628eda612ac1482d72384f64ccf176976114f28f9f41633916` / `ev_fb168336be8da469374c6edaf042742b49879739bd1f5dec1a5c9a65d47593ee` | GetLimits and GemStoreSearch string bytes. |
