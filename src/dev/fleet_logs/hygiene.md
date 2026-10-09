# Fleet `hygiene` — progress log

Mission: file management, docs sprawl, debug_runs policy, line endings, consistency/reliability
in shared code (`src/utils`, `src/core`, `src/traits`, `src/io`, `src/events`).
Charter: [`../fleet_charter_20261008_v1.md`](../fleet_charter_20261008_v1.md).

## 2026-10-08 — slice 1: line endings

- Added `.gitattributes`: `* text=auto eol=lf`, `*.cmd`/`*.bat` `eol=crlf`, explicit text for
  source/data types, explicit binary for images/meshes/blends/fonts/archives.
- Index was already 100 % LF (`git ls-files --eol`: 0 × `i/crlf`), so no blob changed; the
  normalization is the attribute file itself.
- Working tree: rewrote 5 347 clean CRLF files to LF in place (per-file guard: only when the LF
  bytes hash to the staged blob, so no other fleet's edit was touched), then refreshed index stat
  info for the 5 129 hash-identical paths (`git add` of identical content → nothing staged).
- Before: 5 470 `i/lf w/crlf` files warning on every touch. After: LF everywhere except files
  being rewritten by CRLF-emitting tools (Python text-mode writes on Windows) — those normalize on
  commit.
- Found: `tools/mcp/python/rust_engine_mcp/validators/mcp_schema.py` is stored with `\r\r\n`
  (git sniffs it as binary). Tooling-owned → board note.
