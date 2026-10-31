# REA — required for all GW2 client reverse engineering

Owner direction (Blizz, 2026-10-31): every claim about the shipped GW2 client
(`Gw2-64.exe`) goes through REA, the same reverse-engineering tool the
WoW Forever (wotlk-forever) and Vanilla Forever (benilla) leads use. That
covers:

- where the client keeps or derives its login/session/crypto keys
- message (packet) layouts, opcodes, framing and encryption
- the login flow: launcher → auth/portal → character select → map server

Ordinary analysis of our own Rust source does not need REA. Use REA whenever a
claim depends on the shipped binary, its decompilation or its runtime
behavior.

## Where REA lives on the devbox (`sbdevbox`)

| Item | Path |
| --- | --- |
| CLI / MCP server | `/home/blizz/.local/bin/rea` (rea-agents 6.8.0, Node at `/home/blizz/.local/bin/node`) |
| Agent skill | `~/.claude/skills/reverse-engineer-anything/SKILL.md` (also `~/.agents/skills/…`). Load it before the first REA call. |
| Ghidra (native provider) | `GHIDRA_INSTALL_DIR=/home/blizz/re-studio/ghidra_12.1.4_PUBLIC` |
| Java | `JAVA_HOME=/home/blizz/.local/opt/jdk25` |

Non-login shells lack `~/.local/bin` on PATH (`env: 'node': No such file`).
Run `export PATH=$HOME/.local/bin:$PATH` first.

## Preferred: the REA MCP tools

Claude Code (`~/.claude.json`) and Codex (`~/.codex/config.toml`,
`[mcp_servers.rea]`, tool timeout 450 s) already register REA as
`/home/blizz/.local/bin/rea mcp` with the Ghidra/Java env above. In those
agents, use the `rea` tools directly. Start with `open_binary` on `Gw2-64.exe`,
then use search, xrefs, decompile and trace-native-values. Other clients
(Cursor, Gemini, Grok Build) show `config_drift` in `rea doctor`. Do not run
`rea setup` without approval. Use the CLI instead.

## CLI fallback

```bash
export PATH=$HOME/.local/bin:$PATH
export GHIDRA_INSTALL_DIR=/home/blizz/re-studio/ghidra_12.1.4_PUBLIC
export JAVA_HOME=/home/blizz/.local/opt/jdk25
EXE=/absolute/path/to/Gw2-64.exe          # pin and record its SHA256
SNAP=$HOME/re/gw2/notes/gw2-64.snapshot   # keep evidence private, outside the repo

rea analyze "$EXE" --provider ghidra --snapshot "$SNAP" --format json > ~/re/gw2/notes/analyze.json
rea search  "<string or symbol, e.g. a portal/auth hostname>" --format json
rea xrefs   0x<address> --format json
rea decompile 0x<address> --format json
rea trace   "<literal feature>" --format json
rea <command> --help        # exact options; `rea --help` lists every command
rea doctor --json           # only when REA will not start or tools are missing
```

The first Ghidra import of a 64-bit client this size is slow. Reuse the
snapshot and don't start a cold re-import.

## Rules (same as the wotlk-forever lane)

- Keep **observed / inferred / unknown** separate. Cite the REA Evidence ID
  (`ev_…`) or the retained JSON filename for every observed claim.
- Keep raw Evidence and disassembly windows private (`~/re/gw2/notes/`). Commit
  only findings docs under `docs/re/` that reference them, following
  `wotlk-forever/docs/forever-re/*.md`.
- Route each unresolved RE blocker through REA first. If REA cannot decode a
  format, use a bounded format-aware fallback, label it non-REA, and record
  the blocker in a ledger (see
  `wotlk-forever/docs/forever-re/70450-rea-blocker-ledger-20261030.md`).
- Report where REA was actually applied. Do not relabel source-only or
  synthetic fixtures as shipped-client observations.
- REA's runtime tools (capture-process, debug-with-gdb, Frida) launch or attach
  to the client. Use them only when the mission allows client execution, and
  confirm that cleanup finished.
