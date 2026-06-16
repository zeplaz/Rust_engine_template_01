# ARCH-PBG-MASSING-001 — PBG massing placement model (perimeter grid vs mesh-face) `v1`

| Field | Value |
|:---|:---|
| **ID** | **ARCH-PBG-MASSING-001** |
| **Owner** | @planner-mcp |
| **Status** | **ACTIVE** (architecture only — **no generator change** until this gate passes) |
| **Source** | [`docs/archive/2026-06-fleet-drain/prompts_drafts/planner_fix_auto_build.md`](../../docs/archive/2026-06-fleet-drain/prompts_drafts/planner_fix_auto_build.md) §824+ (hierarchy) · Republic-style massing inspiration |
| **Date** | 2026-06-03 |
| **Related** | [`arch_build_grammar_001_schema_v1.md`](arch_build_grammar_001_schema_v1.md) · [`pg_module_audit_warehouse_v1.md`](pg_module_audit_warehouse_v1.md) · [`pilot_grammar_001_g4_checklist_v1.md`](pilot_grammar_001_g4_checklist_v1.md) |

---

## Bottom line

**PBG** = procedural building grammar path: `generate(archetype, district_style, seed)` → massing → placements.

Today every ship path uses **perimeter grid** (W/D/C/R on a rect footprint). **Mesh-face instancing** is the Republic-style upgrade for non-rect silhouettes (L, courtyard, bar, sawtooth crown) — **design this before** touching `footprint_grid.rs`, `building_grammar.rs`, or `assembly.py` fill logic.

**Module list / kit gaps** stay in audit + pilot checklist — this doc does **not** duplicate PG-MODULE-AUDIT-001.

---

## Two placement backends

| Model | What it is | Best for |
|:---|:---|:---|
| **A — Perimeter grid** (current) | 2D plan grid; facade = outer ring; tokens `W`/`D`/`C`/`R`; interior = `Yard` (no mesh) | Rect halls, warehouse pilot, iso tile spine, deterministic MCP snapshots |
| **B — Mesh-face instancing** (future) | Massing = coarse mesh/volume; each **face** (or face run) gets module runs + height bands; corners = edge topology not grid corners | L-bar, courtyards with partial wings, sawtooth roof rhythm, stacked mezzanines, non-axis-aligned lots |

```text
A (now):                    B (if scope grows):

  W W D W W                   ┌──face N──┐
  W . . . W      vs           │ modules  │
  W . Y . W                   └──face S──┘
  C . . . C                     + roof faces
```

---

## Current implementation (frozen contract)

| Layer | Behavior |
|:---|:---|
| **Grammar** | `massing_strategy` + `footprint_mode`: `rect` \| `l_shape` \| `yard_interior` ([`industrial_warehouse_v1.ron`](../../assets/configs/buildings/grammars/industrial_warehouse_v1.ron)) |
| **Rust** | [`FootprintGrid::from_grammar`](../../src/construction/procedural/footprint_grid.rs) → `from_rect` + yard/L notch |
| **Python** | [`footprint_grid_from_grammar`](../../tools/mcp/python/rust_engine_mcp/building_grammar.py) mirrors Rust |
| **Fill** | [`assembly.generate_assembly_snapshot`](../../tools/mcp/python/rust_engine_mcp/assembly.py) — one module per W/D/C/R cell; `Y` skipped |
| **Verify** | [`assembly_grammar_verify`](../../tools/mcp/python/rust_engine_mcp/validators/assembly_grammar_verify.py) — perimeter placement counts |

**v1 L-shape / yard_complex** are still **grid hacks** (interior `Y` cells), not true mesh-face topology. Acceptable for pilot; not the long-term Republic model.

---

## Mesh-face instancing (target architecture)

### Concepts

