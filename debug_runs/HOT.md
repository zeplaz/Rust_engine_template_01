# HOT witnesses — read these only (default)

Agents: **do not** glob `debug_runs/**` or open historical `*_live.json` unless the active slice names one. Archive lives outside this tree: `_local_archive/debug_runs/2026-10/` (gitignored).

**Git:** only this file + `README.md` are tracked. All `*_live.json`, PNG, logs, `art_pipeline/`, etc. are **local-only** (`.gitignore` `/debug_runs/**`).

## Always OK (freshness tools)

| Path | Why |
|------|-----|
| `agent_ops/ops_report_latest.json` | Ops rollup (lane close only) |
| `agent_ops/ops_project_brief_v1.json` | Compressed project brief |
| `agent_ops/minimal_harness_adoption_live.json` | Lean-path adoption |
| `unified_witness_index.json` | Index — prefer over scanning folders |
| `agent_debug_index.json` | Sim spine index |

## Active product (pick from HANDOFF; refresh when closing)

| Path | Lane |
|------|------|
| `stage5_full_app_live.json` | Stage5 FULL_APP regression |
| `construction_stage_live.json` | Construction |
| `fire_streaming_live.json` / `fire_ecology_live.json` | Fire |
| `g_play_product_close_live.json` | G-PLAY close (if present) |
| HANDOFF **ACTIVE** program witness | Whatever the lease names |

## Regenerated at runtime (empty dirs OK)

| Path | Notes |
|------|-------|
| `preview_jobs/` | Worker write target — pile archived |
| `validators/` | `validate-report` raw logs — pile archived |

## Do not read by default

- `_local_archive/**` (outside this folder)
- `art_pipeline/**` bulk history unless closing that art batch
- Root `aps_*_live.json` / closed-program dumps unless the queue row cites them
- `.witness_honesty_precheck` (archived)
