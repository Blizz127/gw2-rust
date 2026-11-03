# Build 209026: what STATE_MAP_DOWNLOAD waits on (static, two rounds)

By a read-only helper on 2026-11-03, after the owner's b114 run stopped in MAP_DOWNLOAD (runner stage trace, 21:18:30Z). Round 1 has been superseded where Round 2 says so. The lead checked the loader side (`0x1409520b0`, `+0x218` created at `0x1409536fe`, factory `0x1417e9630`). Local run 77/78 (runner probe) confirmed that the downloader slot `0x1427fad70` is set at the first connect and that registration `0x140d88140` runs twice while the three manifests are fetched.

**Device versus local:** on the owner's device (Local.dat persisted since the failed 14:22Z run) the client never fetches manifests 2130706510–12. Every local run starts with no Local.dat and fetches them. b115 sets his Local.dat aside once to test this.


Method: rizin disassembly of the frozen Gw2-64.exe plus a PE scan for xrefs. No game run, nothing written to the repo.
Tags: **[V]** verified in disassembly at the cited address. **[I]** inferred.
Sections: `.text` VA 0x140001000, `.rdata` 0x141914000, `.data` 0x142554000.

## 0. Corrections to the handoff (important)

1. The request vtable `0x14235d6d0` was read one row off.
   Raw dump (**[V]**):

   | Offset | Address | Role |
   |---|---|---|
   | +0x18 | 0x1417ea8a0 | |
   | +0x20 | 0x1417ea9d0 | pump |
   | +0x28 | 0x1417eab30 | |
   | +0x30 | 0x1417eaa90 | |
   | +0x38 | 0x1417eac90 | SUM of children's +0x38 |
   | +0x40 | 0x1417ead50 | SUM of children's +0x40 |
   | **+0x48** | **0x1417eae60** | **AND of children's +0x48** (all children complete; returns 1 if there are no children) |
   | **+0x50** | **0x1417eadd0** | **OR of children's +0x50** (any child failed) |

   - MAP_DOWNLOAD calls +0x20, then +0x50 (nonzero → state 6), then +0x48 (zero → `ret`, i.e. wait). **[V]** 0x140952139-0x140952160, wait exit at 0x140952620.
   - So the wait is "not every child is complete". It is **not** "the sum of +0x38 is 0".
   - `[rbx+0x218]==NULL` goes to state 6 ("Completed STATE_MAP_DOWNLOAD with error") at 0x14095262c. **[V]**
2. The 10 s call `0x140d8f620(0)` forwards to `s_mainArchive` vtable +0x20. **[V]**
   - `s_mainArchive` is `[0x1427fadc0]`. The assert string is "s_mainArchive" at 0x140d8fd6f, and the setter is 0x140d8fcb0.
   - If `s_mainArchive` is null the call silently does nothing.
   - It is not a downloader kick.

## 1. What the children are

**Children = `PoDownloader` objects** (Engine\Map\PortalLoader\PoDownloader.cpp, string 0x14235d730).
- Vtable `0x14235d620`, ctor `0x1417ea6d0`, 0x20 bytes. Layout: +0x0c = manifest id, +0x10 = `m_preparer`, +0x18 = "started" flag. **[V]**
- `0x1417e97e0(factory, id)` get-or-creates one per id and caches it in a dictionary at factory +0xb0. **[V]**
- `m_preparer` comes from `0x140d93610(id)`, a lookup in the global dictionary `s_manifestPreparers` (head 0x1425930f8, table 0x142593100).
  - A missing id is an assert, not a wait: "preparer", MultiManifest.cpp:988. **[V]**
- Child slots: +0x38 → 0x1417eac50 and +0x40 → 0x1417ead10. These read the preparer's +0x58 stats struct {+0xd0 bytes done, +0xdc files OK, +0xe4 files total}. **[V]**
- Child +0x48 → thunk 0x1413fe150 → preparer vtable +0x40 = `0x140d92440`, which returns `[+0xdc] == [+0xe4]` ("all files OK"). **[V]**
- Child +0x50 → 0x1413fdfd0 → preparer +0x48 = `0x140d92460`, which returns `state == 6` (failed). **[V]**
  - State 6 vs 5 is chosen at 0x140d93ac1: `neg/sbb/add 6` on the result of +0x40. **[V]**

**The two ids** come from `[factory+0x30]`.
- `[factory+0x30]` is the sub-object at MapPortalLoader+8 (vtable `0x1422dbc28`, MpQuery.cpp). It is set at 0x1417e9f78 and 0x1413fe0fa-0x1413fe108. **[V]**
- Its slot +0x10 = `0x1413fe2a0`. It builds the wide string `"Map" + wcs[mapDef+0xb0]`, then calls `0x140d92d00(name)`. **[V]**
- Its slot +0x18 = `0x1413fe340`. It builds `"Region" + wcs[mapDef+0x110]` and calls the same function. **[V]**
- `0x140d92d00` scans the root manifest `m_root` (`[0x1427fb0e0]`, the ARMF chunk) for the entry whose name (`entry+0x10`, 24-byte entries) equals the string.
  - It returns that entry's id (the first dword), or 0 if none matches. **[V]**
- If the Map id is 0, the factory returns NULL (0x1417e972a). That leads to the state-6 error, not a wait. **[V]**
- So the ids are **manifest ids** looked up by name in the root manifest. They are not file ids.
- The mapDef strings at +0xb0 and +0x110 are not decoded here. [I] they are the map's content-name and region name.
- Root manifest: ApManifest.cpp loads the 'ARMF' chunk at 0x140fdd8ef/0x140fdd956 and calls `0x140d93070` to build one preparer per entry. **[V]**
  - Each preparer is built by `0x140d913a0(id, entry+0x10)`. **[V]**
  - Initial state is 1 (0x140d9161c), error code 9 (0x140d91623), and +0xdc = +0xe4 = 0. **[V]**

## 2. What makes a child complete (preparer state machine)

Preparer class: vtable `0x1420a7368`, size 0xf0, state at +0xc8.

**Child pump** `0x1417ea980`: **[V]**
- If `!started` and `preparer.vt+0xa8` (0x140d92620: `state >= 4`) is true, it calls `preparer.vt+0x70(1)` (0x140d91c70) and sets `started=1` (0x1417ea9af). The flag is set regardless of what +0x70 does.
- Then it always calls the preparer tick `vt+0x88` = `0x140d91950`.

