# Build 209026: conditional persistent empty-composite runner plan

This is a source-only implementation plan, contingent on the private
diagnostic result. It changes neither the frozen run-133 helper nor the
runner. Its scope is the exact concrete `0x141bfc7c0` operation started by
`0x140a75020`; it is not a general job-completion or texture-readiness bypass.
The [repair contract](209026-empty-composite-repair-contract.md) retains the
fresh allocation, no-pixel-work, and normal cleanup proof.

## Smallest maintained change

Add a self-contained helper to `tools/gw2-runner/gw2-runner.gdb`, armed by
the existing `Arm` method after image, inferior, and relocation verification.
Use one public plain GDB breakpoint and its command list, not a Python
`Breakpoint.stop()` subclass. The existing native fixture established that
the plain breakpoint honors native conditions before invoking Python.

Prefer **`0x140a74ced`**, the first instruction of the empty-only block.
The existing `test eax,eax` at `0x140a74ce5` and `je 0x140a74da7` at
`0x140a74ce7` send nonempty rectangles around this site. Here EAX remains
one, RDI remains the destination, R14 remains the primary source, and the
validated frame remains intact. Its first complete seven bytes are
`498b56584885d2`. Keep the existing full prologue, caller, and branch guards;
do not substitute these seven bytes for them.

Do **not** transplant the current register predicate to the later empty
jump at `0x140a74da5`: the intervening diagnostic writes EDI at
`0x140a74d25`. The retained complete body establishing both facts is REA
`ev_c74507ae63057828be2e4558b106143d228ce89cc3ffba87f823832445eb19d5`
in `209026-grtex2d-multi-source-blit-20261105.json` under the private notes
directory. This is a source-derived hook improvement, not an observed
run-133 hook; that frozen helper still uses `0x140a74ce5`.

The native condition should require EAX one and exact saved return address
`[RSP+0x348] == 0x140a750f2 + slide`. The callback then checks the bound
inferior, x86-64 architecture, all-stop mode, image/code bytes, and every
existing same-call operation/source/destination/group/job, empty rectangle,
fresh-state, and bounded backing/readiness predicate. Recover the current
operation from `[RSP+0x340]`; never use a cached operation or assume it is
the destination's list head. Repeat the complete capture immediately before
the sole four-byte `job+0xf8: 0 -> 1` write, then verify readback.

Remove creator UI ownership and all diagnostic admission/time/write limits
from this generalized variant. Retain no per-operation pointers after the
callback. A fresh current-call frame and an unsubmitted group/job establish
the allocation at the instant of mutation; cached lifetime authority is
unnecessary. Keep a few aggregate counters and a bounded number of constant
status lines. Exhausting the log budget must never stop subsequent repairs.
Optional poll/destructor tracing remains diagnostic and separate from repair
admission. No private notes path may become a shipped dependency.

Unknown or unqualified state gets no write. Code/installation guard failure
should explicitly skip this feature. A write attempt with failed readback
must report an uncertain mutation and use the existing controlled runner
failure path, rather than claim a harmless skip or retry that allocation.

## Generalization and alternatives

The no-work branch and the fresh group's normal release are properties of
this concrete operation and starter, independent of creator UI identity.
Keeping those predicates permits the same repair in later previews and
world composites. That extension is source-derived; run 133 alone cannot
demonstrate every world or UI caller. It preserves synchronous/null-job,
submitted, nonempty, differently typed, and unknown cases unchanged.
It also preserves the original rectangle calculation, diagnostic, worker
producer, operation cleanup, and ordinary later texture-ready writer.

No maintained in-place instruction patch satisfying these dynamic guards
has been established. The two-byte empty jump cannot hold the guarded job
store. Reusing live diagnostic instructions would remove behavior and need
new relocation analysis; a new code cave or executable allocation would add
an unsupported dependency. Patching the generic allocator or completion
getter would broaden the change to unrelated job types. The single
empty-only breakpoint is therefore the smallest currently supported runner
change. GDB still incurs a stop on qualifying empty work, but nonempty
composites do not reach this breakpoint.

## Focused validation

Host fixtures should retain all existing rejection/write-failure cases and
add more than 128 fresh operations after the logging budget is exhausted,
without UI-owner state or elapsed-time admission. Assert every qualified
operation gets exactly one four-byte completion write, and all other data
and nonempty cases remain unchanged. A tiny owned native fixture can check
the chosen plain-breakpoint condition/command-list wiring; the existing
GDB 16.3 evidence already verifies original-byte reads under enabled
software breakpoints.

One combined frozen-client smoke should cover a visibly rendered default
Norn Mesmer preview, the actual saved creation request and result
`415 -> 1048`, transfer `430 -> 1065`, then the same saved GUID entering
tutorial 379 through the shared mission listener. Verify the first world
player model/equipment renders and ordinary movement works. Use existing
constant server milestones and durable record counts, without names or raw
requests in logs. Preview success alone does not establish world rendering;
the bookkeeping write/poll/destructor chain alone does not establish visual
success. This is the requested combined validation, not a new separate
approval or repeated passive-run prerequisite.
