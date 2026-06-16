# PLAN-KIT-LOD0-ROADMAP-001 — 50-module lod0 wave roadmap `v1`

| Field | Value |
|:---|:---|
| **Program ID** | **PLAN-KIT-LOD0-ROADMAP-001** |
| **Parent** | [`plan_module_kit_production_tier_v1.md`](plan_module_kit_production_tier_v1.md) |
| **Fleet order** | **MCP-PLN-002** (`mcp_fleet_active_orders_v1.md`) |
| **Owner** | `@planner-mcp` |
| **Date** | 2026-06-02 |
| **Status** | **SIGNED** |

---

## Summary

Canonical kit has **50** module IDs ([`design_procedural_module_kit_v1.md`](design_procedural_module_kit_v1.md) §191–211). **10** are in **lod0_001–002** (in flight). **40** remain — roadmap waves **`kit_lod0_003` → `kit_lod0_010`** (8 × 5) before any `kit_production_*` wave.

**`kit_lod0_003`** module pick is **fixed below** — no blockers for `@designer-mcp` MCP-D0-003.

**`kit_production_*`:** **DEFER** until Material Maker / PBR lane documented (per production tier plan § resolved OQ #3).

---

## Coverage ledger (50 canonical)

| Category | Total | lod0_001 | lod0_002 | lod0_003 | Remaining |
|:---|---:|:---|:---|:---|---:|
| Walls | 10 | 2 | 2 | 1 | **5** |
| Roofs | 10 | 1 | 1 | 1 | **7** |
| Doors | 10 | 1 | 1 | 1 | **7** |
| Windows | 10 | 1 | 1 | 1 | **7** |
| Corner / prop | 10 | 0 | 0 | 1 | **9** |
| **Σ** | **50** | **5** | **5** | **5** | **35** |

---

## `kit_lod0_001` — SHIPPED / in G5 (reference)

| module_id | archetype | bpy `profile` / params | pbr_status |
|:---|:---|:---|:---|
| `wall_brick_1u` | module_wall | flat | shipped |
| `wall_concrete_1u` | module_wall | flat | shipped |
| `roof_pitched_gable` | module_roof | pitched_gable | deferred |
| `door_residential` | module_door | frame (residential dims) | shipped |
| `win_single_1u` | module_window | frame_mullion | deferred |

---

## `kit_lod0_002` — FIXED (fleet MCP-D0-002)

| module_id | archetype | bpy `profile` / params | pbr_status | bpy |
|:---|:---|:---|:---|:---:|
| `wall_wood_1u` | module_wall | flat | deferred | **SHIPPED** |
| `wall_steel_1u` | module_wall | flat | deferred | **SHIPPED** |
| `roof_flat` | module_roof | flat | deferred | **SHIPPED** |
| `door_shop` | module_door | frame @ 2×2.5m | deferred | **SHIPPED** |
| `win_double_1u` | module_window | frame_mullion @ 4m | deferred | **SHIPPED** |

---

## `kit_lod0_003` — FIXED (MCP-PLN-002 pick)

| module_id | archetype | bpy `profile` / params | grid | pbr_status | bpy |
|:---|:---|:---|:---|:---|:---:|
| `wall_concrete_2u` | module_wall | flat @ 8×3×0.3m | [2,1] | deferred | **SHIPPED** |
| `roof_sawtooth` | module_roof | sawtooth @ 8×0.3×8m | [2,2] | deferred | **SHIPPED** |
| `door_warehouse` | module_door | frame @ 4×3×0.25m | [2,1] | deferred | **SHIPPED** |
| `win_industrial_3u` | module_window | strip @ 12×1.2×0.12m | [3,1] | deferred | **SHIPPED** |
| `prop_vent` | module_prop | prop_kind: vent @ 1×0.4×1m | [1,1] | deferred | **SHIPPED** |

**Designer-mcp notes:**

- No `greybox:*` references.
- `door_warehouse`: use `profile: frame` with warehouse dimensions (not separate bpy op).
- `win_industrial_3u`: `profile: strip` (3m grid width = 12m facade bay).
- First **prop** in lod0 lane — validates PG-2 roof clutter read.

**Open blockers for D0-003:** **none**.

---

## 40-module wave roadmap (`kit_lod0_003` → `kit_lod0_010`)

**Fleet MCP-PLN-002:** 50 − lod0_001/002 = **40 left**. Eight waves × 5 modules (003 is wave 1 of 40). Order balances **category debt** (props last) and introduces **new bpy profiles** before production tier.

| Wave | batch_id | Modules (canonical ids) | New bpy / validator focus |
|:---|:---|:---|:---|
| **004** | `kit_lod0_004` | `wall_brick_2u`, `wall_wood_2u`, `roof_pitched_hip`, `door_garage`, `win_arched_1u` | window `arched`; roof `pitched` variant |
| **005** | `kit_lod0_005` | `wall_glass_curtain_1u`, `wall_industrial_panel_2u`, `roof_shed`, `door_office`, `win_strip_2u` | wall thin depth 0.1m |
| **006** | `kit_lod0_006` | `wall_military_bunker_1u`, `roof_parapet`, `door_civic`, `win_shop_2u`, `prop_light` | prop_kind box/light |
| **007** | `kit_lod0_007` | `roof_metal_low`, `roof_tile`, `door_military`, `win_house_1u`, `corner_L` | prop_kind l_corner |
| **008** | `kit_lod0_008` | `roof_bunker`, `roof_canopy`, `door_factory`, `win_office_1u`, `corner_T` | |
| **009** | `kit_lod0_009` | `door_double_shop`, `door_gate_industrial`, `win_bunker_slit`, `win_skylight_1u`, `corner_parapet` | door wide gate 4u |
| **010** | `kit_lod0_010` | `prop_fence`, `prop_tank`, `prop_transformer`, `prop_ac`, `prop_chimney` | props-only wave |

After **010**: all **50** canonical ids have **lod0** index rows (PG-2-complete silhouette set).

---

## bpy / tooling matrix (SHIPPED vs PLANNED)

| Archetype | Profiles in repo today | PLANNED |
|:---|:---|:---:|
| `module_wall` | flat (panel recess optional) | recess panel variant polish |
| `module_roof` | flat, pitched_gable, shed, sawtooth | — |
| `module_door` | frame (any dims) | named profiles `warehouse`/`garage` cosmetic only |
| `module_window` | frame_mullion, strip, arched | curtain wall depth |
| `module_prop` | box, l_corner, vent, ac | tank, chimney mesh detail |

**All `kit_lod0_003` modules:** **SHIPPED** bpy — no coder-mcp op work before D0-003.

**Waves 004–010:** **SHIPPED** except optional polish; no new op required for roadmap acceptance.

---

## `kit_production_*` (DEFER)

| Wave | When | Notes |
|:---|:---|:---|
| `kit_production_001` | After Material Maker lane **SHIPPED** or explicit `pbr_status: shipped` policy waiver documented | Start with 5 walls + tileable PBR set ids |
| `kit_production_002+` | Follow lod0 complete | Full validation contract § PBR required |

Do not schedule `kit_production_001` in fleet queue until MCP-T0 or material exec plan updates status.

---

## Gate / fleet alignment

| Batch | designer-mcp | coder-mcp |
|:---|:---|:---|
| lod0_002 | MCP-D0-002 (G0–G1) | MCP-C0-002 after G1 |
| lod0_003 | MCP-D0-003 (G0–G1) — **unblocked** | MCP-C0-003 after G1 |
| lod0_004+ | Schedule after lod0_003 G1 | Sequential G3–G5 per batch |

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-06-02 | MCP-PLN-002 — lod0_003 pick + 35-module wave roadmap |
| v1.0.1 | 2026-06-02 | Align fleet wording: 40 modules = lod0_003–010 |
