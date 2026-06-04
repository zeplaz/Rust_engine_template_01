# MCP fleet — TILE-REAL-001 + TILE-ENGINE-001 orders `v1`

| Field | Value |
|:---|:---|
| **Program ID** | **MCP-TILE-CLOSE-001** |
| **Owner** | `@orchestrator-mcp` |
| **Date** | 2026-06-02 |
| **Status** | **ACTIVE** |
| **Prerequisite** | **ART-APS-USE PASS** — `tile_batch_rowhouse_victorian_pilot_v1.json` + `aps_pilot_rowhouse_g0_rules.yaml` |
| **Planner** | **NOT REQUIRED** — architecture in [`plan_tile_pipeline_automation_exec_v1.md`](plan_tile_pipeline_automation_exec_v1.md) + this doc |

**Program green blockers:** only these two gates remain (PG-2-WIT already pass).

---

## Gate summary

| Gate | Owner | Pass when |
|:---|:---|:---|
| **TILE-REAL-001** | @coder-mcp | Real bake `dry_run: false` in batch_status; PNGs >1×1; atlas + meta; witness G3; `_tile_atlas_index.ron` row |
| **TILE-ENGINE-001** | @coder | `TileAtlasRegistry` loads index; test resolves pilot atlas + variant UV |

---

## TILE-REAL-001 — @coder-mcp

### Input (already on disk)

| File | Path |
|:---|:---|
| Tile batch | `tools/mcp/schemas/examples/tile_batch_rowhouse_victorian_pilot_v1.json` |
| Assembly snapshot | `tools/mcp/schemas/examples/assembly_snapshot_rowhouse_victorian_v1.json` |
| Variant set | `tools/mcp/schemas/examples/variant_set_rowhouse_victorian_v1.json` |
| G0 rules | `debug_runs/art_pipeline/aps_pilot_rowhouse_g0_rules.yaml` |

### Execute (PowerShell)

```powershell
cd C:\dev\github\Rust_engine_template_01\tools\mcp\python

# 1. Validate spec (validation-first)
python -m rust_engine_mcp.cli validate-report tile_batch ..\schemas\examples\tile_batch_rowhouse_victorian_pilot_v1.json --compress 3

# 2. REAL bake — dry run MUST be off
Remove-Item Env:RUST_ENGINE_TILE_DRY_RUN -ErrorAction SilentlyContinue
$env:RUST_ENGINE_TILE_LIGHT_BLEND = "C:\dev\github\Rust_engine_template_01\utils\Light_keysshotsetup.blend"

python -m rust_engine_mcp.cli tile-batch-run ..\schemas\examples\tile_batch_rowhouse_victorian_pilot_v1.json

# 3. Witness
python -m rust_engine_mcp.cli write-witness tile_rowhouse_victorian_pilot_v1
```

### Implement if missing (same PR as bake)

| ID | File | What |
|:---|:---|:---|
| R1 | `rust_engine_mcp/tile_index.py` | `register_tile_atlas_from_meta(meta_json_path)` → upsert `assets/configs/buildings/_tile_atlas_index.ron` |
| R2 | `tile_pipeline.py` | Call register after successful real bake (`dry_run` false) |
| R3 | `cli.py` + `server.py` | `tile-atlas-register <batch_id>` MCP/CLI parity |
| R4 | `tests/test_tile_pipeline.py` | Assert witness G3 fails on 1×1 stub when dry_run false expected |

**Index row fields (minimum):**

```ron
(
    atlas_id: "rowhouse_victorian_pilot_v1",
    batch_id: "tile_rowhouse_victorian_pilot_v1",
    assembly_id: "victorian_4x3_s42_a7cb",
    tile_id: "rowhouse_victorian",
    atlas_png: "assets/textures/tiles/rowhouse_victorian_pilot_v1_atlas.png",
    meta_json: "assets/staging/tiles/tile_rowhouse_victorian_pilot_v1/atlas_meta.json",
    development_tier: "lod0",
    style_pack_id: "style_victorian",
)
```

Variant UVs stay in `atlas_meta.json` — index points at meta, not duplicated.

### Acceptance checklist

- [ ] `batch_status.json` has `"dry_run": false`
- [ ] Variant PNGs exist and are **not** 1×1 stub (check file size or dimensions)
- [ ] `assets/textures/tiles/rowhouse_victorian_pilot_v1_atlas.png` exists
- [ ] `atlas_meta.json` has 2 tiles with UV rects
- [ ] `debug_runs/art_pipeline/tile_rowhouse_victorian_pilot_v1_live.json` → `gates.G3: pass`
- [ ] `_tile_atlas_index.ron` contains pilot entry
- [ ] Update `debug_runs/art_pipeline/mcp_art_program_green_live.json` → `TILE-REAL-001: pass`
- [ ] `pytest tests/ -q` green (37+)

