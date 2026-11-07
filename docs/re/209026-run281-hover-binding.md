# Run281: hover confirmation and controller refusal

Run281 used the explicitly granted `10803b4-efresume` snapshot under window71.
The live server advanced independently; this run does not test those newer
releases. It preserved the normal stage0 fixture, 126 story arguments, blank
tutorial heart, original synthetic character, and production input behavior.

Root reviewed five controller images. The initial scene and three candidate
hover images did not show a readable Thora label. A subsequent passive shot
showed **Veteran Thora Griffonbane** over the candidate. The last hover and
passive shot retained pointer `[647,397]`, the same owned client identity and
the same 3182-byte log snapshot. This establishes a later visual observation,
not exact native input or rendering latency.

Root then attempted to select Thora from that passive scene. The frozen
controller requires the current scene image to equal the successful hover
action's image. The passive image was different, so that assertion rejected
the action before focus admission or native input. The click record reports
both native-input flags false. Root released incomplete without retrying.
No selection, interaction, movement, vector pulse, quest step or combat
occurred. This result neither demonstrates nor diagnoses a walking failure.

The smaller correction is procedural. Reserve one of the three hover slots:
after a passive shot identifies the NPC, perform a fresh same-coordinate
hover, review that new hover image, and click only if it still visibly
identifies the intended NPC. This preserves the existing exact-image,
pointer, ownership, quiet-log and freshness checks. A prior passive label
cannot replace a missing identity in the final hover image. No controller
or reader semantics were changed, and run281 remains closed.

Window71 returned **2026-11-07 00:42:41.052004Z**. UI and outer capture exited13;
the strict reader exited0 with valid incomplete evidence. Cleanup verified
container absence, all83 immutable non-prefix inputs, unchanged154-byte
synthetic record, frozen and working executable hashes, and console privacy.
Unverified process-environment observations in the outer capture were not
treated as ownership of those unrelated processes.

Private evidence under `~/re/gw2/notes/`:

- Root observation: `run281-root-hover-refusal-observation-20261107.json`,
  SHA256 `1cc4498c1930244ec0437a8bddc2d11a4d2ef643a6dbc1d05568860ce10c4a3a`.
- Passive image capture: `ev_e637679bb64be1afcb9d7f0383efee6c93dc4fa7bbfa3c91a9ed137a2f628a9b`.
- Refused click: `ev_792bd2ad4f05cf2ec4b28f055dca78f46b96651319150621345bac5436218249`.
- Strict readback: `209026-run281-norn-thora-trophy-ido-readback-20261106.json`,
  SHA256 `bd1a05188680792ce1154e8a1e01043126a1478482de39ae7e52144a580105be`,
  `ev_3ed0a9b6b849bdb954bb2a93be8bd440579d9b41d43844677af0da73136fa1f2`.
- Resource return: `209026-run281-resource-return-20261106.json`,
  SHA256 `18dfcfc4d7a5b3ffcddce1b50aacfd500d6faf1f22ec700a4acb7adece951add`,
  `ev_504575c6b9c9742a65d91d2ae82bdaf7d7a34657d5de168e04a522663c929370`.

The return pins the completed launch capture and exact readiness receipt.
No production, owner-store, schema or launcher change came from this run.
