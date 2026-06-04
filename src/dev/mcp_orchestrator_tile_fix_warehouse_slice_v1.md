# Orchestrator slice — TILE-FIX warehouse minimum G4 `v1`

| Field | Value |
|:---|:---|
| **Slice ID** | **TILE-FIX-WAREHOUSE-MIN-G4-001** |
| **Program** | **PLAN-TILE-FIX-AUTO-BUILD-001** |
| **Owner** | `@orchestrator-mcp` |
| **Date** | 2026-06-03 |
| **First shippable building** | **warehouse** (`style_industrial_west`) |
| **Exit witness** | `debug_runs/art_pipeline/tile_fix_10_warehouse_industrial_live.json` |
| **Rules** | [`tile_fix_10_promotion_gates_v1.md`](tile_fix_10_promotion_gates_v1.md) · [`tile_greybox_production_frozen_v1.md`](tile_greybox_production_frozen_v1.md) |

**Orchestrator:** sequence only — **no** Rust/Python edits in this lane.

---

## Context (confirmed)

| Fact | Evidence |
|:---|:---|
| TILE-FIX-01..10 plumbing | **DONE** — `tile_compile_minimum_bake.py`, promotion validator, 18 pytest |
| Active `_tile_atlas_index.ron` | **EMPTY** — v2 row only after green + `--register` |
| Greybox production v1 | **FROZEN** — do not re-promote |
| Blocker | `building_definition_warehouse_industrial_west_production_v1.json` — `wall_steel_1u` / `roof_sawtooth` still **lod0** `job_id`s |
| TILE-FIX-09 | **green** — matrix + bdef contract (`tile_fix_09_warehouse_live.json`) |
| TILE-FIX-10 | **not written** until production shell GLBs + minimum bake |

---

## Phase flow

```text
@orchestrator-mcp     (this doc — no code)
        ↓
@planner-mcp          shell job_ids + paths (readonly, short)
        ↓
@coder-mcp            geometry_run wall_steel + roof_sawtooth PRODUCTION
                      → bdef + assembly_snapshot JSON
                      → material PNG gate (steel_panel_01, roof_metal_01)
        ↓
@coder-mcp            tile_compile_minimum_bake (24 cells → pack v2 → witness)
        ↓
@designer-mcp         G4 on real 128px stills (≥3 states × 8 facings)
        ↓
@coder                --register + map stamp smoke (if witness green)
```

---

## Phase A — @planner-mcp (readonly)

**Deliverable:** one-page confirm list (no implementation).

| Item | Repo convention |
|:---|:---|
| Wall job | `wall_steel_1u_production_run001` |
| Roof job | `roof_sawtooth_production_run001` |
| Corner (already prod) | `corner_L_production_run001` |
| Door (optional shell) | keep `door_shop_lod0_run001` until door production exists, or document waiver |
| GLB paths | `assets/models/modules/{job_id}/model.glb` |

**Files to touch (Phase B owner = coder-mcp):**

| File | Role |
|:---|:---|
| `tools/mcp/schemas/examples/building_definition_warehouse_industrial_west_production_v1.json` | `modules[].job_id` for wall/roof |
| `tools/mcp/schemas/examples/assembly_snapshot_warehouse_industrial_west_production_v1.json` | `job_id` + `glb_path` per placement |
| `assets/configs/buildings/visual_config_warehouse_industrial_west_v2.json` | facing matrix / minimum_g4 (confirm aligned with `warehouse_state_facing_matrix_v1.yaml`) |

**Matrix:** `debug_runs/art_pipeline/warehouse_state_facing_matrix_v1.yaml`

---

## Phase B — @coder-mcp

| Step | Action |
|:---|:---|
| B1 | `geometry_run_job` for `wall_steel_1u` + `roof_sawtooth` — `development_tier: production`, profiles `module_wall` / `module_roof` (sawtooth), materials `steel_panel_01` / `roof_metal_01` |
| B2 | **FAIL** promote/bake if material PNGs missing (`albedo.png`, `normal.png`, `roughness.png` per profile) |
| B3 | Update bdef + assembly snapshot `job_id` + `glb_path` to production tier |
| B4 | Minimum compile pipeline |

**Commands (PowerShell):**

```powershell
cd C:\dev\github\Rust_engine_template_01

# Blocked check / plan only (before shell GLBs)
python tools/mcp/scripts/tile_compile_minimum_bake.py --plan-only

# Full minimum path (after B1–B3 green)
python tools/mcp/scripts/tile_compile_minimum_bake.py `
  --building tools/mcp/schemas/examples/building_definition_warehouse_industrial_west_production_v1.json

# Register ONLY if witness green
python tools/mcp/scripts/tile_compile_minimum_bake.py `
  --building tools/mcp/schemas/examples/building_definition_warehouse_industrial_west_production_v1.json `
  --register
