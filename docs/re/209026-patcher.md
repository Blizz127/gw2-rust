# Build 209026: launcher patch step (asset CDN, root manifest, manifests)

The launcher won't go past "Downloading 0KB" until its patch step is satisfied. Our
offline asset CDN (`gw2-assetcdn`) and manifest generator (`gw2-manifest-gen`)
reproduce that step from the local `Gw2.dat`. Evidence IDs are REA results on
`Gw2-64.exe` sha256 `99342078…8621`. Runtime results come from the `--network none`
sandbox (runs 22–31; logs are private in `~/gw2-sandbox/logs`).

## Observed

**1. `/latest64/101` reply** (`FUN_140d8c630`, ev_2e70a2e9…). The client copies up to
127 bytes of the body and parses five integers with `strtoul`:
`build exeFileId exeSize rootManifestFileId rootManifestSize`. A missing field becomes 0.
The request path is `/%s/%u` with `latest64` or `latest` (`FUN_140d8ee90`, ev_123b41af…).

**2. Root manifest load** (`FUN_140fdd6e0`, `ApManifest.cpp`, ev_ddaf2fb3…):
- It tries the local copy first (`FUN_140d92b40`).
- Otherwise it allocates exactly `rootManifestSize` bytes and queues a download. If the
  download fails it asserts `Failed to load root manifest.` (line 133).
- The packfile signature must be `ARMF` (`FUN_140dec6e0`); then it parses it.

**3. Download URL** (run 24): `GET /program/101/1/0/<fileId>/compressed`, then
`GET /program/101/1/0/<fileId>` (raw), retried on 404. Serving raw bytes on the second
path works.

**4. The client caches downloaded manifests in its `Gw2.dat` by file id** (run 26).
Serving different content under the same id later trips
`m_readOffset + m_readLength <= m_totalBytes` (`FileArchive.cpp:1038`). Use a fresh
id whenever the content changes.

**5. Packfile pointer width** (`FUN_140de8eb0`, `Packfile.cpp`, ev_cee19f57…). The 64-bit
client widens 32-bit packfiles on load and asserts `pointers.Count()` (0x27b) when such a
file has no fix-ups. Write manifests as 64-bit (header flags 5, as in real `mapc` files)
and no widening happens. 32-bit files use flags 1, as in real `STAR` files. The fix-up
table is `u32 count` followed by the `u32` payload offsets of the pointer slots.

**6. Root → preparers** (`FUN_140d93070`, ev_982102d1…). Each `manifests[]` entry is
24 bytes in memory: `baseId, fileId, size, flags, name*`. The client creates one
preparer per entry, keyed by `baseId`.

**7. Manifests are looked up by name** (`FUN_140d92d00`, a wide string compare):
- `L"Launcher64"` (`FUN_140236cb0`, ev_9eed32af…);
- `Map<name>` (`FUN_1413ed150` → `MhHelper` `FUN_1413ed1b0`);
- `Map…` and `Region<name>` from map definitions (`FUN_1413fe2a0`, `FUN_1413fe340`).

A missing preparer asserts `preparer` (`MultiManifest.cpp:988`).

**8. A manifest with no records** asserts `manifest->records` (`MultiArchiveTrim.cpp:236`)
(run 29).

**9. Run 30/31:** a `Launcher64` manifest listing every local asset (819,347 records)
loads without an assert. The launcher then queues every record for download in order
(ids 8192, 8193, …), and the UI shows "Downloading 0KB". Switching `size` from stored to
decoded size (run 31) did not change this.

**10. Why everything was queued for download** (REA, sub-investigation;
`FUN_140d94910` ev_4c62e3d0…, `FUN_140d92980` ev_bb6a2f49…, `FUN_140d943b0`
ev_90256eab…, `FUN_140d95270` ev_cab3e33c…). The client keeps an install table keyed by
`baseId`: `{baseId, targetFileId, installedFileId, size}`.
- It fills the table from its archive's file table, 6-dword entries `{…, stream (byte),
  baseId, fileId}`. Entries with `stream != 0` log "unexpected stream" and are skipped.
  When an entry's actual size doesn't match the cached size, it logs a size mismatch and
  deletes the file.
- A record needs download when `installedFileId != record.fileId`. A missing table entry
  counts as installed = 0.
- Finishing a download sets target = installed = fileId.
- Record and root `flags` are not read on that path.

**11. The archive trimmer** emptied the working copy, while the frozen baseline stayed
untouched. After run 27, which used a root manifest with no manifests, MFT entry 1 (the
file index) of the working copy's `Gw2.dat` was all zeros. So runs 30–33 tested against
an empty archive, and their "everything missing" result is explained by that, not by the
record format. Recovery: `sandbox/run.sh restore 209026`.

## Inferred / open

- **Base and file ids:** of the 819,347 MFT entries the index references, 304,885 have
  two ids and 514,462 have one. We record `base = smaller`, `file = larger`. Not
  confirmed.
- **Next run:** restored `Gw2.dat` plus a full-archive `Launcher64` manifest. Expect no
  downloads if `base = smaller id` matches the archive's file table. Check the index
  afterwards (no trim).
- **Exe self-update:** the latest-reply exe fields (1, 2) are 0. No exe download has been
  observed so far.
