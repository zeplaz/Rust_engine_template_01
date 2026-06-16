# ARCH-MAT-001 — Material authority = assembly snapshot (hard rule) `v1`

| Field | Value |
|:---|:---|
| **ID** | **ARCH-MAT-001** |
| **Status** | **ACTIVE** — non-negotiable architecture |
| **Supersedes** | “assign materials in Blender” for production |
| **Related** | [`arch_material_authority_001_v1.md`](arch_material_authority_001_v1.md) (process doc) |

---

## Rule

**Authoritative source:** `assembly_snapshot` / `assembly_graph` node fields.

```json
{
  "node_id": "wall_001",
  "module_id": "wall_steel_1u",
  "material_profile": "steel_panel_01",
  "semantic_tags": { "location": ["street_facing"] },
  "lod_policy": "production"
}
```

| System | Role |
|:---|:---|
| **APS** | **Authors** `material_profile`, tags, overrides |
| **Snapshot JSON** | **Ship contract** for build + render |
| **Blender worker** | **Consumes** snapshot — import GLB, apply profiles, render, export |
| **Blend file** | **Compiled artifact** — not material source of truth |

**Blender never authors materials** for normal production. Viewport paint is debug/repair only.

---

## Violations (reject in review)

- Ship gate: “open blend and assign PBR”
- Witness: `keyframe_manual` on headless slabs with no snapshot materials applied
- Promotion: atlas from bake that ignored `material_profile` on placements
- Docs that say “pause warehouse until materials” **instead of** “warehouse runs as spine test; production sign-off blocked until authority + honest validators”

---

## Enforcement

| Layer | Mechanism |
|:---|:---|
| APS | Material library → `update_placement` → Save snapshot |
| MCP | `assembly-build-run` reads snapshot only |
| Worker | BUILD-WORKER-001: apply `material_profile` from registry paths |
| Validators | APS-MAT-008: missing profile / missing maps = fail |
| Pilot | Track B proves spine with snapshot materials — not Blender UI |

---

## References

- [`plan_three_track_execution_v1.md`](plan_three_track_execution_v1.md)
- [`plan_material_studio_phase_v1.md`](plan_material_studio_phase_v1.md) (APS-MAT-*)
- [`arch_blender_worker_contract_v1.md`](arch_blender_worker_contract_v1.md)
