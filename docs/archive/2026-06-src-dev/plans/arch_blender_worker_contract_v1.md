# ARCH-BLENDER-001 — Blender worker roles (not primary authoring) `v1`

| Field | Value |
|:---|:---|
| **ID** | ARCH-BLENDER-001 |
| **Source** | [`docs/archive/2026-06-fleet-drain/prompts_drafts/planner_fix_auto_build.md`](../../docs/archive/2026-06-fleet-drain/prompts_drafts/planner_fix_auto_build.md) L2077–L2097 |
| **Parent** | [`plan_material_studio_phase_v1.md`](plan_material_studio_phase_v1.md) |
| **Status** | **ACTIVE** (contract only) |
| **Date** | 2026-06-03 |

---

## Roles

| Worker role | Operations | APS trigger |
|:---|:---|:---|
| **Render** | `keyframe_render`, iso still export | Pilot G4, promotion proofs |
| **Bake** | tile ortho / atlas pack | Variants → Atlas |
| **Convert** | assembly build, GLB import, repair | MCP `build-assembly` |
| **Material apply** | Map `material_profile` → shader/images from registry | BUILD-WORKER-001 (reads snapshot only) |

## Forbidden as primary path

- Open blend to **assign** materials by hand
- Viewport sanity as ship gate for profiles
- Artist daily workflow inside Blender UI

Authoring: **APS Material Studio + Assembly** ([`arch_material_authority_001_v1.md`](arch_material_authority_001_v1.md)).