### Blockers

| Issue | Action |
|:---|:---|
| Blender not found | `locate-blender` / set `BLENDER_EXE` in MCP env |
| `assembly_build` fails | validation-first on assembly job status JSON |
| Still dry_run true | env var still set — clear before run |

---

## TILE-ENGINE-001 — @coder

**Start after TILE-REAL-001 witness G3 pass.** No planner — mirror `ProceduralModuleRegistry` pattern.

### Read

- [`src/construction/procedural/module_index.rs`](../../src/construction/procedural/module_index.rs) — loader pattern
- [`assets/staging/tiles/tile_rowhouse_victorian_pilot_v1/atlas_meta.json`](../../assets/staging/tiles/tile_rowhouse_victorian_pilot_v1/atlas_meta.json) — after real bake
- [`plan_tile_batch_v1_planner_mcp_v1.md`](plan_tile_batch_v1_planner_mcp_v1.md) § G5 — separate index, not `_module_index.ron`

### Implement (≤6 files, can split 2 PRs)

| File | Content |
|:---|:---|
| `src/construction/procedural/tile_atlas_index.rs` | **new** — `TileAtlasEntry`, `TileAtlasRegistry`, load RON/JSON |
| `src/construction/procedural/mod.rs` | export + `init_tile_atlas_registry` |
| `src/construction/procedural/tests.rs` or inline | `tile_atlas_index_loads_pilot` |
| Optional: terrain/map hook | resolve `variant_key` → `Handle<Image>` from atlas_png + UV — **minimal**: registry load only is enough for gate |

**Constants:**

```rust
pub const TILE_ATLAS_INDEX_RON: &str = "assets/configs/buildings/_tile_atlas_index.ron";
```

**API (minimum):**

```rust
impl TileAtlasRegistry {
    pub fn get(&self, atlas_id: &str) -> Option<&TileAtlasEntry>;
    pub fn resolve_variant_uv(&self, atlas_id: &str, variant_key: &str) -> Option<[f32; 4]>;
    // reads meta_json lazily or at load time
}
```

**Do not:** mix tile rows into `_module_index.ron`. **Do not** rework PG-2 (`procedural_assembly_live.json` already green).

### Tests (required names)

- `tile_atlas_index_loads_pilot_rowhouse`
- `tile_atlas_resolve_variant_uv_clean_day`
- `tile_atlas_registry_empty_when_index_missing` (graceful)

### Witness

Extend `debug_runs/art_pipeline/mcp_art_program_green_live.json`:

```json
"TILE-ENGINE-001": { "status": "pass", "atlas_id": "rowhouse_victorian_pilot_v1", "variants_resolved": 2 }
```

Set top-level `"green": true` when both TILE gates pass.

### Verification

```powershell
cargo test -p proc_A_dine01 --lib procedural::tile_atlas
cargo test -p proc_A_dine01 --lib procedural
```

---

## Paste — @coder-mcp

> **TILE-REAL-001** from `src/dev/mcp_fleet_tile_real_engine_orders_v1.md`.
>
> Pilot batch: `tools/mcp/schemas/examples/tile_batch_rowhouse_victorian_pilot_v1.json`.
> Clear `RUST_ENGINE_TILE_DRY_RUN`. Run real `tile-batch-run`. validation-first only.
> If `_tile_atlas_index.ron` writer missing, add `tile_index.py` + register after bake.
> Witness G3 must pass with `dry_run: false`. Update `mcp_art_program_green_live.json`.
> pytest green. Report PNG paths + atlas size + gate checklist.

---

## Paste — @coder

> **TILE-ENGINE-001** from `src/dev/mcp_fleet_tile_real_engine_orders_v1.md` — **after** TILE-REAL-001 G3.
>
> Add `TileAtlasRegistry` loading `assets/configs/buildings/_tile_atlas_index.ron`. Mirror `module_index.rs`. Resolve variant UV from linked `meta_json`. Tests: `tile_atlas_index_loads_pilot_rowhouse`. ≤6 files. PG-2 done — do not touch. Set program green witness when pass.

---

## Paste — @planner

> **No dispatch.** TILE-REAL/ENGINE architecture is in `mcp_fleet_tile_real_engine_orders_v1.md`. Separate `_tile_atlas_index.ron` already decided in tile batch plan G5.

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-06-02 | Focused close-out for program green |