```

**Regression:**

```powershell
cd tools/mcp/python
python -m pytest tests/test_tile_fix_pipeline.py tests/test_tile_promotion_gates.py tests/test_tile_atlas_v2_schema.py -q
```

**validation-first:** use `validate-report` / promotion JSON — not raw bake logs.

---

## Phase C — @designer-mcp

| Step | Action |
|:---|:---|
| C1 | G4 rubric on **real** 128×128 minimum stills — **3 states × 8 facings** (24 cells) |
| C2 | Update `warehouse_industrial_west_production_signoff.yaml` or `tile_fix_09_warehouse_g4_signoff.yaml` → `proceed_ship: yes` **from witness**, not PNG-exists |
| C3 | Read `tile_fix_10_warehouse_industrial_live.json` — confirm `minimum_g4_ship` / `lookup_mode: minimum_g4` |

**Do NOT:** re-promote greybox v1 atlases · `mcp_export_pilot_keyframes_g4` as ship path · mark green on PNG-exists alone.

---

## Phase D — @coder (optional tail)

| Step | When |
|:---|:---|
| D1 | `_tile_atlas_index.ron` one v2 row — only if `--register` ran and witness green |
| D2 | `map_tile_atlas_stamp` smoke with `rotation_quarter_turns` on a test site — FULL_APP check |

**Blocked until:** Phase C designer G4 + `tile_fix_10_*` green.

---

## Do NOT

- Re-promote greybox production v1 (`tile_greybox_production_frozen_v1.md`)
- Use `@coder` for Blender bpy — **@coder-mcp** only (`tools/mcp/`)
- Ship on lod0 wall/roof job_ids
- Full **576** cell matrix in this slice (post shell-GLB promote #3)

---

## After minimum G4 green

| Track | Owner | What |
|:---|:---|:---|
| Next building | @orchestrator → same pipeline | bunker / shopfront / rowhouse |
| Full matrix 576 | @planner-mcp + @coder-mcp | after “shell GLB promote #3” |
| FULL_APP map proof | @coder | stamp + rotation on site |

---

## Paste — @planner-mcp

```
Slice TILE-FIX-WAREHOUSE-MIN-G4-001 — Phase A only (readonly).
Read src/dev/mcp_orchestrator_tile_fix_warehouse_slice_v1.md + tile_fix_10_promotion_gates_v1.md.

Confirm shell job_ids: wall_steel_1u_production_run001, roof_sawtooth_production_run001.
List files Phase B must edit: building_definition_warehouse_industrial_west_production_v1.json,
assembly_snapshot_warehouse_industrial_west_production_v1.json,
visual_config_warehouse_industrial_west_v2.json.
No code. Short bullet output only.
```

---

## Paste — @coder-mcp (Phase B — say “run Phase B”)

```
Slice TILE-FIX-WAREHOUSE-MIN-G4-001 — Phase B from src/dev/mcp_orchestrator_tile_fix_warehouse_slice_v1.md.

1) geometry_run_job: wall_steel_1u + roof_sawtooth — production tier, PBR profiles.
   FAIL if steel_panel_01 / roof_metal_01 PNGs missing on disk.
2) Update building_definition + assembly_snapshot job_id/glb_path to *_production_run001.
3) python tools/mcp/scripts/tile_compile_minimum_bake.py --building tools/mcp/schemas/examples/building_definition_warehouse_industrial_west_production_v1.json
4) --register ONLY if tile_fix_10_warehouse_industrial_live.json green (validate_tile_promotion).
validation-first. pytest test_tile_fix_pipeline test_tile_promotion_gates test_tile_atlas_v2_schema -q.
Do NOT use greybox v1 or lod0 wall/roof for ship.
```

---

## Paste — @designer-mcp (Phase C)

```
Slice TILE-FIX-WAREHOUSE-MIN-G4-001 — Phase C from src/dev/mcp_orchestrator_tile_fix_warehouse_slice_v1.md.

G4 on real 128px minimum stills (24 cells = 3 states × 8 facings).
Witness: debug_runs/art_pipeline/tile_fix_10_warehouse_industrial_live.json must be green first.
Sign-off: proceed_ship yes from validate_tile_promotion / minimum_g4_ship — NOT greybox v1, NOT PNG-exists-only.
Update warehouse production signoff yaml accordingly. No Rust.
```

---

## Paste — @coder (Phase D)

```
Slice TILE-FIX-WAREHOUSE-MIN-G4-001 — Phase D after tile_fix_10_warehouse_industrial_live.json green.
Register path already run by coder-mcp --register if witness green.
Smoke: map_tile_atlas_stamp with rotation_quarter_turns on one warehouse site. Witness note in procedural_tiles_runtime or map stamp live json.
≤6 files. Do not rework PG-2.
```
