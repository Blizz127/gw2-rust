# Build 209026: CPU texture work completion and lifetime

The CPU mip operation has a separate completion counter and requires another
operation-processing visit before its destructor marks the texture ready.
A completed worker alone does not set the texture readiness bit in the paths
traced here. This is frozen static evidence, not proof that a particular
run-128 texture ever acquired this operation.

The executable is `/home/blizz/gw2-client/build-209026/Gw2-64.new.exe`, SHA-256
`99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`.
See [the operations note](209026-grtex2d-operations.md) for creation-side
binding and [the model pipeline](209026-creation-model-pipeline.md) for the
four runtime samples. No client launch, attachment, working archive access,
flag change, or deployment occurred in this slice.

## Keep the three object layouts separate

| Object | Offset | Established use in this slice |
| --- | --- | --- |
| GrTex2d texture data | `+0x60` | Texture flags; ready bit `0x10000` |
| GrTex2d texture data | `+0x94` | Outstanding-work counter; zero transition removes it from context list `+0x68` |
| GrTex2d texture data | `+0xa8` | Tagged `m_opList` head |
| Mip operation, vtable `0x141bfc7e0` | `+0x28` | Retained destination texture-data pointer |
| Mip operation | `+0x30` | Job group, assertion names `m_group` |
| Mip operation | `+0x38` | Embedded job array header |
| Mip operation | `+0x40`, `+0x48`, `+0x4c` | Job-array pointer, capacity, count |
| Mip operation | `+0x50` | Number of completed jobs |
| Single mip worker | `+0x78` | Pointer to the operation's completion counter |
| DXT mip worker | `+0x60` | Pointer to that same completion counter |

The load-task fields used by the patched branch near `0x140a6f57a` are a
different object layout. In particular, load-task `+0x48` is not this
operation's job count `+0x4c`.

## Operation processing, release, and the readiness writer

The four slots at `0x141bfc7e0` are:

| Slot | Target | Observed behavior |
| --- | --- | --- |
| `+0` | `0x140a6d440` | Deleting wrapper; calls destructor `0x140a6c920` |
| `+8` | `0x140a6feb0` | Starts work if count is zero; otherwise checks completed count |
| `+0x10` | `0x1402406f0` | Returns 1 |
| `+0x18` | `0x140163990` | Empty return |

At `0x140a6feb6`, zero operation job count selects deferred startup. This
increments **texture** `+0x94`, registers the texture in the context list at
`[0x14268b840]+0x68` on transition to one, creates a group with
`0x140dd4340`, and calls `0x140a74e40(texture, group, operation+0x38,
operation+0x50)`. It then returns zero. `0x140a75110` performs the same startup
for the immediate-start constructor path.

For nonzero job count, `0x140a6ff37..0x140a6ff47` returns precisely whether
`operation+0x50 == operation+0x4c`. There is no separate readiness-bit write
or successful empty-work return in this inspected processing method.

The normal per-texture processor `0x140a6ff50` checks operation virtual
`+0x10`, preserves ordering against overlapping earlier rectangles, then calls
virtual `+8` at **`0x140a70026`**. A zero result leaves the operation queued.
A nonzero result calls virtual zero with delete argument one at
**`0x140a70038`**.

Destructor **`0x140a6c920`** explicitly gates its completion cleanup on
`operation+0x4c != 0` at **`0x140a6c92f`**. Zero jumps to `0x140a6ca04`,
skipping the group/job cleanup, texture outstanding-count decrement, and ready
writer. Nonzero count calls group cleanup `0x140dd4380`, releases each job via
`0x140dd4870`, decrements texture `+0x94`, and removes the texture from the
context list when that counter reaches zero. It then sets texture
`+0x60 |= 0x10000` at **`0x140a6c9fd`** if the bit was clear. A previously
ready texture with an empty operation list skips the redundant OR.

Both branches release the retained texture reference, free the job-array
storage if present, and unlink the operation. Thus this destructor is not a
general empty-work readiness fallback. The only direct incoming reference
found by the exact-address query was the deleting wrapper at `0x140a6d44f`;
the virtual call above supplies the ordinary completion route.

## Who records completion

`0x140a74e40` reads texture dimensions `+0x48`, format `+0x64`, and mip count
`+0x68`. It sizes the operation job array using `0x140b4fcb0`, then calls
`0x140b4fd20`, passing the group, jobs array, returned-job-count pointer, and
operation completion-counter pointer. Assertions name `group`, `jobsOut`, and
`jobCountOut` in `Engine/Gr/Img/ImgMip.cpp`. The wrapper clears the completion
counter and calls `0x140b4f8c0`.

The ordinary asynchronous branch creates one job through `0x140b50190` and
sets the returned job count to one at `0x140b4fc7b`. That job installs callback
**`0x140b4f820`** at `0x140b50217`. After calling mip work `0x140b4f580`, the
callback writes one through its `+0x78` completion pointer at
**`0x140b4f863`**.

The specialized path for formats `0x16`, `0x1a`, `0x1c`, and `0x1d`, with
base-level area at least `0x1000`, sizes jobs using `0x140c21580`. The producer
uses `0x140c21a80` / `0x140c21b80`, with the source string `ImgDxtMipPush`, and
installs callback **`0x140c21700`**. Its common tail reads worker `+0x60` at
**`0x140c21a2d`**, then atomically increments the pointed completion counter
via `0x1409b2050` at **`0x140c21a56`**. The helper's `lock xadd` and increment
are verified. Both worker constructions submit through `0x140dd43a0`; the
JobManager scheduler and its runtime activity were not traced here.

