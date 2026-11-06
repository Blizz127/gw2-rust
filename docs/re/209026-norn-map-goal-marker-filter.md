# Build 209026: map-reference filter in quest goal markers

Static REA, 2026-11-06. The selected lead was current map definition
**+0x108 → referenced record +0x68**, read by the known quest manager.
The bounded consumer proves a pointer-equality filter within a candidate-goal
marker path. **It supplies no Great Hunt actor, encounter condition or
action-to-step rule.** Quest 154's sole goal 1 is excluded by the path's
explicit count-zero guard, independently of that map reference's value.

## Complete consumer and exact guards

Chained unwind metadata resolves the enclosing method to
**0x141202bb0**. Captures cover its complete body through the return at
**0x141203054**; the source label is `ChCliQuest.cpp`.

The method exits when manager+0xe4 is nonzero, its virtual+0x70 predicate
is false, or current map definition+0x2c equals 4. It asserts
`m_activeQuest != nullptr`, obtains the active quest's definition and goal,
then passes definition+0x38 to the service returned by **0x141466a80**,
virtual+0x10. A null result exits. This slice does not rename that service
or expand every helper it calls.

At **0x141202ccd..0x141202cf7**, it obtains the current map definition
through the established **0x141411100** getter, reads map+0x108 when
nonnull, then referenced record+0x68. A missing map or first reference
produces null. That pointer is saved for later comparison; no actor or
numeric content key is extracted from it.

The subsequent loop visits candidate quest definitions and their inline
goals at +0xa8/count+0xb0, stride 0x70. At
**0x141202dc3..0x141202dd3**, it skips a goal when either:

- it is the active goal pointer; or
- its **Dword+0x20 equals zero**.

For other goals the service's virtual+0x18 enumerates candidates and
supplies position/map outputs. For each nonnull candidate, the code follows
**candidate+0x40 → map+0x108 → referenced record+0x68**, using null
when either pointer is missing. At **0x141202e41..0x141202e46**, an
inequality with the saved current-map pointer skips to the next candidate.
The field's broader content meaning remains unnamed; pointer equality
alone does not establish an event, encounter, region or actor category.

The matching path converts the returned position using **0x1413f9a30**,
checks existing objects through service virtual+0x50, and invokes virtual
+0x98 with a content pointer asserted as **`markerDef`**, the candidate
quest and candidate object. A nonnull returned pointer is appended to the
manager's +0xd0 collection. This is the inspected marker-related path;
the downstream virtual methods are not claimed to recover encounter rules.
No direct quest-step completion or actor-construction call was identified
in this body, and unexpanded callees are not a client-wide absence proof.

## Exact cached Norn applicability

The cache-only reader admitted containers 1282830 and 1282833, disabled
archive fallback, checked frozen hashes, verified quest/map type and data
keys, and checked the map's indexed extent and declared relocation.

| Exact record | Verified field |
| --- | --- |
| Quest 154, 1282830:4145560 | One goal at 1282830:4146200. |
| Goal 1, 1282830:4146200 | Numeric ID+0x10=1; pointer+0x18=0 with no relocation; count+0x20=0. |
| Map 379, 1282833:3863896, indexed extent 552 | +0x108 externally references 1282838:2998456. |

The referenced container 1282838 was **not opened or extracted**. It is
unnecessary to establish goal 1's count-zero rejection. The consumer also
skips the active goal explicitly, so this path does not establish how the
current tutorial step's actor or objective is chosen.

The missing edge remains a typed authored or authentic
**actor/action/condition → quest 154 step** rule. The goal's zero array,
map-reference equality, returned marker positions and other story-group
goals cannot substitute for it. This slice stops at that bounded result;
it neither repeats the fixed-map table/key searches nor concludes that all
client content lacks encounter data.

## Evidence

Frozen PE SHA-256:
`99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621`.
Cached source hashes remain 1282830
`e5d11e4fae8627d3efced749e7ab599b26abac59e502716aaf7d63ef509bb81f`
and 1282833
`5ab847328dd307407d9701711890f08282f9c6f3639d24bb9684ea46006e26ef`.

Full private bundle:
`~/re/gw2/notes/209026-norn-map-goal-marker-filter-20261106.json`,
**116623 bytes**, SHA-256
**991db0c264c62e3c0d0f7a32f9f97f24000e030d183c35d5cd5ef92793ba1f97**.

| REA evidence | Scope |
| --- | --- |
| `ev_a929532870ff817c28455bf709c00f06a9d5e07e88c88e2c1dcb2a2dd16a4d45` | Current-map reference chain and named marker assertion. |
| `ev_a9f61fbea28e6ea5495ed8347ed522b49bf3983a3a797b726975698881d2ced6` | Goal guards, candidate pointer comparison and continuation through return. |
| `ev_e73f2f94f9938154a9f57f1988cea99c79ef248fc8c342ca03ce421f80fe0158` | Exact cache-only quest 154/goal 1/map379 fields and source hashes. |
| `ev_7eba3011470677242fbb9f37a3b13c941016e6b183aabb11539931774ff75982` | Chained unwind entry, initial guards and active-quest assertion. |

All four captures exited zero with complete raw output and verified owned
helper-process cleanup. Some cumulative rendered-terminal frames were
omitted; neighboring functions outside the stated complete body remain
uninterpreted. No archive, client, container, resource lease, publisher state,
gameplay fixture or Rust code changed. No runtime validation is claimed.
