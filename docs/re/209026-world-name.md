# Build 209026: how a world id becomes the displayed world name

Static analysis only (rizin on the frozen `Gw2-64.exe`, image base `0x140000000`), by a read-only helper on 2026-11-03. The lead re-checked the entry guard bytes, the zero cave at `0x142553600` and the `[rax+0x28]` compare. Nothing was run.
**V** = verified from disassembly or table bytes at the cited address. **I** = inferred.

## Answer in one paragraph
The server never sends a world name. All name text comes from the client's own content. The single
world-id-to-name function is **`0x140312330(worldId)`**, which returns a `wchar_t*`. It finds the content-373
record whose `+0x28` equals the id, takes the text id at record `+0x2c` and resolves it through the text system.
The home-world dialog's Name column and 15 other call sites all use it. The smallest safe patch is a guarded
conditional breakpoint at `0x140312330`: when `ecx == 1001`, return a pointer to a static UTF-16 "Banshee Realm"
and skip the function. Chat and party labels that read `+0x2c` directly need a second hook (see §4).

## 1. Does any server message carry a world name? No
- **V** The staging list holds 4-Dword entries. `0x140240ba0` (server 25) reads `[rdx+2]`, `[rdx+6]`, `[rdx+0xa]`
  and `[rdx+0xe]`. The list stride is 16 bytes: `0x140240cd1 shl r8, 4` before the memcpy `0x1409b7320`.
  There is no string field, and no pointer in the entry.
- **V** I decoded the 21 server-side definitions at `0x141923fe0` (kinds: 4 Dword, 2 Byte, 11 GUID, 13 String,
  17 ArrayVarSmall, 19 BufferFixed, 21 BufferVar). Only five carry any string or buffer, and none relates to worlds.
  - Server 17 and 18 carry `{Dword, GUID, Dword, kind 5, kind 5, String(20), BufferVar(512)}` (kind 5 is not a string kind; I did not decode it, **I** a fixed-size integer). Their handlers
    `0x140240a00` and `0x140240a50` go to `0x14023aa50` and `0x14023ab70`, which look up an outstanding request
    by its request id (`cmp [rax+0x30], r10d`), i.e. they are request replies.
  - Server 8 carries `BufferFixed(32)` ×2 and `ArrayVarSmall(50)`. The array entries are Word, Word and Dwords.
  - Server 10, 27, 13 and 30 carry only Dwords, arrays or buffers. Servers 3, 15 and 20 also have kinds 3 and 12 (param 0, unlike the
    length-carrying string kind 13), so **I** they are fixed-size values, not names.
  - The only string kind (13) appears only in servers 17 and 18.
  - **I** None of these is a display name for a world. I did not trace every consumer of those buffers.
- **Not checked:** the game protocol (the post-auth map/agent protocol). It is large. The in-game consumers
  of a world id that I found all go through the same content-373 lookup (see §2 and §4), which is evidence
  the client does not expect a name on the wire. I did not exhaustively check it.

## 2. Name column: world id to text (exact chain)
**Grid fill** (`0x14086b9e0`, from create handler `0x14086b3dc`) loops `0x14023d8e0(i)` and calls the row-add
`0x14086ab80(grid, entry)`. **V**

**Row add** `0x14086ab80`:
1. `0x14086ab92 call 0x14086e3f0`. It returns `0x14023cd90() == 2`, where `0x14023cd90` is `mov eax,[0x14266bf3c]`.
   This is the "server 8 Dword3 == 2" gate. **V** for the code; **I** that `0x14266bf3c` is server 8's Dword3
   (taken from the existing doc).
2. If the gate is set, it scans content type 373 (`0x14086abab mov edx, 0x175`; the walk uses vtable
   `+0x48` on the object at `[rax+0xe0]`) and compares `[rec+0x28]` with the id (`0x14086abd0 cmp [rax+0x28], edi`).
   If the id is not found it returns 0 and no row is added (`0x14086abed xor eax, eax`). **V**
3. If the gate is clear (`jz 0x14086abfb`), no content check is made and the row is added. **V**
4. The row is created (Id `[r15]`, population `[r15+4]` via row vtable `+0x20` and `+0x18`).
5. **Name cell** (callback `0x14086ff10`; its source file is `SsdGridCellName.cpp`):
   ```
   0x14086ad6b  mov  ecx, [r15]          ; world id
   0x14086ad6e  call 0x140312330         ; -> wchar_t* name
   0x14086ad73  mov  rcx, [rbx]          ; rbx = Name cell
   0x14086ad76  mov  rdx, rax
   0x14086ad79  mov  r8, [rcx+0x48]      ; cell vtable 0x141b626b8, slot 9 = 0x140870670
   0x14086ad80  call r8
   ```
   **V**. `0x140870670` tail-jumps to `[label_vtable + 0x88]` (set text) with that pointer (`0x1408706bd..0x1408706cd`). **V**