| Term | Meaning |
|:---|:---|
| **Massing mesh** | Low-poly shell: extruded footprint + optional roof step / mezzanine step (data-only or glTF greybox) |
| **Face** | Oriented quad (or n-gon split to quads) with outward normal, length in grid units, height span in floors |
| **Face run** | Contiguous modules along one face (e.g. 6× `wall_1u` on north facade) |
| **Instancing** | Same `module_id` repeated with transforms along face UV / arc-length — snapshot still lists discrete placements for MCP/Blender |

### Pipeline (when B is adopted)

```text
GrammarGenerateResult
  → MassingShell (mesh or parametric face list)
  → FaceRunPlanner (slot_key, count, door/window breaks)
  → module_placements[]  (same assembly snapshot schema)
  → APS / headless worker (unchanged downstream)
```

**Authority unchanged:** assembly snapshot remains source of truth; only the **planner** that emits placements changes.

### Schema hook (only if scope grows)

Extend `building_grammar_v1` massing strategy (optional field — do not implement until approved):

```ron
(
    id: "sawtooth_bar",
    weight: 10,
    placement_mode: "mesh_face",  # default "perimeter_grid"
    shell: "industrial_bar_v1",   # references massing_shell_v1 RON/JSON
)
```

New artifact type (planner-only draft): `massing_shell_v1` — faces with `normal`, `length_u`, `floor_min`, `floor_max`, `opening_policy`.

---

## Decision gate

| Stay on **A** (perimeter grid) | Invest in **B** (mesh-face) |
|:---|:---|
| IndustrialWarehouse pilot + G4 tile spine | Need **silhouette** not representable as rect + yard notch |
| PG-MODULE-AUDIT-002 still filling slots | Detail grammar needs **per-face** density (vents/stacks along one elevation) |
| APS footprint heatmap is enough UX | APS preview must show **non-rect** massing accurately |
| Witness: perimeter count verify passes | Product asks for **Double Hall / true L / bar** with interior wings |

**Rule:** No `CODER-*` massing refactor until product picks A-only extension vs B pilot archetype.

---

## Recommended order (planner)

| Step | ID | Note |
|:---|:---|:---|
| 1 | **ARCH-PBG-MASSING-001** | This doc — **done** when merged |
| 2 | PG-MATERIAL-GENERATION-001 + APS preview | Material authority (L1315+) — independent of A/B |
| 3 | PILOT-GRAMMAR-001 on **A** | Warehouse 4×2 hall on perimeter grid |
| 4 | **ARCH-PBG-MASSING-002** (if gate opens) | `massing_shell_v1` schema + one greybox shell (`industrial_bar_v1`) |
| 5 | **CODER-PBG-MASSING-003** | Rust `FaceRunPlanner` + snapshot wire; Python parity |
| 6 | **CODER-PBG-MASSING-004** | Retire grid L-notch; migrate `l_shape` / `yard_complex` to shell or keep dual path behind `placement_mode` |

---

## Explicit non-goals (this slice)

- Changing [`building_grammar.rs`](../../src/construction/procedural/building_grammar.rs) massing strategies
- New module SKUs (see [`pg_module_audit_warehouse_v1.md`](pg_module_audit_warehouse_v1.md))
- Blender manual material / keyframe (see [`arch_material_authority_001_v1.md`](arch_material_authority_001_v1.md))
- Bevy render meshing of massing shell (preview doc only: [`aps_preview_004_bevy_worker_v1.md`](aps_preview_004_bevy_worker_v1.md))

---

## Agent routing

| Question | Route |
|:---|:---|
| “Which wall module for warehouse door slot?” | @designer-mcp + audit — **not** this doc |
| “How do we place modules on an L-wing?” | This doc → if approved, @planner-mcp **ARCH-PBG-MASSING-002** then @coder |
| “Grammar rule chain / tags / materials” | [`plan_building_grammar_evolution_v1.md`](plan_building_grammar_evolution_v1.md) |

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-06-03 | Thin slice: A vs B, gate, no generator diff |
