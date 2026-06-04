# TILE-FIX-10 — Promotion gates `v1` (minimum G4)

| Field | Value |
|:---|:---|
| **Status** | **DONE** (validator plumbing) · **ship art** requires manual `keyframe_render` — see [`mcp_orchestrator_tile_fix_warehouse_slice_v2.md`](mcp_orchestrator_tile_fix_warehouse_slice_v2.md) |
| **Validator** | [`tools/mcp/python/rust_engine_mcp/validators/tile_promotion.py`](../../tools/mcp/python/rust_engine_mcp/validators/tile_promotion.py) |
| **Pipeline** | [`tile_compile_loop.py`](../../tools/mcp/python/rust_engine_mcp/tile_compile_loop.py) — `run_minimum_compile_pipeline`, `write_tile_fix_10_witness` |
| **Witness** | `debug_runs/art_pipeline/tile_fix_10_{building_id}_live.json` |

---

## Gate checklist (`validate_tile_promotion`)

| Gate | Rule |
|:---|:---|
| **Shell GLBs** | `wall_steel_1u` / `roof_sawtooth` (etc.) must be **production** `job_id`s — blocks warehouse until promote |
| **Assembly** | Production-tier snapshot + module GLBs on disk |
| **Materials** | `albedo.png` + `normal.png` + `roughness.png` per profile (no greybox fallback for ship) |
| **Bake** | **24** minimum cell PNGs in staging (`MINIMUM_G4_CELLS`) |
| **Atlas** | `atlas_meta` **v2** with **24** complete lookups (`minimum_g4_ship` / `lookup_mode: minimum_g4`) |

**Not in minimum G4 (deferred):** full **576** cell matrix (8 facings × all states × fire frames) — post shell-GLB promote.

---

## Minimum G4 matrix

- **24 cells** = `ship_minimum_states` × `minimum_g4_facings` (default **3 states × 8 facings**, frame 0).
- Schemas: `minimum_g4_facings`, `minimum_g4_cells` on `visual_config_v1`; `minimum_g4_ship` on `atlas_meta_v2`.

---

## Schema green ≠ ship art (2026-06-03)

`validate_tile_promotion` **passed** on headless v2 minimum bake proves **JSON + 24 lookups** — not factory iso quality. Witness must set `green: false` and `art_quality: rejected_headless_procedural` until manual keyframe stills pass designer G4.

**Do not** `--register` from `tile_compile_minimum_bake.py` for ship.

---

## Warehouse blocker (expected)

[`building_definition_warehouse_industrial_west_production_v1.json`](../../tools/mcp/schemas/examples/building_definition_warehouse_industrial_west_production_v1.json) still references **lod0** wall/roof jobs.

`run_minimum_compile_pipeline` exits **`tile_fix_10_blocked`** with `shell_blockers` and writes compile plan only until:

1. Promote `wall_steel_1u` / `roof_sawtooth` (or equivalent) to production GLBs.
2. Update building_definition module `job_id`s + assembly snapshot paths.
3. Re-run bake → pack → witness.

---

## Fixes bundled with TILE-FIX-10

| Fix | Detail |
|:---|:---|
| **Facing 0** | `atlas_meta` lookup parse uses `facing is not None` (not `facing or -1`) so facing **0** is valid |
| **PNG-exists** | Superseded by promotion validator + v2 lookup completeness |

---

## Tests

```powershell
cd tools/mcp/python
python -m pytest tests/test_tile_fix_pipeline.py tests/test_tile_promotion_gates.py tests/test_tile_atlas_v2_schema.py -q
```

**18 passed** (minimum G4 path + promotion gates + v2 schema).
