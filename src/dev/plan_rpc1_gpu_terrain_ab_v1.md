# RPC-1 A+B — GPU terrain bake + honesty freeze `v1`

**Program:** PLAN-RENDER-PROD-CLEANUP-v1 · Phase 1  
**Decision:** A **and** B together (operator 2026-07-06) — build real GPU world-terrain bake **and** make names/docs honest as it lands.  
**Queue:** `tools/orchestrator/queues/render_production_cleanup_queue.json`  
**Honesty lint:** `python -m rust_engine_mcp.cli terrain-honesty-lint` → `debug_runs/terrain_honesty_lint_live.json`  
**Parent plan:** [`plan_render_production_cleanup_v1.md`](plan_render_production_cleanup_v1.md) · [`plan_gpu_terrain_production_exec_001_v1.md`](plan_gpu_terrain_production_exec_001_v1.md)

## End-state (honest)

```text
TerrainSourceMode ≜ { CpuRaster(diagnostic|editor), GpuBake(default release) }
GpuTilemap         ≜ deferred (DR-MIG-TILEMAP) — never constructed today
Fire / weather     ≜ LIVE overlays — never rebaked into terrain texture each tick
Minimap label      ≜ world_raster (default) · gpu_bake when TERRAIN_GPU_BAKE_SPIKE consumers bind bake Image (RPC-1-005); operator pixel prove = RPC-1-006
```

## Architect calls (frozen)

| # | Call | Verdict |
|:---|:---|:---|
| 1 | **Bake mechanism** | Prefer **dirty-region scissored bake** that samples material atlas into a world texture (dormant `terrain_instanced_draw` may *feed* bake indices — do not delete in B-only). Alt: compute bake following minimap compositor pattern — spike only if scissor bake fails pixel proof. |
| 2 | **Layer split** | Terrain texture = materials/topo only. Fire tint, sparks, weather = live composited overlays. |
| 3 | **Honest naming** | Rename path: `GpuInstancedAtlas` (lying) → transitional docs first → `GpuBake` / `CpuRaster`. Retire or gate `GpuTilemap`. Fix docs still promising `gpu_atlas`. |
| 4 | **Migration order** | Master green every slice. CPU raster remains display truth until bake proven by capture probes. Sequence **before** heavy `tile_world_fallback.rs` Phase-3 split so that file is not churned twice. |

## Slice ladder (RPC-1-0xx)

| id | Owner | Goal | Exit |
|:---|:---|:---|:---|
| **RPC-1-001** | planner | Freeze this doc + lint baseline | `terrain_honesty_lint` witness written; queue picks next |
| **RPC-1-002** | coder | **B0** — docs honesty (`gpu_atlas` → `world_raster` / transitional notes) | lint `doc_promises_gpu_atlas` = 0 |
| **RPC-1-003** | coder | **B1** — rename helpers/comments toward `GpuBake` without behavior change | `cargo test -p proc_A_dine01 --lib gpu_terrain` + stage5 subset green · **DONE 2026-08-12**: variants `CpuRaster`/`GpuBake` + const aliases `CpuFallback`/`GpuInstancedAtlas`; lint `dishonest_variant_name` cleared. Remaining allowlisted: `gputilemap_never_constructed` until `DR-MIG-TILEMAP` |
| **RPC-1-004** | coder | **A0** — bake spike (scissor atlas→world tex) behind flag | capture probe proves non-CPU path OR spike rejected with note · **DONE 2026-08-12**: `TERRAIN_GPU_BAKE_SPIKE` (default OFF) → `TerrainGpuBakeIndexMap` + dirty scissors + world bake Image target; fire excluded; CPU display still authority; witness `debug_runs/terrain_gpu_bake_spike_rpc1_004_live.json`. RTT write pass deferred to RPC-1-005. |
| **RPC-1-005** | coder | **A1** — wire minimap+tactical consumers to bake when flag on | `terrain_source` can honestly read `gpu_bake` · **DONE 2026-08-12**: host atlas→world scissor write; minimap binds bake when consumers ready; label `gpu_bake` only on that bind; default OFF stays `world_raster`; witness `debug_runs/terrain_gpu_bake_spike_rpc1_005_live.json`. Operator pixel proof = RPC-1-006. |
| **RPC-1-006** | designer/operator | Pixel prove + flip default | witness + captures; lint honest_gate |

## Deterministic L0 (no LLM)

```powershell
python -m rust_engine_mcp.cli terrain-honesty-lint
python -m rust_engine_mcp.cli agent-flow-route --goal "RPC-1 GPU terrain bake honesty" --domain render --force-hard
```

## Anti-goals

- Rebaking fire into terrain each tick  
- Deleting `terrain_instanced_draw` before bake decision proven  
- Claiming `gpu_atlas` in witnesses while label is `world_raster`  
- Parallel rewrite of `tile_world_fallback.rs` god-split during A0–A1
