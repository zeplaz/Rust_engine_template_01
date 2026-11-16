# Studio massing ↔ grammar bridge `v1`

| Field | Value |
|:---|:---|
| **⟨ID⟩** | **STUDIO-MASSING-GRAMMAR-BRIDGE-001** |
| **Owner** | `@planner` (schema) → `@coder-mcp` (export) → `@coder` (demo / test scene) |
| **Status** | **SIGNED** |
| **Date** | 2026-11-15 |
| **Parents** | [`plan_voxel_massing_presentation_v1.md`](plan_voxel_massing_presentation_v1.md) · [`plan_artist_studio_expansion_v1.md`](plan_artist_studio_expansion_v1.md) · [`plan_operator_build_readability_exec_001_v1.md`](plan_operator_build_readability_exec_001_v1.md) |
| **Deferral** | **DR-VOXEL-OCCUPATION** — presentation only |

**Rule:** Grammar footprints and APS `massing_preview.json` feed **ghost silhouette** only. Never write `FootprintMatrix`, commit payloads, or sim occupation.

---

## 1. Authority map

| Lane | Owner | Owns | Must not own |
|:---|:---|:---|:---|
| Grammar / pilots | `@designer-mcp` / `@coder-mcp` | `_pilot_catalog.ron`, `_mock_shapes.ron`, `building_grammar.generate`, `grammar_rule_chain` | Bevy ECS, `massing_columns` resource |
| APS conditioning (IO-09) | `@coder-mcp` | Seeded `massing_preview.json`, StudioTask `massing_export`, Republic HTML QC | `FootprintMatrix`, dig/place |
| Engine derive (PCG-06) | `@coder` | `StudioMassingOverride`, sole writer of `ConstructionVisualRequests.massing_columns` | GPU meshes, APS UI |
| Paint (RND-01) | `@coder` | RO consume columns in `world_map_frame` / `pixel_world` | Height heuristics, occupation |

```text
pilot / ARCH-DNA
  → building_grammar.generate + grammar_rule_chain
  → footprint cells → massing_columns[]
  → massing_preview.json + studio_dash HTML
  → RUST_ENGINE_STUDIO_MASSING=1 / StudioMassingOverride
  → sync_voxel_massing_visual_requests (sole writer)
  → pixel_world / world_map ghost silhouette
```

---

## 2. REJECT ledger (unusable book / concept ties)

Do **not** schedule work that invents these bridges. `lookup.py voxel` may surface false friends — they are **not** adoption targets. Primary matrix row for code: **PCG-06**.

| Tempting claim | Why reject | Redirect |
|:---|:---|:---|
| Voxel physics / PHYS-* on stacks | floors = paint heuristic, not bodies | stacked quads only |
| AI / behaviour trees on voxels | no DEC/TAC ↔ massing link | construction commit + PCG-06 grammar |
| NAV-05 pathfinding on column stacks | 2D logistics floodfill | NAV logistics rows |
| LGT-04 volumetrics / LGT-07 AO as massing authority | homonym / whole-module index noise | RND-01 + rim tokens |
| Gameplay validity from stack height | VOXEL-MASSING-PRES-001 non-goal | `FootprintMatrix` / site phase |
| RND-09 as current paint path | deferred until readability green | follow-on instancing |
| Weather `voxel_hero_budget` | unrelated mesh heroes | air presentation policy |
| Sim voxel occupation / dig | **DR-VOXEL-OCCUPATION** | future serial plan only |

**Keep:** PCG-06 · IO-09 · ECS-15 · RND-01 · RND-18 · MATH-06 · RND-05 (zoom collapse).

---

## 3. Program phases (acceptance)

| Phase | Deliverable | Status |
|:---|:---|:---|
| **0** | This ledger + voxel plan §8 status / reject note | **SHIPPED** 2026-11-15 |
| **1** | Grammar/pilot → `massing_preview` export + schema lineage fields | **SHIPPED** — `massing_from_pilot.py` |
| **2** | Republic dash fields + `massing_export` task completion | **SHIPPED** — `run_queued_massing_export` |
| **3** | Engine demo harness (checked-in fixture → override) | **SHIPPED** — `RUST_ENGINE_STUDIO_MASSING=fixture` |
| **4** | `BuildReadability` test scene from catalog pilots (no warehouse Rust branches) | **SHIPPED** — `--test buildread` |
| **5** | Grammar G4 / PCG-06 S4 coverage sibling (feeds export; not voxel physics) | **SHIPPED** — tier G4 + rule coverage ADOPTED |

---

## 4. Demo harness

Operator steps: [`tools/studio_dash/demo_massing_override.md`](../../tools/studio_dash/demo_massing_override.md).

- Env `RUST_ENGINE_STUDIO_MASSING=fixture` → checked-in pilot JSON
- Lib: `voxel_massing::tests::checked_in_pilot_fixture_loads_override`

## 5. Play-test world (sibling)

Quadrant fire / build-order / hydrology / rain-in-PixelWorld live in
[`plan_playtest_quadrant_world_v1.md`](plan_playtest_quadrant_world_v1.md).
**buildread + VoxelMassing defy** fire seeds and harness test-particle chrome;
**PixelWorld product rain** is WeatherVfx (not `massing_columns`).

## 6. Out of scope

- `plan_voxel_occupation_v1` / dig / Minecraft sim
- Structural collapse, AI/BT on columns, navmesh on floors
- Inferring capacity / gameplay from stack height
- Fire / test-particle / LGT-04-as-massing on the massing path