There is a narrow zero-job edge in the count helper: `0x140c21580` initializes
the total to zero and returns it when `levelsProvided >= levelsRequested`.
The caller here supplies one provided level. This could matter only if the
specialized path is selected with no additional requested mip levels; none of
the run-128 samples supplies the required texture format/dimensions/mip count
to establish that situation. It is not evidence of a zero-job bug in the
observed textures. The inspected operation code would treat a zero count as
startup again, rather than completing it by this destructor.

## Servicing callers and the remaining dependency

An exact image-wide query for `0x140a6ff50` found and validated only these
direct calls in the queried encodings:

- `0x140a6e723` and `0x140a6e732`, in context advance `0x140a6e6d0`.
- `0x140a70193`, in dependent-texture visitor `0x140a70150`.

The latter visitor walks the source texture's dependent array, processes each
dependent texture, then clears the array. Its two exact direct callers are
**`0x140a70415`**, in a resource-replacement/error path that also writes ready
and error-related flags, and **`0x140a704e4`**, after successful texture
transfer in `0x140a70450`. The existing operations note separately records
the unresolved upstream caller of context advance `0x140a6e6d0`.

This proves a post-resource-transfer processing route and a context-advance
route. It does not establish a current frame callback, their frequency after
loading, or that either has serviced an observed scratch texture. It also
does not make cleanup/drain equivalent to normal completion.

The next discriminating observation is to match a sampled scratch texture to
its actual operation at constructor `0x140a71e80`, then retain operation
`+0x4c` / `+0x50` and its processing/deletion outcomes. The cases are distinct:
operation not yet created; created but work not started; workers incomplete;
workers complete but operation not processed; or destructor reached without
the expected texture transition. Three empty runtime operation lists with a
clear ready bit do not by themselves select a stalled-worker explanation.
No unconditional readiness override follows from these findings.

The lifecycle-probe author received four bounded candidate hooks. At
`0x140a75110`, RCX is the operation before immediate startup. At
`0x140a74fbe`, after work production, R15 is the texture, RDI is
`operation+0x38`, and R13 is `operation+0x50`; verifying those relations,
the operation vtable, and destination identity permits count observation.
At `0x140a6ff37`, RCX/RBX is the operation after the nonzero-count gate.
At `0x140a6ca04`, RDI is the operation after its conditional completion
branch; this site also admits the zero-count bypass, so it must not be
labeled as proof that the ready-bit writer executed. All hooks require a
match to one of the already-bound scratch-texture identities and finite
transition caps. No probe was run by this investigation.

The exact sixteen-byte guards are retained separately in
`/home/blizz/re/gw2/notes/209026-grtex2d-cpu-work-hook-guards-20261105.json`,
evidence `ev_1f21148e182292ad37a72b7b6f620ebea388fcb22b6dbb7561c357a1449505c8`.

## Evidence and bounds

Complete private export (18 REA records):
`/home/blizz/re/gw2/notes/209026-grtex2d-cpu-work-completion-20261105.json`,
SHA-256 `69e04e97bb7c120ef9b25c43c42485d5649c2a110ac60390730a452100bb4c34`.
All captures exited zero, retained complete raw/normalized output, and verified
owned-process-group cleanup. Some records list unrelated `/proc/*/environ`
permission failures in process sampling; those processes were not touched.

| Finding | REA Evidence ID |
| --- | --- |
| Destructor and exact deleting-wrapper reference | `ev_0f7fe518e213a7cd5ef32393d98365eeeee4f9a199c4d70346477b5ba27d7ac8`, `ev_3dde2eeee4d58b4b03d609dbc2ab6219f115a8e02ead19bd66278feb67753545` |
| Operation table, startup helper, wrapper | `ev_3eb35eed29ba87c1f8685474da96357d511da94ba5cbf8e0242bd330234f57c4`, `ev_62c41c31b086c6e23d49d8be86943453cf47c423fed8de42c4a5ba30fd472bba` |
| Complete processing and normal deletion paths | `ev_a2ac7766cff5130d3eb7d9aae41d643f2b36dc16ff36b2e6790d53a1fe1e3de0` |
| Mip job count and producer | `ev_72c30d76637fe894cb2b96cb8591207412f7edd53df0a5cc39ff19e1668356ba`, `ev_5dfdc3b261ae2192b7f533889234f49d084a6f6c8a6eaeab0d4652e96e549c7c` |
| Exact normal-processing and dependent-visitor caller queries | `ev_62085e2b4d69edfe528a6563f22bf4d22844242dfe499d5e9eee6e45eda12dea`, `ev_fa382d8b9abc8284fd0d923924b9c856c23d36ae8cd09a84dd75cb582fc4bc4e` |
| Resource-completion callers and job callback construction | `ev_e6ddca8504fd69255df4dfc6ac6b62c408225d05796880bd9646b0e4ecff9804` |
| Single-job completion callback | `ev_645da771e99689d1ac237dd8e61528088b9b30ad8965aac68d11b05d87e283e6` |
| Specialized count, worker construction, dependent traversal | `ev_b530f7506db778a4310213688f81f0f43193fd9c40aacfe4352827250ce90430` |
| DXT callback completion tail and atomic increment | `ev_ab0445dfa87e0f0950175c3a1e229f0ffac57222a02752ce3f72c25c24c4c690`, `ev_e87f50c9f1bd9aa8ebd7f25d826fa4e95bdd57c537ca54ccebbc0f186895dfbf` |

The existing hash-bound inventory helper uses `.pdata` chunks, which can end
inside a logical function. Selected bounded disassembly windows were used to
follow demonstrated branches through their returns where needed; incomplete
initial chunks are retained in the bundle and are not treated as complete
functions. Exact-reference queries covered executable E8/E9 and REX.W
RIP-relative LEA encodings plus aligned absolute Qwords in readable
non-executable sections, validating every returned code candidate at a decoded
instruction boundary. Their negative results exclude other encodings,
indirect construction, and runtime callers.
