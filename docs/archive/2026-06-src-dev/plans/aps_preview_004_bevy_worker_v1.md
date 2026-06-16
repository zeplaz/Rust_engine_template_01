# APS-PREVIEW-004 — Bevy preview worker architecture `v1` (doc only)

| Field | Value |
|:---|:---|
| **ID** | APS-PREVIEW-004 |
| **Status** | **done** (implemented 2026-06-03) |
| **Parent** | [`plan_building_grammar_evolution_v1.md`](plan_building_grammar_evolution_v1.md) |
| **Implements** | APS-PREVIEW-002 |
| **Binary** | `cargo build --bin bevy_preview_worker` |
| **Witness** | [`debug_runs/preview_worker_smoke_live.json`](../../debug_runs/preview_worker_smoke_live.json) |

---

## Goal

Art Pipeline Suite requests a **thumbnail of an assembly snapshot** using the **same Bevy renderer** as the game — not a second Blender or custom pyglet path for assembly preview.

---

## Components

```text
┌─────────────────────┐     JSON-RPC / file IPC      ┌──────────────────────────┐
│  APS (tkinter)      │ ───────────────────────────► │  bevy_preview_worker     │
│  assembly_panel.py  │ ◄─────────────────────────── │  (cargo bin, headless)   │
└─────────────────────┘     PNG path + metadata      └──────────────────────────┘
```

| Piece | Responsibility |
|:---|:---|
| **APS client** | `preview_assembly(snapshot_path)` → spawn worker, wait, show thumb |
| **Worker binary** | `bevy_preview_worker` (new `src/bin/` or feature-gated harness) |
| **Contract** | Job JSON in, witness + PNG out |

---

## Job contract (v1)

**Input** (`debug_runs/preview_jobs/<job_id>.json`):

```json
{
  "schema_version": 1,
  "operation": "preview_assembly",
  "assembly_snapshot": "assets/staging/assemblies/industrial_west_4x2_s43_a879.json",
  "camera": { "preset": "iso_ne", "distance_m": 24.0 },
  "output": {
    "png": "debug_runs/preview_jobs/industrial_west_4x2_thumb.png",
    "width": 512,
    "height": 512
  }
}
```

**Output** (`<job_id>.status.json`):

```json
{
  "status": "done",
  "png": "debug_runs/preview_jobs/industrial_west_4x2_thumb.png",
  "elapsed_ms": 1200,
  "modules_loaded": 24
}
```

---

## Worker pipeline (inside Bevy)

1. Load `assembly_snapshot` JSON (Rust `procedural::assembly_snapshot` types).
2. Resolve GLBs from `module_placements` (reuse module_index paths).
3. Spawn meshes at `position` / `rotation_euler` (same convention as MCP import).
4. Apply **material_profile** via existing PBR asset paths (or greybox fallback).
5. Iso camera from preset (align with `Tile_iso_rig_v1` framing).
6. Render one frame to PNG (existing offscreen / harness path from Stage 5 if available).
7. Exit 0; no window.

**Out of scope v1:** variant layers, fire animation, multiview.

---

## APS integration (future APS-PREVIEW-002)

| Tab | Preview source |
|:---|:---|
| Catalog | Keep model-viewer HTTP (single GLB) |
| **Assembly** | **Bevy worker** (multi-module snapshot) |
| Variants | Bevy worker + variant_set overlay **or** last bake PNG |
| Atlas | 2D image + UV grid (no Bevy) |

Env: **`RUST_ENGINE_BEVY_PREVIEW=0`** disables Bevy spawn (browser-only). Default: try `target/debug/bevy_preview_worker` then `cargo run --bin bevy_preview_worker`.

---

## CLI parity (MCP / agents)

```powershell
cargo run --bin bevy_preview_worker -- preview-assembly path/to/snapshot.json --out debug_runs/preview_jobs/out.png
```

Same job JSON as APS — tri-mode rule.

---

## Failure modes

| Case | Behavior |
|:---|:---|
| Missing GLB | `status: failed`, list `missing_glb[]` |
| Worker timeout | APS shows last good thumb + error |
| Headless GPU unavailable | Fall back to catalog browser multi-GLB (degraded) |

---

## Dependencies

| Needs | From |
|:---|:---|
| Snapshot + grammar fields | ARCH-ASSEMBLY-GRAPH-002, CODER-SNAPSHOT-GRAMMAR-WIRE |
| Module paths | `_module_index.ron` |
| Harness / offscreen render | Stage 5 test harness patterns |

---

## Implementation order (when coding)

1. `bevy_preview_worker` bin + job schema  
2. Load placements + spawn (reuse `procedural_build_spawn` extraction path)  
3. APS “Preview assembly” button  
4. Witness `debug_runs/preview_worker_smoke_live.json`

**Implementation:** `src/preview/assembly_worker.rs`, `src/bin/bevy_preview_worker.rs`, wired from `rust_engine_mcp/assembly_preview.py`.
