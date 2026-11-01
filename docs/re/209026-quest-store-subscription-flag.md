# Build 209026: quest store subscription flag readers

Static REA follow-up, 2026-11-01. This closes the last open gate on the
quest-to-store chain. The startup callback subscribes the quest object's
+0x20 interface to the context +0xc8 store service and sets **quest +0xf0
bit 0x10**. If the emulator never answers GemStoreSearch, no item result
arrives, and the bit stays set. Question: does anything read that bit as a
wait condition?

Result: apart from its setter and the item-callback clear, **only the
destructor reads bit 0x10**, to undo the subscription. No startup or
readiness reader was identified. The client remained stopped; no request,
purchase, account access or agreement acceptance occurred.

## Field layout

Constructor `0x1411fd630` installs five tables and zeroes Dword +0xf0:

| Offset | Table | Slots |
| --- | --- | --- |
| +0 | `0x142166030` | 25 |
| +8 | `0x1421660f8` | 10 |
| +0x10 | `0x142166148` | 13 |
| +0x18 | `0x1421661b0` | 35 |
| +0x20 | `0x1421662c8` | 6 |

The tables have 33 distinct targets. A method entered through an interface
addresses the field at 0xf0 minus that interface's offset; for example, the
+0x20 interface uses +0xd0. Each bit of +0xf0 records one registration of
the object with a service:

| Bit | Set | Cleared | Meaning (from the paired register/unregister calls) |
| --- | --- | --- | --- |
| 0x1 | `0x141201540` | Destructor `0x1411fd8a0`, `0x1412017b0` | Activation. Tested at `0x14120013a`, `0x14120035a`, `0x1412016fa` and the destructor. |
| 0x2 | `0x141201540` (after `0x141405350` virtual +0x58 on +8) | `0x141200ae0`, destructor | +8 interface registration. |
| 0x4 | `0x1412028c0` region (virtual +0xd0 on +0x10) | `0x141200620`, `0x1412029bf`, destructor | +0x10 interface registration. `0x141202950` loads the Dword and masks **only 0x4**. |
| 0x8 | `0x141201540` / `0x141202924` (virtual +0x1b8 on +0x18) | Startup callback `0x141200450` (asserts, then clears), destructor | +0x18 interface registration for the startup callback. |
| **0x10** | **`0x141200513`**, after `0x1412cf930` virtual +0xd8 on +0x20 | **`0x1412005fe`** (item callback, after virtual +0xe0), **`0x1411fdbf9`** (destructor) | Store-service subscription. |

The destructor reads the Dword at `0x1411fdbd1` and tests bit 0x10. Only
when it is set does it call the store service's virtual +0xe0 with the
+0x20 interface, then clear the bit. That is lifetime cleanup, not a wait.

## Sweeps

1. **Global masked-immediate grep.** Searched the full Intel listing for
   immediates 0x10, 0xef, 0xffffffef and 0xffef on displacements 0xf0, 0xe8,
   0xe0, 0xd8 and 0xd0. It returns 21 hits. Only three lie in quest-class
   code: the setter, the item-callback clear and the destructor clear. The
   other 18 were mapped through the PE `.pdata` table to their primary
   functions, following chained unwind info out of split fragments. None of
   these is a quest table target. The primaries are `0x1402c2c40`,
   `0x1402cda10`, `0x1402d0b10`, `0x1403e4090` (two hits), `0x140878ca0`,
   `0x1408f8290`, `0x14096c380`, `0x140abad50`, `0x140affbf0`,
   `0x140d4d400`, `0x140f705d0`, `0x141076930` (a stack operand),
   `0x141121d80`, `0x1413aa4e0`, `0x1414daa70` and `0x1414dac70`, plus
   the leaf `0x141027340`, which has no `.pdata` entry. The same mapping
   resolves the destructor's clear fragment to `0x1411fd8a0`.
2. **Class-bounded scan.** Every one of the 33 table targets was
   disassembled to its end. Each was checked for the interface-adjusted
   displacement and for whole +0xf0. Field-address `lea` uses were
   followed: `0x141200450` tests bit 0x8 and `0x141200620` tests bit 0x4.
3. **Code-region scan.** Scanned 0x1411f8000–0x141208000, the constructor
   through the non-virtual helpers, for any +0xf0 operand. Code before the
   constructor, and at `0x141207a55`/`0x141207cad`, uses +0xf0 only as Qword
   pointer loads or stores, consistent with other structures.
   Within the quest code, the only Dword loads are those in the table
   above.

## Consequence for the startup chain

Taken together with the
[tracker reader sweep](209026-quest-store-tracker-readers.md) and the
[Purchase callback trace](209026-quest-store-purchase-callback.md), suppose
the emulator never answers the asynchronous GemStoreSearch:
- the request tracker keeps an inert null entry
- the quest's store subscription stays registered until destruction
- no Purchase is built

None of these pieces has an identified reader that blocks startup. **No static wait on a store answer was identified under these bounds.**
At runtime, a never-answered request probably ends as a timeout error.
That error would take the 0x50 error branch, whose request-specific retry
path is still untraced. A store fixture may therefore prefer an explicit
error or empty reply over silence.
No emulator response policy is changed by this note.

The claim does not cover:
- Dword-load-then-test readers outside 0x1411f8000–0x141208000; the global grep catches only immediate-memory forms
- the store service's own handling of its listener list
- any runtime ordering

## Evidence

Private bundle: `~/re/gw2/notes/209026-quest-store-subscription-flag-1.json`
(26 Rizin records, including the tracker and Purchase sweeps). Frozen
executable SHA-256
`99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`,
unchanged after each call.

Helper scripts are in `~/re/gw2/notes/`: `flag-sweep.py`,
`quest-flag-class.py`, `region-field-scan.py` and `pdata-func.py`.

| REA evidence | Observation |
| --- | --- |
| `ev_c149d00797acf0753f16d34d42b82a7c6fd330b234bb82fe937a74cea10f7471` | Item-callback clear `and [rdi+0xd0],0xffffffef`. |
| `ev_1876c0fb1a107e5051734f270e6dec1c79e2c351c3acb2ddc46dbf735aacc013` | Constructor tables, +0xf0 initialization, destructor prefix. |
| `ev_7339d0549d6ab313d0c3edc65d6d1296562d2d1713809f2fcd5903a1d877f614` | Global masked-immediate grep (21 hits). |
| `ev_30eba6dd77077dfe3d04f20a840a1aa0cd1178846c6362f6e421efd17fa124ff` | Table slot counts, hit containing functions, class-bounded field accesses. |
| `ev_5a84e6dcb489935d9d5670e0f3a09a04917163ed70b87e185f8e7022fe259496` | Startup callback, bit-0x4 method, destructor bit-0x10 test and unsubscription. |
| `ev_bdcf6eefb31b5995840db95cf2d603549efa0c8bf9654d3c315d3ae48eda4f74` | Code-region +0xf0 scan. |
| `ev_0398065ff9080cb0b9d80f706a3c25062abee635235adbfe2b8a0ec12f2f20fd` | Activation (bits 0x1/0x2/0x8), bit-0x4 helper, destructor bit checks. |
| `ev_da5c3e5cb3a7d458d2e767babdda9671421ca7ba83c43aeeb3016ee05a0b88d9` | `.pdata` primary-function mapping of all masked hits, with chained unwind followed. |