6. **Location cell** uses `0x1409b1b60(id)`, which is pure arithmetic on `id / 1000`. It is not a name fetch. **V**

**`0x140312330(ecx = world id)`** (verified instruction by instruction):
```
0x14031233a  mov  edi, ecx
0x14031233c  call 0x1409b1fb0              ; content manager
0x140312346  mov  edx, 0x175               ; type 373
0x14031234b  mov  rbx, [rax+0xe0]
0x140312360  call [rax+0x48]               ; iterate type-373 records
0x140312368  cmp  [rax+0x28], edi          ; record world id == edi
0x1403123a9  mov  ecx, [rax+0x2c]          ; record text id
0x1403123b5  call 0x1412e38e0              ; = mov ecx,[rcx]; jmp 0x1410dbdd0
0x1403123c4  ret                           ; rax = wchar_t* from the text system
```
- `0x1410dbdd0` is `xor edx,edx; jmp 0x1410dbde0`, so `GetText(textId)`. `0x1410dbde0` formats the string in a
  text buffer and returns `[buf+8]`, UTF-16 and NUL-terminated (the length is `[buf+0x14]`, end = `+2*len`). **V**
  The localized string itself lives in Gw2.dat, so the original "Anvil Rock" text id is not visible statically (**I**).
- **Other callers of `0x140312330`** (16 `call` sites including the grid one, no tail-jumps) **V**: `0x1404339ed`, `0x140433a30`,
  `0x140868154`, `0x14086830b`, `0x140868439`, `0x140868591`, `0x14086ad6e`, `0x14086caa6`, `0x140911714`,
  `0x1409128f0`, `0x1409129da`, `0x14091767a`, `0x1409179c3`, `0x140917dc5`, `0x14092e422`, `0x1413dc598`
  (I did not identify every UI surface). The ones in `0x1408680xx`–`0x14086caxx`
  are in the ShardSelect dialogs (they sit beside the `Ssd*.cpp` code). **I**
- I did not find a separate character-select or in-game HUD "world label". **I** The `0x1409117xx`–`0x14092exxx` sites
  are the likeliest candidates (they call `0x140312330`, then a vtable setter or formatter).

## 3. World id not in content 373
- **V** `0x140312385` (not found): `xor r9d,r9d; mov r8d, edi; mov edx, 1; mov ecx, 0xc274; jmp 0x1410dbde0`. So it returns
  text id `0xc274` formatted with one argument, the world id. **No assert and no blank**: a fallback string is used.
  What it reads like ("World 1234"?) is in Gw2.dat and I could not see it (**I**).
- **V** Gate behaviour in the row add: when server 8 Dword3 != 2 the row is added without any content check and
  its name is that fallback. When Dword3 == 2, an id missing from 373 gets no row at all.
- **V** Location for an unknown id is benign: `0x1409b1b60` returns 0 (`0x1409b1bcb xor eax,eax`) when the id is
  outside `1000..`, and the territory index comes from `id/1000` regardless of content.

## 4. Smallest guarded patch: make id 1001 read "Banshee Realm"
**Do not touch the frozen files; this is an in-memory edit applied by the gdb runner.**

### Primary hook (covers the dialog's Name column and every `0x140312330` caller)
- Guard bytes at `0x140312330` (24 bytes): `48 89 5c 24 08 57 48 83 ec 20 8b f9 e8 6f fc 69 00 4c 8d 44 24 38 ba 75`
  (the prologue and the first `call 0x1409b1fb0` plus the start of the `edx=0x175` setup). **V** (read from the file)
- Breakpoint at `0x140312330`, condition `$ecx == 1001`. On hit:
  ```
  set $rax = <cave>           ; pointer to the UTF-16 string
  set $rip = *(unsigned long long*)$rsp
  set $rsp = $rsp + 8
  continue
  ```
  This is a manual `ret`. It leaves callee-saved registers untouched, because the function's body is skipped and
  never pushes anything before the entry hit. All 16 call sites use `rax` straight after the call. **V** for the
  grid caller (`0x14086ad73 mov rcx,[rbx]; mov rdx, rax`); **I** for the others.
- Cave for the string: `.rdata` slack, **`0x142553600..0x142554000`**. The file data ends at `0x142553600` and the
  page is mapped to `0x142554000` (PE headers: raw end `0x142553600`, mapped end `0x142554000`). It is zero-filled
  at load. Guard: read 32 bytes at the cave and require all zero, write, then read back and compare.
  Use `0x142553600` for the string and keep the rest. (Any equivalent 28-byte zero area works; if you want to stay out of
  data pages, `.text` has `0x141913200..0x141914000` of zero tail.)
