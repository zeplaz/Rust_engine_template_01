# Documentation archive

Superseded plans, fleet dispatches, planner audits, and prompt drafts live here — **not** in `src/dev/` or active `prompts/` paths.

## Rules

1. Every bulk move is logged in `<bundle>/MOVED_LOG.json`.
2. **Do not delete** — move only; git history preserves diffs.
3. Active work stays in `src/dev/` and is indexed from [`src/dev/development_plan_index.md`](../../src/dev/development_plan_index.md) and [`src/dev/archive_index_v1.md`](../../src/dev/archive_index_v1.md).
4. Re-run archive scripts after closing a fleet wave or prompt drain:
   ```powershell
   python tools/orchestrator/scripts/archive_stale_dev_docs.py
   python tools/orchestrator/scripts/archive_prompts_phase2.py
   python tools/orchestrator/scripts/archive_src_dev_phase3.py
   python tools/orchestrator/scripts/cleanup_empty_dirs.py
   ```

## Bundles

| Folder | Contents |
|:---|:---|
| [`lod0_tile_pilots_2026-06/`](lod0_tile_pilots_2026-06/MANIFEST.yaml) | APS headless pilot atlases (asset archive sibling under `assets/archive/`) |
| [`2026-06-fleet-drain/`](2026-06-fleet-drain/) | Closed fleet waves, planner audits v5–v15 |
| [`2026-06-prompts-guides/`](2026-06-prompts-guides/) | UI phase specs, runbooks, matrix |
| [`2026-06-src-dev/`](2026-06-src-dev/) | Closed plans, signoffs, stage trees (~437 files) |

## Reference (not archive — user/outside material)

[`docs/reference/`](../reference/README.md) — inbound art/econ notes, large drafts, legacy C++ maps, terrain/transport Q&A packs.

## Build artifacts (not archived — deleted)

Alternate Cargo target dirs (`target_fleet_*`, `target_coder_*`, …) at repo root are **gitignored** (`/target_*/`) and safe to delete locally (`cargo clean` does not remove them).