**Tick `0x140d91950` by state:** **[V]**
- **State 1**
  - `m_archive` = `s_mainArchive` (0x140d8f560, which dereferences without a null check). Then `0x140d91fc0` runs.
  - `0x140d91fc0` sets state 2 and calls `0x140d943b0(manifestId,...)` to look up the manifest's file id.
    - If that lookup fails it logs "Manifest 0x%08x is missing." and reports a fatal error "Manifest is missing." (0x140d92047-0x140d9206b).
  - It allocates a buffer and reads the manifest file via `0x140de2280` into handle +0x30.
  - It then calls `0x140d94540`.
- **State 2**
  - Waits for handle +0x30 vtable +0x30. Then `0x140d91cf0` parses the 'MFST' chunk ("packManifest", "packFile").
  - State 3 runs a sub-initializer (`[+0x38]`, `m_highIdsRequest`, "high ids"). When it finishes, `0x140d92150` runs.
- **`0x140d92150`**
  - Copies the manifest records (20 bytes each) into +0x48, with count +0x54.
  - Inserts **every** record key into the pending set (+0x60/+0x68).
  - Sets total `+0xe4 = record count`.
  - Sets state to 4 (`MOUNT_STATE_INITIALIZED_NEEDS_PATCHING`) or 5 (`PATCHED`). The exact selection is at about 0x140d922f7-0x140d92313. Not decoded further.
- **State 4 (patching):** no tick work. The preparer is purely a passive *source* of work.
  - `vt+0x60` = `0x140d92480` is `GetNext`. In state 4 it hands out the next record from the pending set (20-byte copy plus an AddRef on `m_archive`) and bumps +0xd8. **[V]**
  - `vt+0x28` / `vt+0x30` are `OnSuccess` / `OnFailure` (0x140d93b50 / 0x140d93b60). They forward to `0x140d928c0`, which calls `0x140d93920` (per-preparer `OnFileResult`). **[V]**
  - `OnFileResult` removes the key from the pending set and always increments `+0xe0` (finished).
    - `status==0` increments `+0xdc` (OK).
    - `status==8` is neutral.
    - Any other status logs "Download failed for manifest 0x%08x, fileId 0x%08x with error %u" and stores the error in +0xec (0x140d93a31-0x140d93a5e). **[V]**
  - When `+0xe0 >= +0xe4`: state = 5 if `+0xdc==+0xe4`, else 6 (0x140d93ab9-0x140d93acb). **[V]**
    - State 6 is the error branch that MAP_DOWNLOAD polls through +0x50.

**Who consumes `GetNext(+0x60)`: the asset downloader.** **[V]**
- AssetDownloaderCommon.cpp, function around 0x140d890b0.
  - It iterates the per-priority source list (`[queue+0x128]`, count `+0x134`) and calls `source.vt+0x60`.
  - It asserts `info.baseId` and `info.fileId` (lines 0x46c and 0x46d).
  - The queues are a static table at 0x1427fa5c0 with stride 0x148, priorities 0..5.