- String bytes (UTF-16LE, NUL-terminated, 28 bytes):
  `42 00 61 00 6e 00 73 00 68 00 65 00 65 00 20 00 52 00 65 00 61 00 6c 00 6d 00 00 00`
- gdb writes through ptrace, so the read-only page is not a problem. Under Wine, check the readback anyway.

### Secondary hook (only if "Anvil Rock" still appears in a chat or party style label) — **I**
`0x1409a70d0..0x1409a720e` resolves a world id without `0x140312330`: it matches `[rax+0x28]` against `edi`
(`0x1409a7175 cmp [rax+0x28], edi`), then `0x1409a71cc mov eax,[rax+0x2c]`, `0x1409a71d8 call 0x1412e38e0`,
and formats the result with text id 13/14 (`0x1409a7209 call 0x1410dbc20`). If it matters, put a breakpoint at
`0x1409a71dd` (the instruction after the call), condition `$edi == 1001`, and set `$rax = <cave>`. I have not
confirmed that `rdi`/`edi` survives the `0x1412e38e0` call (callee-saved on Win64, so it should), and I did
not identify what UI uses it.

### Alternative: patch by text id (broadest, slowest)
`0x1410dbdd0` is the entry for every text fetch (and `0x1412e38e0` jumps to it). A conditional breakpoint
`$ecx == <textId>` would change every place the string is shown, including paths I did not find. The text id of
world 1001 is not in the binary. To find it, add a read-only breakpoint at `0x1403123a9` with condition
`*(int*)($rax+0x28) == 1001` and log `*(int*)($rax+0x2c)`. Caveat: the text system is called very often, so a
conditional gdb breakpoint there is expensive; I would use it only as a one-shot discovery, not as the patch.

### Risks
1. **Text lifetime.** The original return is a transient buffer; the cave string is permanent. That is safe unless a caller
   frees or edits the pointer. None of the callers I read do. **I** for the callers I did not read.
2. **Callers that copy by length.** If a caller uses `wcslen`, the NUL is there. If it reads a length stored with the
   original text id it would be wrong. I saw none. **I**
3. **Hook cost.** `0x140312330` is called only when UI is built (a few times), so a conditional breakpoint is cheap.
4. **Other ids.** The condition limits it to 1001. The other ids (1002–1059, 2001–2039) are untouched. If the emulator
   ever offers a second world, add a second guarded rule; the same code works with a different id and string.
5. **Wine/CET.** Skipping the function by writing `$rip`/`$rsp` is fine without shadow stacks (Wine does not enable them).
6. **Fallback.** If the guard bytes do not match (a different build), do not apply the patch.
7. **Text markup.** The original text goes through a parser validation (`TextApi.cpp:694`, `CParser::Validate`).
   "Banshee Realm" has no markup characters, so a raw string is fine.

### Breakpoint-free variant (jmp stub) — **I**, not tested
`tools/gw2-runner/gw2-runner.gdb` stays attached and uses Python `gdb.Breakpoint` subclasses (`TrustPatch`, `DhPatch`)
that write memory in `stop()`, so the breakpoint form above fits it (write the `$rax/$rip/$rsp` changes in `stop()`
and return False). If a pure byte patch is preferred:
- The 12 bytes at `0x140312330..0x14031233b` (`48 89 5c 24 08 57 48 83 ec 20 8b f9`) have no RIP-relative operands, so they
  relocate cleanly. The next instruction (`call 0x1409b1fb0`) starts at `0x14031233c`.
- Overwrite them with `jmp stub` (5 bytes) + 7 NOPs. Put the stub in the `.text` zero tail (file data ends at `0x141913200`,
  VirtualSize ends at `0x14191302c`; use `0x141913040..0x141914000`, which is r-x).
- Stub: `cmp ecx, 0x3e9` / `jne L` / `lea rax, [rip+str]` / `ret` / `L:` the 12 original bytes / `jmp 0x14031233c`; the
  string follows the stub. Guards: the entry bytes match and the stub area is all zero; read back after writing.

### Alternatives considered
- Sending a different world id. **There is no evidence in the binary that any content-373 id carries the name "Banshee Realm"**:
  names live in Gw2.dat as text ids and are not visible statically. The discovery breakpoint at `0x1403123a9` (see above) would
  dump the `+0x2c` text id for each id; resolving each to a string would settle it. Until that is known, assume the client patch is needed.
