# PLAN-ATLAS-BUDGET-SIZING-001 — landscape atlas budget framework `v1`

```text
⟦SYMLANG⟧⟐v1  ◈PLAN
⟨ID⟩ PLAN-ATLAS-BUDGET-SIZING-001
Date: 2026-06-16
Status: **SIGNED** (@planner-mcp)
Owner: @planner-mcp → @designer-mcp (matrix sign) → @coder-mcp (batch)
Parent: APS-E4 · $ref:src/dev/design_landscape_lg5_atlas_v1.md
Example: $ref:assets/staging/specs/tile_batch_landscape_expanded_v1.example.json
```

**Goal:** Size **topology × state** matrix **before** bake — state-machine rule from tile-generation skill. Prevents 3-tile pilot re-bake on every new burn row.

---

## Budget formula

```text
slots = |topology_kinds| × (|base_states| + burn_frame_count + |regrowth_states|)
grid_cols = ceil(sqrt(slots)) aligned to atlas_meta_v2 UV grid
grid_rows = ceil(slots / grid_cols)
```

### Pilot expansion (v1 numbers)

| Input | Value |
|:---|:---|
| `topology_kinds` | Patch, Corridor, Ring → **3** |
| `base_states` | clean (+ optional damaged per topology) → **2** |
| `burn_frame_count` | **8** (`veg_burn_00..07`) |
| `regrowth_states` | nuclei, front → **2** |
| **Raw slots** | 3 × (2 + 8 + 2) = **36** |
| **Padded grid** | **3 × 14** (example job JSON) |

---

## State axes (landscape domain)

| Axis | Values | Maps to `variant_key` |
|:---|:---|:---|
| `state` | clean, damaged, burn, regrowth | catalog + tile_batch |
| `burn_frame` | 0..7 | `veg_burn_{:02}` |
| `regrowth_macro_phase` | Nuclei, Front | `veg_regrowth_*` |
| `topology_kind` | Patch, Corridor, Ring | `topology_*` |

Building axes (`power`, `fill`, `lighting`) remain **buildings domain only** — do not copy into landscape batch without `atlas_domain: landscape`.

---

## Gates

| Gate | Rule |
|:---|:---|
| **G0** | Designer signs expansion matrix (`DMCP-E4-MATRIX-CHARTER-001`) |
| **G1** | `atlas_budget` block present in job JSON before bpy |
| **G2** | `validate-report tile_batch` — variant_key ⊆ catalog |
| **G3** | PNG count == `matrix_rows × matrix_cols` (padded cells may be duplicate placeholder) |
| **G4** | `ship: true` only after artist re-verdict (APS-E0) |

---

## Rejected

- Growing pilot atlas ad-hoc without resizing grid
- Per-topology separate atlases before v2 charter (cost explosion)

---

## Sign-off

| Role | Date | Action |
|:---|:---|:---|
| **@planner-mcp** | 2026-06-16 | **SIGNED** |

```text
⟦/PLAN-ATLAS-BUDGET-SIZING-001⟧  ΔWF→ DMCP-E4-MATRIX-CHARTER-001
```