- Source registration is `0x140d88140(source, prio, flag)`, called from preparer +0x70 (0x140d91cda).
  - **Its first action** (0x140d88168) is `cmp qword [0x1427fad70],0; jnz ok`.
  - If `[0x1427fad70]` is NULL it calls `source.vt+0x38` (preparer vt+0x38 = 0x140d938e0, which only frees the preparer's buffer at +0xb0) and returns.
  - Nothing is registered.
  - The `started` flag was already set, so the child never retries.
- `[0x1427fad70]` is the AssetDownloader service instance.
  - It is cleared at 0x140d88b2d (shutdown, 0x140d88af0).
  - A static initializer, the function ending at 0x140d88ae0 (`jmp 0x140e4c210`), registers the service via `RegisterService(descriptor 0x142593090, priority 0x0a, &slot 0x1427fad70)`. **[V]**
  - The pointer is not written by any direct `mov [0x1427fad70]` except the null store. It is populated by the service framework when that service is started. **[V]** (xref scan: reads only, plus the one null store).
  - [I] whoever starts the service list decides whether it exists.
- The per-file "already installed" check happens inside the downloader. It reports `OnSuccess` without network I/O. [I] This is not traced to the end.

## 3. Q3: can a file wait without any network request?

Yes, by the code, in these ways:
- **Downloader service never instantiated (`[0x1427fad70]==NULL`).** **[V]** for the mechanism (0x140d88168-0x140d88178).
  - The preparer is never registered.
  - `+0xdc` stays 0 while `+0xe4 > 0`.
  - MAP_DOWNLOAD's +0x48 is 0 forever. There is no timeout and no error.
  - It is silent on the network.
  - It does not depend on Local.dat or Gw2.dat contents, because **all** records are counted, including already-installed ones (see 0x140d92150).
- **Downloader exists but its worker never runs**, for example paused or file writes disallowed. [I]
  - MultiArchive.cpp:497 asserts `!FileWritesDisallowed()` (0x140d8fd12).
  - Not traced.
- The main archive `s_mainArchive` (0x1427fadc0) is NULL. Its null guard is in 0x140d8f620.
  - **State 1 would crash**: 0x140d8f560 has no null check. So [I] it is non-NULL, or the preparer is not in state 1.

## 4. Most likely cause and how to unblock

**Caveat that matters.**
- If the preparers were still in states 1-3, `+0xdc == +0xe4 == 0` makes `+0x40`/`+0x48` return TRUE (complete) on the first tick. MAP_DOWNLOAD would then not wait at +0x48. **[V]** (initial values at 0x140d915fc-0x140d91611).
- A permanent wait at +0x48 therefore requires `+0xe4 > 0` and `+0xdc < +0xe4`, i.e. a preparer in state 4 whose records are never reported done. **[I]**
- That fits "downloader never pulls or registers the preparer". It fits the live symptom: a silent network and a Local.dat written ~3 s after entering the state (an archive/Local.dat flush that does not mean download activity).
- It also means the preparer has already **read its manifest records**. The Map manifest (or the Region one) was found, and state 4 was reached via `0x140d94540` / `0x140d943b0`, with the manifest file coming from `s_mainArchive`.
- **Stall location, checked with `pdf` of 0x1409520b0:** the only branch to the bare-return epilogue 0x140952620 is the `jz` at 0x140952160 (the +0x48 == 0 test). **[V]**
  - The other exits are state changes: error to state 6 at 0x1409521a2 and 0x14095264b, and completion to state 7 at 0x1409525ec.
  - After +0x48 passes, the function does more work (0x140cd8b60, `0x141095210` vtable +0xe0, `0x1413fe030` vtable +0x18, then a per-record loop). I did not decode that tail in detail. **[I]** it runs through to state 7 without waiting.
  - So the stall is at +0x48, unless that tail contains a blocking call.

**Ranked hypotheses**
1. **[I] (leading, conditional on the stall being at +0x48, which the `pdf` check above supports) AssetDownloader service not started**, so `s_manifestPreparers` entries sit in state 4 and are never registered.
   - Evidence: the null-guard behaviour in 0x140d88140 that skips registration silently, and no retry from the child.
   - Check next: who starts the service list that contains descriptor 0x142593090 (priority 10).
   - Check next: whether a condition skips it. Candidates are the "CDN latest == client build" case, `-noPatch`/`-shareArchive`, or an unreachable assetcdn host. Strings: `Need to patch before using -shareArchive or -noPatch.` (0x141922ce8), `s_gamePatcher` (0x141922a08), `!m_srvPatcher`.
   - AssetPatcher: ApManifest.cpp / ApArchive.cpp / ApExe.cpp / ApVerify.cpp; the ApManifest loader is at 0x140fdd6e0.
2. **[I] Downloader started but the preparer's `+0x70` was called while the downloader was not yet up.**
   - The `started` flag never resets, so the same silent permanent wait follows. Startup order: the preparer is created when the root manifest loads, but the downloader may come up later.
3. **[I] Records exist but the downloader's GetNext loop is blocked.** Examples: a window/throttle for in-flight requests, or the 10 s `s_mainArchive` +0x20 call being the thing that should unblock it.

**Unblock options**
- **Server/CDN side:** nothing on the CDN can unblock hypotheses 1 and 2. They need no network. A CDN `latest64` value that equals the client's build is a candidate trigger for "no downloader", but this is **unverified**.
- **Risk:** making the CDN report a different build is likely to drive the exe-patch path (ApExe.cpp, "Client needs to be patched first." at 0x14210e6d8), not just the asset downloader. Treat it as risky as well as unverified.
- **Client side:** check the AssetPatcher flow. If the CDN's `/latest64/101` reply ("209026 0 0 2130706510 160") makes the patcher believe the build is current and skip the downloader, either:
  - make the CDN's reported build/manifest id differ so the patcher activates and starts the downloader, or
  - run with a launch flag, if one exists, that forces the downloader.

  The decision logic is in the 0x140fdd6e0-0x140fe2dd8 AssetPatcher region.
- **Network gate:** CDN URL formats are `/latest64`, `/manifest/program/%u/%u` and `/program/%u/%u/%u/%u` (0x1420a6c80-0x1420a6cb0).
  - Host: `assetcdn.%u.arenanetworks.com` (0x1420a6bf0), built at 0x140d8d270.
  - Cookie: `authCookie=access=/latest/*!/manifest/program/*!/program/*~md5=%s` (0x14210e300).
  - None of these has been observed to fire. The preparer path above never reaches them without the downloader.

## 4b. External conditions: traced vs not traced

| Condition | Status |
|---|---|
| Async archive read of the manifest file | Traced as far as the call: `0x140de2280` / `0x140d94540` from preparer state 1. Not followed into the file layer. |
| Local.dat "installed" record | **Not traced.** [I] the check lives inside the downloader's `GetNext` consumer (about 0x140d890b0). |
| File-download queue (Patch/Download/Archive) | Traced: registration `0x140d88140`, consumer about 0x140d890b0, queue table 0x1427fa5c0. |
| Decryption key (AssetEncryption\Cli\GaeCliContext, 0x1425bd180) | **Not encountered** on the traced path. Not ruled out. |
| Server message | **Not encountered** on the traced path. Not ruled out. |
| Open question for the lead | Did the client itself request `/latest64` or anything from assetcdn at startup, before map load? If yes, downloader code was alive, which weakens hypothesis 1 in favour of hypothesis 2. |

## 5. Addresses and strings used

| Item | Address |
|---|---|
| MAP_DOWNLOAD | 0x1409520b0 |
| PoDownloader vtable / ctor | 0x14235d620 / 0x1417ea6d0 |
| Request vtable | 0x14235d6d0 |
| Factory | 0x1417e9630 |
| Factory owner ctor | 0x1417e9ee0, called from MapPortalLoader init 0x1413fe070 at 0x1413fe108 |
| get-or-create child | 0x1417e97e0 |
| Preparer vtable / ctor / tick / GetNext / OnFileResult | 0x1420a7368 / 0x140d913a0 / 0x140d91950 / 0x140d92480 / 0x140d93920 |
| Root manifest build | 0x140d93070 |
| Root manifest by-name lookup | 0x140d92d00 |
| `s_manifestPreparers` | 0x1425930f8 (head), 0x142593100 (table) |
| Root manifest `m_root` | 0x1427fb0e0 |
| Downloader source registry | 0x140d88140 |
| Downloader GetNext consumer | about 0x140d890b0 |
| Downloader service global | 0x1427fad70 |
| Downloader queue table | 0x1427fa5c0 (stride 0x148) |
| `s_mainArchive` (setter) | 0x1427fadc0 (0x140d8fcb0) |
| Strings | "Manifest 0x%08x is missing." 0x1420a7448; "Download failed for manifest 0x%08x, fileId 0x%08x with error %u" 0x1420a7540; "state == MOUNT_STATE_INITIALIZED_NEEDS_PATCHING || state == MOUNT_STATE_PATCHED" 0x1420a74c0 |

---

# Round 2 (lead's follow-up: who starts the downloader, what gates its loop, root manifest source)

## R2.0 Retractions of round-1 claims

- **Retracted:** "`0x140e4c210` is a service registry" and "downloader service never started" (hypothesis 1 / unblock bullet 1).
  - `0x140e4c210` is **Timer registration** (Timer.cpp, thread name "TimerThread" at 0x140e4c382). **[V]**
  - `[0x1427fad70]` is the timer handle for the downloader tick. **[V]**
  - The job object is the static object at `0x142593090`, vtable `0x1420a5438`:
    - vtable +0x00 = `0x140d87220` is the tick.
    - vtable +0x18 = `0x140d87650` is the deleting dtor.
    - Registration interval is 0x0a (10 ms).
  - The tick returns 10 while any queue has work and -1 when all queues are idle.
    - `0x140d88140` wakes the timer through `timer.vt+0x20` after a registration (0x140d883fa).
    - So a NULL `[0x1427fad70]` at registration only happens before init or after teardown (0x140d88b2d, from main teardown at 0x1402384c4 / 0x140237826).
- **The downloader exists in the owner's run.** **[V]** by straight-line code:
  - `AssetDownloader::Initialize(bool)` is `0x140d88920`.
  - It is called from MainCli's startup function (starts ~0x14023756f) at `0x1402376fa`.
  - It follows `0x140d8c7d0(host, programId, cookie, ..., connections)` at 0x1402376e2 with no conditional jump in between.
  - That `0x140d8c7d0` call is the asset HTTP/CDN client init that produced the observed `/latest64` connection. [I] it is the same call, not proven.
  - The only conditional exits are earlier in that function, to `0x140237558` (0x1402375b9, 0x1402375c2, 0x1402375f6, 0x140237619). They are mode checks, not decoded.
- Startup parameters read in that function:
  - Param 123 (`0x7b`), a string. If it equals "default" the host is built as `assetcdn.<programId>.arenanetworks.com`. Otherwise **the param string is used as the CDN host** (0x140237642-0x140237685). [I] this is `-assetsrv <host>`; the name list contains "assetsrv" at 0x14213dd50 but I did not map param ids to names.
  - Params 182 (`0xb6`) and 183 (`0xb7`) feed the HTTP client; 182 has default 8. [I] 182 is `patchconnections` (the name list has it at 0x14213e000).
  - Param 7 (bool) is passed to `Initialize`. It is stored at `[0x1427fad80]` and only sets a per-item limit (0xfffff versus -1) at 0x140d89dc8-0x140d89ddf. It is not a download on/off switch.
  - The cookie string is built at 0x140fdd4c0.

## R2.1 (a) Answer: nothing gates the downloader's existence on this path

See R2.0. It is created once at startup and is alive for the whole session.

## R2.2 (b) Downloader tick: what gates work before it pulls from a registered preparer

Tick `0x140d87220` runs 6 priority queues (static table 0x1427fa5c0, stride 0x148). For each queue it calls `0x140d876c0`: **[V]**
1. `0x140d87a20`: advance in-flight work items (the HTTP phase).
2. `0x140d87f30`: start phase.
   - If the pending-request list `[queue+0x48]` is empty it calls the producer `0x140d890b0`.
   - The producer calls `GetNext` (`source.vt+0x60`) on every registered source (`[queue+0x128]`, count `+0x134`). Each `GetNext` returns a record, and a 0x38-byte request item is queued in `[queue+0x48]`. There is **no gate** on the /latest result, exe id, root manifest, or any flag in this producer.
   - For each queued request it calls `archive.vt+0xa8(baseId, 0)` on `m_archive` (= `s_mainArchive`) at 0x140d87fc9.
     - If it returns nonzero, the request is satisfied locally: `source.vt+0x28` (OnSuccess) is called (0x140d87ff5-0x140d87ffd). **No network.**
     - If it returns zero, `QueueWork` (`0x140d89ea0`) allocates a work item from a pool. If the pool is exhausted (`[pool+0x88] >= [pool+0x90]`, 0x140d89edb) it returns 0 and the request stays queued, retried every tick. **Silent.**
3. In `0x140d87a20`, an item that needs the network is only started when the global active counter `[0x1427fad7c] < 10` (0x140d87bfc), except items with `+0xcc == 7`. The counter is incremented at 0x140d87caf and decremented in `0x140d87760` (0x140d8783d, 0x140d87863, 0x140d8796a, 0x140d87977). [I] a leaked counter would stall all later downloads silently. Not shown to happen.
4. `0x140d876c0` also runs a 5 s (0x1388 ms) stall timer through `[queue+0x144]` and `0x140d8c600` / `0x140d8c850` on `[0x1427fada8]`. These are not decoded further.

CDN host and request builders (AssetHttpRequest.cpp):
- HTTP client vtable `0x1420a6ba8`: +0x18 `0x140d8ec80`, +0x20 `0x140d8ee90` (`/latest64/%u`), +0x28 `0x140d8f0c0` (`/manifest/program/%u/%u`), +0x30 `0x140d8f280` (`/program/%u/%u/%u/%u`). **[V]**
- The host is built at 0x140d8d270 (`assetcdn.%u.arenanetworks.com`) unless a host string is already present.

**Preparer-side interest groups (new):**
- Preparer `vt+0x70` (0x140d91c70) is AddInterest(group). In state 4 it appends the group to the list at +0xa8 and calls `0x140d88140`.
- Preparer `vt+0x78` (`0x140d93d00`) is RemoveInterest / Cancel(group). **[V]**
  - States 0 and 2: set the abort flag `[+0xe8]=1`, and if the file read handle is done, go to state 7.
  - State 1: only sets `[+0xe8]=1`.
  - State 3: sets the initializer's abort flag.
  - State 4: removes the group from +0xa8, and unregisters from the downloader via `0x140d884a0` when no groups remain.
  - States 5-7: no-op.
- At `0x140d91a02` the tick turns **`[+0xe8] != 0` into state 7 instead of entering state 4**. State 7 is terminal: it is never complete (+0x40) and never an error (+0x48 tests `state==6`). **[V]**
  - A preparer put into state 7 therefore makes MAP_DOWNLOAD wait forever with no network and no error. **[V]** (state 7 handling at 0x140d93d13, 0x140d91a02-0x140d91a07).
  - Child pump `0x1417ea980` would also call `+0x70` on it: state 7 is neither 4 nor 5, so the assert at 0x140d91c9d fires. [I] it is logged or reported.
- Child dtor `0x1417ea800` calls `+0x78(1)` on its preparer, then ticks it until `state >= 4`. **[V]**
  - The preparer is shared. If a previous PoDownloader child for the same manifest id was destroyed while the preparer was still in states 1-3, the preparer ends in state 7.
  - A later request for the same id then waits forever.
  - Children are cached in the factory dictionary (factory +0xb0, 0x1417e97e0), so a reused child or preparer would keep that state. **[V]** for the caching. [I] that a destroy-before-init actually happened.
- MainCli patch phase (function `0x140237b90`, called from 0x1402366ab / 0x140236c48 / 0x140236ea4): **[V]**
  - `0x140d93840` ticks every preparer until its state is >= 4, with 1 ms sleeps.
  - It then calls `+0x78(6)` on each, and sums `+0x68` (`0x140d93ce0` = `+0xe4 - +0xd8`, records not yet handed out).
  - If the sum is nonzero and command-line flags 0x28 / 0x1f are set (the message names -shareArchive / -noPatch), it reports "Need to patch before using -shareArchive or -noPatch." (0x140237cb9).
  - [I] by map-load time every preparer in the root manifest has already been initialized (states 4/5), so the Map preparer is not normally in state 1-3.

## R2.3 (c) Does the Map/Region lookup need the CDN root manifest or the Gw2.dat copy?

- `0x140d92d00` scans `m_root` = `[0x1427fb0e0]`, the ARMF chunk that ApManifest (`0x140fdd6e0`) loads. **[V]**
- ApManifest first asks the **local archive index** whether the root manifest is present: `0x140d92b40` → `0x140d943b0` (MultiArchiveLocal.cpp, index dictionary at 0x1427fb110/0x1427fb118, records of 16 bytes {id, size, crc..}). **[V]**
  - It asserts `rootManifestExists` (0x140d92ba7).
  - If the record exists with matching size, it returns 0 (nothing to download) and the root is read **locally** through `0x140de2280` / `0x140d94540` ([0x1427fb140] vt+0x78). No network.
  - On a size mismatch it logs "[0x%08x] Existing exe in archive has size %u (expected %u)." (0x140d92bdd), removes the record (`0x140d94160`) and builds a download batch.
- The per-map manifests (the Map/Region preparers) are also read locally in preparer state 1 (`0x140d91fc0` → `0x140d943b0` → `0x140de2280` → `0x140d94540`). **[V]**
  - A missing record gives "Manifest 0x%08x is missing." and a fatal report "Manifest is missing." (0x140d92047-0x140d9206b).
- [I] The lead's observation fits this: no `/program/...` request was seen for 2130706510, so the root manifest came from the local archive (Gw2.dat/Local.dat index). The CDN reply was only consulted for id and size, to decide whether the local copy was current.
- [I] If the local index record for the root manifest had been missing or had the wrong size, a `/program/101/1/0/2130706510` request would have appeared.

## R2.4 Revised ranking and cheap discriminators

Ranked [I]:
1. **A preparer for the Map or Region manifest is in state 7** (aborted during init) or otherwise terminal, so MAP_DOWNLOAD waits forever.
   - It is shared and cached, so the destroy-before-init case would persist.
   - It fits "no network, no error" exactly.
2. A preparer in state 4 whose registered files never finish, because of the 10-item active counter or pool exhaustion.
3. Anything after the PoDownloader step (not seen to wait).

Log lines in the client log or debug output that would discriminate. They are all emitted through `0x1409bb500`. The lead can grep:
- "Manifest 0x%08x is missing." (0x1420a7448) means state 1 failed to find a local record.
- "Falling back to full file-download for 0x%08x:0x%08x" (0x1420a5950) / "uncompressed" (0x1420a59c0) means an HTTP item reached the network phase.
- "Download failed for manifest 0x%08x, fileId 0x%08x with error %u" (0x1420a7540) means a failure was reported (state 6).
- "Raw file 0x%08x:0x%08x has CRC 0x%08x." (0x1419228e0), "File 0x%08x not found in main archive." (0x141922990), "Failed loading file 0x%08x for CRC-computation." (0x141922930) are the CRC path in MainCli.
- "[0x%08x] Existing exe in archive has size %u (expected %u)." (0x1420a7600) means the local root manifest record was rejected.
- An assert on `state == MOUNT_STATE_INITIALIZED_NEEDS_PATCHING || state == MOUNT_STATE_PATCHED` (0x1420a74c0, MultiManifest.cpp:814) would mean `+0x70` was called on a state-7 preparer. That would support hypothesis 1 directly.

---

## Round 3 (manifest persistence, self-restart, the null request)

By a read-only helper, 2026-11-03. **Lead's note:** the deployed fix (kit cd93d58, root 2130706520) adds `MapValleyHill` and `RegionValley`. The latter is "Region" + mapDef +0x110 = "Valley", decoded from content. A mention of "RegionValleyHill" below is the helper's typo. On the device, the state-6 error exit *is* the hang, because the dispatcher has no case for state 6.


Read-only. Frozen `Gw2-64.exe` (base 0x140000000), rizin disassembly plus PE scans. No client run, no repo edits.
**[V]** = read in disassembly at the cited address. **[E]** = observed on disk or in a log in the sandbox. **[I]** = inferred.
Rounds 1-2 are `docs/re/209026-map-download.md`. Where this note disagrees with them, this note wins.

## 0. Bottom line

1. **The exe fields do not cause the restart.** With reply build == 209026 the client never touches the exe-patch path.
   - `exeFileId` and `exeFileSize` are read only in the build-mismatch branch.
   - "209026 0 0 <manifestId> <manifestSize>" already means "my exe is current". Leave those fields alone.
2. **Gw2-64.tmp touched at 0 bytes is not evidence of a self-update.** It is created empty at every normal startup.
   - [V] code at `0x1410d9c60` → `0x1410da0b0`.
   - [E] the local sandbox dir has a 0-byte `Gw2-64.tmp` with mtime 22:54:14, minutes before `debug.log` at 22:54:22. Those runs stop at the User Agreement panel and do not restart.
   - The cause of the exit about 2 minutes later is **not found statically** (§1.4).
3. **Persistence is Local.dat.** The "installed" table that decides fetch-or-not is built only from Local.dat's file table (§2).
   - The manifest ids that the client sees as present are matched by `baseId` and size. The CDN-supplied fileId must also equal the stored one.
4. **Our root manifest 2130706510 has no `Map<name>`/`Region<name>` entry.**
   - [E] the 160 B root has only `Launcher64` and `English`.
   - run79's root `2130706520` (268 B) has `Launcher64`, `English`, `MapValleyHill`, `RegionValleyHill`.
   - [V] with no matching entry, the MAP_DOWNLOAD request factory returns NULL and the stage goes to state 6 at once (§3.3). That is the error exit, not a stall.
   - So the device's run-2 stall is **not explained** by the 160 B root. §3.3 lists what must differ, and the runtime check that separates them.
   - Either way, the CDN must serve a root with the map's `Map`/`Region` entries.
5. **Smallest fix (§4):** serve the run79-style root (it contains the map manifests) under **fresh ids on every change**. Keep the `/latest64` exe fields `0 0`. Make the client re-fetch by changing the manifest id, or delete Local.dat before the run.

## 1. Q1: restart / exe self-update

### 1.1 `/latest64` parse [V]
`0x140d8c630(client, out)` reads the body (at most 127 bytes) and tokenizes it 5 times with `0x1409ba140`. A bad or missing token gives 0.

| out offset | value |
|---|---|
| +0x00 | build |
| +0x04 | exeFileId |
| +0x08 | exeFileSize |
| +0x0c | manifestFileId |
| +0x10 | manifestFileSize |

Callers are `0x140fdf1d5` (inside 0x140fdf100, the `/latest64` variant) and `0x140fe1c87`.

### 1.2 The comparison [V]
The state machine in MainCli (`0x140236180`, 10 states, switch table 0x1402366e8) starts the exe patcher in state 0.
- State 0 (`0x1402361c5`..`0x140236234`) builds a "PatchExeThread" object with `0x140fded50`. Vtable `0x14210e760`, thread name string at 0x14210e7b0.
  - Field +0x84 = `(GetParam(0x28) == 0)`, so "patching allowed" unless param 0x28 is set.
  - Field +0x98 = callback `0x140238640`.
- The worker is `0x140fdf470`:
  1. `0x140fdf100` fetches `/latest64`. It is tried twice. The result is stored at `[rsp+0x48..0x58]`.
  2. `0x140fdf4c8`: `call 0x1409afcc0`. That function is `mov eax, 0x33082`, the client's compiled-in build **0x33082 = 209026**. The comparison is client vs `latest.build` (0x140fdf4cf).
  3. `0x140fdf4f3`..`0x140fdf4fe`: `cmp client, latest.build; jz 0x140fdf5fd`.
  4. **Equal build → `0x140fdf5fd`.** It copies manifestFileId into `[obj+0x8c]` and manifestFileSize into `[obj+0x90]`, then finishes. exeId and exeSize are ignored.
  5. Different build: if `[obj+0x84] != 0` and the client build is not greater than `latest.build` (`bl = client > latest`):
     - `0x140fdf511`..`0x140fdf53f` calls the exe patcher `0x140fdee60` with `r8d = exeFileId` and `r9d = exeFileSize` (AssetDownloadExe.cpp).
     - Otherwise (patching disabled, or the client is newer) it takes the `0x140fdf54c` path and fetches the program manifest `0x140fdeb80`/`0x140fdf220`. It then reads the `baseId 0x1005` record from it via `0x14156f4b0` (`0x140fdf5b5`..`0x140fdf5d0`).
- MainCli state 1 (`0x14023626b`) then:
  - calls vt+0x38 on the thread object (nonzero means failed → `[this+4]=1`, `0x140236f00`);
  - otherwise calls vt+0x30 to store the two values into `this+0x18` and `this+0x1c`.
- State 2 (`0x14023638f`) creates `ApManifest(baseId=0x1005, fileId=[this+0x18], size=[this+0x1c])` through `0x140fdd5f0`.

**Answer to "what values make the client treat its exe as current":** `build == 209026` is sufficient and the safe value. (A client newer than the CDN, or param 0x28 set, also skip the exe download, but do not rely on that.) exeFileId and exeFileSize are ignored in that case. A reply of "209026 0 0 …" is already right.

**If build differs:**
- Greater than 209026 with patching enabled → exe download via exeId/exeSize, a restart, and an empty or failed exe.
- Smaller than 209026 → the client takes the `0x140fdf54c` path.
- Do not do either.

### 1.3 Where "Existing exe in archive has size %u (expected %u)" really is [V]
It is the string at 0x1420a7600, logged from `0x140d92b40` at `0x140d92bdd`.
- `0x140d92b40(baseId, fileId, size, &request)` is the **root-manifest** "is the local copy current" check.
- It is called by the ApManifest loader `0x140fdd6e0` with `[obj+0x78/0x7c/0x80]` = baseId 0x1005, manifestFileId, manifestFileSize.
- The word "exe" is a naming leftover. There is no exe record in this check. (The log arguments are fileId, existing size, expected size.)
- Behaviour:
  1. `0x140d943b0(baseId, 0, &installedFileId, &size, &flag)` looks up the install table.
     - If there is no record at all it **asserts** `rootManifestExists` (0x140d92ba7, line 0x3ff).
     - `0x140fdd6e0` calls `0x140d95170(baseId, fileId, size)` first, which inserts the record with `target = fileId`.
  2. `flag = (target == installed)`.
     - If `flag != 0` and the stored size equals the expected size → **return 0: nothing to fetch.**
     - If the sizes differ → log the line, remove the record with `0x140d94160`, and build a download request registered through `0x140d88140`.
     - If `flag == 0` → build the download request.
- So a root is "local" iff the Local.dat-derived table says installed fileId == `latest.manifestFileId` **and** stored size == `latest.manifestFileSize`.

### 1.4 What `Gw2-64.tmp` really is, and what exits [V/E/I]
- `0x1410d9c60(0)` is called from main at `0x1402375ef`. If the module name ends in ".exe" it builds the sibling name with ".tmp" (`0x14212b9f8`) and calls `0x1410da0b0(path, 0)`.
- With `edx == 0`, `0x1410da0b0` does: GetFileAttributesW, clear read-only, CreateFile, `WriteFile(h, buf, 0 bytes)`, SetEndOfFile, CloseHandle (0x1410da0e3-0x1410da1b0). **[V] That is a zero-byte touch.**
- [E] The sandbox shows exactly that: `Gw2-64.tmp`, 0 bytes, mtime 2026-11-03 22:54:14. The runs that stop at the User Agreement panel do the same.
- [I] (branch not fully read) If the exe is run as `<name>.tmp` instead of `.exe` (`0x1410d9e29` branch), the same function appears to copy back over the `.exe` and relaunch. That would be the self-update relaunch; the callback `0x140238640` → `0x1410d9a20` writes the downloaded exe bytes to the `.tmp`. Run 1 shows a 0-byte tmp, which matches the plain startup touch.
- **Cause of the exit ~2 min after start: [I], not found statically.** Candidates, in order:
  1. An assertion crash in the Wine client.
     - [E] `prefix/.../Guild Wars 2/ArenaNet.log` from the sandbox (2026-11-01) shows `Assertion: s_configDataError != NET_PENDING`, GcApi.cpp(2621), `Uptime 0:01:00`, build 209026.
     - It is the same kind of "dies about a minute after start". If the device log shows the same, no patcher change is needed.
     - **Concrete next step: read the device's `ArenaNet.log` and `Crash.dmp` from run 1.**
  2. The launcher bubble (not the client) relaunched it. Run 1's own "logged out" line is outside what I traced.
  3. MainCli state 1 saw the PatchExeThread report failure (`0x14023627a`: vt+0x38 nonzero → `0x140236f00` → quit). That needs both `/latest64` attempts to fail. Not what run 1 shows ("GET /latest64 succeeded").

## 2. Q2: where the fetched manifests go and why run 2 skips the fetch

### 2.1 Two archives [V]
- **Local.dat = `s_localArchive` (global 0x1427fb140).**
  - Main (`0x140237190`) builds `Local.dat` paths. The name string is `L"Local.dat"` at 0x1419226a0, loaded through the pointer at 0x142554000 (see `0x140237745`/`0x140237778`).
  - It opens the archive with `0x140239520` (`0x1402377fd`) and installs it through `0x140d8f770` → `0x140d94580` (`0x140237877`).
  - `0x140d94580` stores a **baseId limit 0xfff** at 0x1427fb148, reads the archive's file table, and builds the install table through `0x140d94910` (its only call site is `0x140d946cd`, with `r9 = [0x1427fb140]`).
  - Install table: 16-byte records at `[0x1427fb0f8]`, keyed by baseId through the dictionary at 0x1427fb110/0x1427fb118.
    - `+4` = target fileId, `+8` = installed fileId, `+0xc` = size.
    - Source of the fields from the earlier doc item 10; [V] for the offsets via `0x140d943b0`.
- **Gw2.dat = `s_mainArchive` (0x1427fadc0).**
  - It is opened in `0x140238560` and installed through the setter `0x140d8fcb0` at `0x140238603`.
  - That happens in the state-1 transition (`0x140236353`) **after** the exe-patch thread has finished.
  - It is not what feeds the install table.

### 2.2 Writing and reading manifests
- A successful download callback `0x140d95270` calls `0x140d953a0(table, baseId, fileId, size, flag=1)` and marks the record installed. `0x140d95170` (flag 0) sets the target. [V]
- `0x140d94540(baseId, …)` (the preparer's read) goes through `[0x1427fb140]` vtable +0x78. **Manifest bytes are read from the local archive (Local.dat).** [V]
- [I] The sandbox prefix's `Local.dat` is 26,886,304 bytes (mtime 11-03 05:01). It has an ANet header and `MFST` blobs at 0x208/0x20c. I do not know which run wrote it, so it only suggests that Local.dat holds fetched content. Gw2.dat's mtime in the client dir (11-01 00:34) is older than the local runs.
- **Caveat, contradicts earlier docs:** `209026-patcher.md` item 4 says manifests were cached in Gw2.dat (run 26), and item 13 says a Launcher64 manifest listing every Gw2.dat asset produced no downloads (run 34). That does not fit an install table built from Local.dat only.
  - Possible explanations: in those sandbox runs the local archive path resolved to Gw2.dat's directory/name; or `0x140d94160`/`0x140d953a0` add main-archive records somewhere I did not trace. I did not run the xref pass on writers of `0x1427fb0f8`/`0x1427fb118`.
  - The baseId limit 0xfff (0x1427fb148) versus root baseId 0x1005 is also unexplained. `0x140d94160` branches on it (`0x140d9416f`), but both branches do a dictionary lookup (`0x140d9429f`); I did not decode the difference.
  - The device's own behaviour (fresh Local.dat → fetch, existing → no fetch) is the strongest evidence that Local.dat state decides it.
- Therefore (assuming the table is the decision):
  - Run 1 (fresh `Local.dat`): empty table → fetch root, then the root's children (the 13 MB, the 112 B).
  - Run 2: the table was built from Local.dat. A record with baseId 0x1005, installed fileId == 2130706510 and size 160 satisfied the check in §1.3, so there was no fetch. The same for the children (`0x140d92980`, installed != fileId test).
- **[I] The device's Local.dat is 5,349,904 bytes, much smaller than the 13 MB manifest it should hold.**
  - Either the manifest data is compressed in the archive.
  - Or the table has records without data.
  - I cannot separate the two statically.
  - A cheap check on the device is to list Local.dat's file table (does it have records for fileIds 2130706510/11/12?).

### 2.3 If the 13 MB manifest is not stored
- Nothing in the Map/Region lookup reads it. `0x140d92d00` scans only the **root** (`m_root`, 0x1427fb0e0).
- The 13 MB `Launcher64` manifest is only read by its own preparer. It matters only for file-level downloads, not for finding the Map/Region manifest ids.

## 3. Q3: MAP_DOWNLOAD in run 2

### 3.1 Why there is no registration [V]
- Registration `0x140d88140` has three callers on the paths in question:
  - `0x140d92c98`, the root fetch request (§1.3);
  - the root-children batch built from `0x140d92980`;
  - preparer `+0x70` (`0x140d91cda`), which is reached from the child pump `0x1417ea980` when the preparer is in state 4.
- Run 1 had two registrations (the root request, then the children batch). Run 2 skipped both because the table said "installed". Zero registrations is exactly what a fully-local startup should show. It does not by itself mean a stall.
- State 5 ("PATCHED") preparers register nothing and are complete.
  - `+0xdc == +0xe4` makes the child's +0x48 true.
  - MAP_DOWNLOAD would then pass.
  - So a **stall with zero registrations means the Map or Region preparer is not in state 4, 5 or 6.** It is in 1-3, or 7.

### 3.2 State of the preparer when the root is "local but not loaded"
- Preparers are created per root entry by `0x140d93070` from `m_root`. This is independent of whether the root came from the network. [V]
- In preparer state 1 (`0x140d91fc0`) the manifest is looked up in the install table (`0x140d943b0`) and read from Local.dat (`0x140de2280`/`0x140d94540`). [V]
- If the lookup fails, the code logs "Manifest 0x%08x is missing." and reports a fatal error, which is not a stall.
- State 2 waits on the read handle (`+0x30`, vtable +0x30). If the read handle never completes, the state never advances. [I] this is a candidate stall (record present in the table, bytes absent or unreadable in Local.dat).
- I found **no root-manifest mount or patch step that runs only in the fetch path.** The map preparers do not wait on one. [V by §3.1/§3.2 code: nothing branches on "root was downloaded"].

### 3.3 The Map id is 0 with our 160 B root (important mismatch)
- `0x1413fe2a0` builds `L"Map" + name` and `0x1413fe340` builds `L"Region" + name`, each through `0x140d92d00`, which scans the root's 24-byte entries (names at `entry+0x10`).
- The factory `0x1417e9630` calls `[sub-object]+0x10` (Map) first. **If it returns 0 → `0x1417e972a`: returns NULL.** [V]
  - If only Region returns 0, the request has one child (the Map child).
- `0x1409536fe` stores the factory result at `loader+0x218`. MAP_DOWNLOAD then takes the NULL path (`0x140952130` → `0x14095262c`, "Completed STATE_MAP_DOWNLOAD with error"). [V]
- [E] The 160 B root `2130706510` has only `Launcher64` and `English`. run79's 268 B root `2130706520` has `MapValleyHill` and `RegionValleyHill` too.
- Checked after review:
  - The "error" call at `0x140952646` is `0x1409bafd0`, a **no-op stub** (it spills its arguments and returns). So the NULL exit is not a blocking reporter. It falls straight through to `mov [rbx+0x2c0], 6` at `0x14095264b`. [V]
  - Slots: the owner sub-object vtable 0x1422dbc28 has +0x10 = `0x1413fe2a0` (Map) and +0x18 = `0x1413fe340` (Region) (qword refs at 0x1422dbc38/0x1422dbc40). `0x1413fe2a0` returns the `0x140d92d00` result directly, with no fallback id. [V]
  - `0x1417e9630` sits in a vtable slot at 0x14235d4b0, which is vt+0x8 if that vtable starts at 0x14235d4a8. I did not decode which object `0x1413fe030` returns, so "vt+0x8 of that object is the factory" is still borrowed from round 1. [I]
- So **statically, a 160 B two-entry root must leave state 5 at once (state 6), not stall.** The device stalls, so one of these must hold. All [I]:
  1. The stage trace or the owner's observation does not mean what we think. Runtime discriminator: a live +0x48 wait ticks `0x140952139` every frame and calls `0x140d8f620` every 10 s. The error exit hits neither and writes state 6 once.
  2. The device was not using the 160 B root content we serve today (CDN files under ids 2130706510/11/12 changed between run 1 and run 2, or between devices). **Unlikely via Local.dat alone:** a stored root accepted under id 2130706510 must also be exactly 160 bytes (§1.3 size check). A root with Map/Region entries is larger (268 B in run79), so the size check would reject it and refetch. Stale content of that size would only have two short names.
  3. The device client used a different `latest` reply than the one reported, or a different map definition path.
  4. The tail of MAP_DOWNLOAD after `0x14095215b` (not decoded) contains a wait that I did not see.
- **Cheap discriminator for the lead.** At MAP_DOWNLOAD, read `[0x1427fb0e0]` (the `m_root`). Its entry count is at `+4` and the array at `+8`, 24 bytes per entry, name pointer at `entry+0x10`. A second read: `[0x1427fb140]`'s table for baseId 0x1005 (`0x140d943b0`). That shows which root content the device is actually using, and whether the Map entry exists.

## 4. Q4: smallest fix

1. **Exe fields: change nothing.** Keep `/latest64/101` = `209026 0 0 <rootId> <rootSize>`. [V §1.2]
2. **Serve the run79-style root** (Launcher64, English, `Map<name>`, `Region<name>` with their small manifests).
   [V] Without a `Map<name>` entry the request factory returns NULL and MAP_DOWNLOAD errors (§3.3).
3. **Never reuse a root or child manifest fileId for different content.**
   - The only freshness test is the installed fileId, plus the size (§1.3). A matching id and size means the client trusts Local.dat's bytes without refetching.
   - Mint new ids whenever content changes. Bump both the root id and each child id.
   - To make every start behave like run 1 (always fetch), return a **fresh root id per CDN start** and fresh child ids in that root. Then `target != installed` → refetch root (flag == 0, §1.3) and children.
   - Cost: Local.dat grows with each unique set of ids.
4. **Equivalent without any CDN change: delete the device's Local.dat before launching.** That is what every local sandbox run does implicitly. [E: local runs have no Local.dat and always fetch.]
5. **Do not** try to make the exe look "different" to force the downloader. Round-1 hypothesis 1 (downloader not started) was already retracted. The exe path is a different risk (§1.2).

## 5. What I did not do or could not settle
- The cause of the exit at ~2 min (§1.4). Needs the device's `ArenaNet.log`/`Crash.dmp`, or a breakpoint at `0x140236f00`.
- Whether Local.dat stores the 13 MB manifest compressed (§2.2). Needs the file table.
- The tail of MAP_DOWNLOAD after `0x14095215b` (not decoded).
- Whether Local.dat is looked up in the exe directory or in AppData on the device (`0x141090e20` kinds 1 and 2 near `0x14023772e`; not decoded).
- Preparer state timing for the Map/Region children at run 2. Only the discriminator in §3.3 settles it.

## 6. Addresses used

| Item | Address |
|---|---|
| `/latest` parse | 0x140d8c630 |
| PatchExeThread ctor / vtable / worker / `/latest64` fetch | 0x140fded50 / 0x14210e760 / 0x140fdf470 / 0x140fdf100 |
| Exe patch / program-manifest path | 0x140fdee60 / 0x140fdeb80, 0x140fdf220 |
| Client build constant | 0x1409afcc0 (`mov eax, 0x33082`) |
| MainCli patch state machine / state setter | 0x140236180 / 0x1402387e0 |
| ApManifest ctor / loader / local check | 0x140fdd5f0 / 0x140fdd6e0 / 0x140d92b40 |
| Install table add / lookup / remove | 0x140d95170, 0x140d953a0 / 0x140d943b0 / 0x140d94160 |
| Local.dat open and install | 0x140237190 (main), 0x140239520, 0x140d8f770, 0x140d94580, 0x140d94910 |
| Main archive open and install | 0x140238560, 0x140d8fcb0 |
| Zero-byte touch | 0x1410d9c60, 0x1410da0b0 |
| Map/Region by-name lookup / factory | 0x140d92d00 / 0x1417e9630 (NULL at 0x1417e972a) |
| MAP_DOWNLOAD NULL/error exit | 0x140952130 → 0x14095262c |
| Globals | `s_localArchive` 0x1427fb140; limit 0x1427fb148; table 0x1427fb0f8; dictionary 0x1427fb110/118; `m_root` 0x1427fb0e0; `s_mainArchive` 0x1427fadc0 |
| Sandbox evidence | `client-work-209026/Gw2-64.tmp` (0 B, 22:54:14), `prefix/.../Local.dat` (26,886,304 B), `prefix/.../ArenaNet.log` (GcApi.cpp:2621 assertion), `runs/run79-map-manifests/cdn-files/2130706520` (268 B root with MapValleyHill/RegionValleyHill) |
