# Build 209026: creation behavior data

Static REA findings, 2026-11-01, on the frozen executable identified in
[client-build.md](../client-build.md). This narrows server 37's auxiliary
data requirements on the **missing-agent, type-0 character creation** path.
It does not establish behavior payload semantics for existing agents or
other agent types. No client replay was performed.

## Container and wire fields

Server 37 fields 3 and 4 are the Dword and variable buffer copied by
`0x141035c50` into command +0x28. The allocator's assertion says
`bytes <= AGENT_BEHAVIOR_MAX_BYTES`, with maximum 250. Calling this
container agent behavior data follows that client assertion.

| Container offset | Initialization/use |
| --- | --- |
| +0x00 | Dword overwritten from packet field 3. |
| +0x04 | Byte count, stored as u32; field 4's decoded count is loaded as u8. |
| +0x08 | Eight inline payload bytes, or a heap pointer when count exceeds eight. |
| +0x10 | Zero-initialized state checked by assertion `!m_bigEndian` before copying. |
| +0x18 | Zero-initialized additional storage; not interpreted by this trace. |

Before the packet override, initializer `0x141023f10` produces a locally
incremented Dword with its high bit controlled by the agent context type.
This is separate from the command's independently allocated local ID at
+0x4c. The trace does not assign an authoritative semantic name to the
packet Dword.

Copy helper `0x141024710` calls resize/access helper `0x1410244d0` and
copies exactly the supplied length. Zero is admitted: the fresh container
already has count zero, so it returns its inline storage. The copy wrapper
`0x1409b7320` jumps to `0x1418e9ae0`; its count-zero switch entry at
`0x1423b62a0` points to `0x1418e9b0e`, a plain return. It does not read the
source pointer on that branch. Counts up to eight use inline storage;
larger counts allocate. Destruction frees only the larger allocation.

## First character creation does not interpret the payload

With command +0x20 null, immediate execution `0x14103e880` invokes the
creation virtual +0x10, `0x141039b10`. This forwards the behavior-container
address to the base-agent constructor `0x141024aa0` as argument 13.
That constructor passes it as R8 to the selected agent factory at
`0x141024ef6`.

For agent type 0, the factory is `0x1411b66e0`. Its complete function
does not read or preserve the incoming R8. It obtains the base agent from
RDX's interface, constructs the character via `0x1411d2810`, inserts the
character into its context table and returns the character +0x48 interface.
The character constructor is called with only its allocation and base
agent. The behavior container is not copied into it.

Back in immediate execution, the creation command's fixed mask 0x10000
selects virtual +0x20, `0x140246e90`, which invokes its deleting destructor.
For server 37 this is `0x141039140` → `0x141024ff0`. The latter unlinks the
command, destroys the behavior storage through `0x141023f60`, and removes
its timer registration. This completion path also does not interpret the
behavior Dword or payload.

Consequently, **field 3 = 0 and field 4 = empty have no observed payload
dependency on this first-character-creation branch**. This removes the
need to invent behavior bytes for an initial offline fixture. It does not
validate the other creation fields, world initialization, player/component
population or finalization, and it has not been confirmed in a game replay.

## Existing-agent limitation

When command +0x20 already names an agent, `0x14103e880` takes a different
path. It can pass the behavior container to the game component's virtual
+0xa0 predicate, and later dispatches the command to agent virtual +0x160.
Other command executors also forward this container to component callbacks.
The first-creation finding must not be generalized to those paths or to
the interpretation of nonempty behavior payloads.

## Evidence

Private bundle: `~/re/gw2/notes/209026-agent-behavior-data-1.json`.

| REA evidence | Observation |
| --- | --- |
| `ev_6edcb051d6d24150ed0739c9a1288a5d2f19891329a0b9e96b902e6c03ce3780` | Command/container construction and packet field copies. |
| `ev_4ed670fb24de7cb11c600c1a5d739894ea30fcd50dd057d54dab226623558b8a` | Container initialization, resize and forwarding through base creation. |
| `ev_bfe1b609775a88ad994926a0d316504f5bc831543457b42c34bbd2291955c332` | Complete type-0 factory, character constructor arguments and named assertions. |
| `ev_7667095495ee1b547823d0e8eb1d12f64713abcdc012f74a6ddbd9e50ce5d21d` | Missing-agent versus existing-agent execution and factory dataflow. |
| `ev_326757566e78bd1ccc431c4c1dee164b044344aa65d5bcb92f21b44e0144b5f1` | Command vtable, completion slot and copy/allocation helpers. |
| `ev_2b4f19cf8b8ffdaff49d603f1474ecd4db8bc668e093a50ebd8c69ad0e8d515a` | Deleting destructor chain and copy-wrapper target. |
| `ev_bab5592ae8fdf53fff8f2e80c83c1b4efe444e3970a52813cbacd734bb2eb60a` | Behavior destructor, timer unlink and small-copy switch. |
| `ev_4fa8a1a64f6c97b6e759e5e2f58ff180629f5679b0521fc727c71875387c76c1` | Zero-length switch entry points directly to return. |
