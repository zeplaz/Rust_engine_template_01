# Legacy asset reference manifest `v1`

| Field | Value |
|:---|:---|
| **Version** | `1.0.0` |
| **Date** | 2026-05-23 |
| **Role** | Canonical disk paths for UI silhouette traces and tile-map sources |
| **Child briefs** | [`ui_phase4_icon_atlas_brief_v1.md`](ui_phase4_icon_atlas_brief_v1.md) |

**Rule:** Gameplay loaders may use other paths; **UI atlas bakes** must trace from rows below unless a brief explicitly overrides.

---

## Power / utilities

| Id | Path | Notes |
|:---|:---|:---|
| `power_transformer_oil_cooled` | `assets/textures/power/tile_map_powerstuff_power_trasformer_oil_cooled_alpha.png` | **Utilities (UT)** authoritative |
| `power_mobile_generator_dev` | `assets/textures/power/tile_map_rust_dev_utils_alpha.png` | **Generator trailer** — **not** UT; QA disambig only (`UT_MG`) |

---

## Vehicles (silhouette sheets)

| Id | Path |
|:---|:---|
| `vehicle_civ_truck_empty` | `assets/textures/vehicles/civ_truck_01/tile_map_8_empty_miday.png` |
| `vehicle_ural_empty` | `assets/textures/vehicles/ural_01/tile_map_ural_01_empty_midday.png` |
| `vehicle_bus_alpha` | `assets/textures/vehicles/bus_01/tilemap_bus_01_alpha.png` |

---

## Misc / petroleum

| Id | Path | Notes |
|:---|:---|:---|
| `misc_barrel_alpha` | `assets/textures/misc/hjm-barrel_alpha.png` | P5 petroleum tab |
| `misc_railroad_track` | `assets/textures/misc/railroad_track.png` | RD / RL trace reference |

---

## Industry / civil (crop TBD in bake)

| Id | Path | Notes |
|:---|:---|:---|
| `misc_cities` | `assets/textures/misc/cities.png` | CV civic mass crop |
| `misc_wooden_buildings` | `assets/textures/misc/wooden_buildings_01.png` | IN factory mass crop |

---

## Orchestrator mirror

Machine index: [`tools/orchestrator/knowledge/ui_texture_assets.json`](../../../tools/orchestrator/knowledge/ui_texture_assets.json) — extend `phase4_icon_atlas` per Phase 4 brief §9.
