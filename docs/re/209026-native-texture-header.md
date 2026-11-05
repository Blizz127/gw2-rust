# Build 209026: native texture header dimensions

For an ordinary little-endian ATEX-family asset payload, the frozen native
decoder reads the format FourCC at byte 4, width as unsigned 16-bit at byte 8,
and height as unsigned 16-bit at byte 10. These are file-header dimensions,
before the texture load task's later reduction path. This contract permits
a bounded comparison with a runtime source texture; it does not validate
the complete compressed image, its mip payloads, or its rendered pixels.

The investigation read only frozen `Gw2-64.new.exe`, SHA-256
`99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`,
through REA. No archive, game runtime, container, or production state was
accessed or changed. Asset extraction remains a separate task.

## Exact route and offsets

Load-task continuation `0x140a6f490` reads its image decoder from task
`+0x40`. It copies decoder Dword `+0x44` to task format `+0x58` and decoder
Qword `+0x38` to task dimensions `+0x68/+0x6c`. These decoder offsets are
**in-memory fields**, not file offsets. See the
[full-texture branch analysis](209026-grtex2d-operations.md).

Its exact caller at `0x140a6f2f0` advances that same decoder using
`0x140b160d0` at `0x140a6f0c4`. The latter identifies ATEX-family magic
through `0x140b86a30`, initializes decoder file type zero, and dispatches
`0x140b84310`. Initializer `0x140b84290` creates private state zero;
the first four state-table targets at `0x140b868dc` are:

| State | Target | Input consumed | Result |
| --- | --- | --- | --- |
| 0 | `0x140b843cc` | Four bytes | Signature and header flags |
| 1 | `0x140b84544` | Four bytes | Format FourCC → decoder `+0x44` |
| 2 | `0x140b84974` | Two bytes | Zero-extended width → decoder `+0x38` at `0x140b84a1f` |
| 3 | `0x140b84a2c` | Two bytes | Zero-extended height → decoder `+0x3c` at `0x140b84adf` |

The dispatcher replays its initial four signature bytes through decoder
`+0x74/+0x78`; they are not an additional four bytes before the header.
Thus a diagnostic reader needs at least twelve decoded asset bytes. It
should report the raw signature, FourCC, and the two dimensions separately
from any runtime texture or requested pool dimensions.

## Signature and format constraints

For `M = u32le(payload[0:4])`, the native predicate accepts exactly:

`(M & 0xfffffff9) in {0x58455441, 0x58545441, 0x43455441,
0x50455441, 0x55455441, 0x54455441}`.

The normalized tags are `ATEX`, `ATTX`, `ATEC`, `ATEP`, `ATEU`, and `ATET`.
The masked low bits `0x2` and `0x4` become decoder flags `0x8` and `0x10`;
the normalized `ATEC/P/U/T` variants additionally set `1/2/4/0x20`.
Do not reject those flag variants by requiring only literal `ATEX`, and do
not assign further semantics to the flags from this slice alone.

The relevant exact FourCC mappings are:

| FourCC | Native format |
| --- | --- |
| `DXT1`, `DXT2`, `DXT3`, `DXT4`, `DXT5` | 22, 23, 24, 25, 26 |
| `DXTA`, `DXTL`, `DXTN`, `3DCX` | 27, 28, 29, 30 |
| `BC5X`, `BC7X` | 31, 32 |

Other recognized formats also exist. A minimal reader should preserve an
unmapped FourCC as unknown rather than guess its native format. The native
decoder has byte-swapping paths when its platform field is 5 or 6; this
offset/endianness contract specifically describes ordinary little-endian
payload bytes. It does not apply these offsets to another image family or
to raw archive-compressed member bytes.

If a correctly identified extracted payload itself reports 512×128, that
proves its header declares that size before this task's reduction step.
It does not by itself prove which runtime acquisition used the asset, rule
out cache reuse or later mutation, or validate all image data.

## Evidence

Private complete records:
`/home/blizz/re/gw2/notes/209026-native-texture-header-20261105.json`,
SHA-256 `cff669b1d4c6bf8fb8b1b43bb62b0fa534caae6426cab65c58c2a8a4aab1ea8e`.

| Evidence | Finding |
| --- | --- |
| `ev_beaa554182f0b4a3a6b6aaf3108afb9346eaa2b5a0c68db82cbd8d07e012af9f` | Exact load-task caller and image-decoder advance |
| `ev_29a477d9155ff6036e7aed45d2a0577f536e6c8f776be67d8d495acbeba2828b` | Signature dispatch, initial-byte replay, decoder platform distinction |
| `ev_4f214e913485a711a45bef236aaa2348081cc20df24d7b8c7390816e3cb08aa4` | ATEX private state initialization |
| `ev_fad00a02f656f527d951b9414dbbfb0995003915fe4fb9c6decf29a49d0e8ca9` | Complete signature predicate |
| `ev_b7220645ebf7c1ef28062aab599f479827d3f14b52da35b675e37f967e34bd32` | Bounded complete header-state instructions, relevant format mappings, dimension stores |
| `ev_25e8a03a6217384f58a92f8ddb24b93bc14bdba81c33d0b43ae9940ecffda562` | Exact state-table entries and invalid-header diagnostics |

The header capture retains every raw and normalized PTY frame (14,422 bytes,
zero omitted normalized frames); its `truncated` indicator refers only to
omitted cumulative rendered terminal states. An earlier 4 KiB decoder window
is likewise retained with complete raw output, but is not a complete decoder
body. No additional decoding semantics are claimed from that partial window.
All captures exited zero and verified owned-process cleanup. Unrelated
unreadable process-environment entries were not investigated.
