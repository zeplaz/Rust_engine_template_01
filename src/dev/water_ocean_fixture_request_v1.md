# Water VFX — ocean fixture request `WATER-DESIGN-002`

| Field | Value |
|:---|:---|
| **Queue ID** | **WATER-DESIGN-002** |
| **Version** | `1.0.0` |
| **Date** | 2026-05-24 |
| **Owner** | `@designer` (Design pass) |
| **Status** | **SIGNED** (2026-05-24) |
| **Review** | [`water_vfx_review_record_v1.md`](water_vfx_review_record_v1.md) |

---

## Problem

**Resolved (2026-05-25):** visual proof `water_ocean_tiles: 1715`, `water_particle_coast_foam: 128` in `stage5_full_app_live.json` after lake-shore → ocean catalog split.

---

## Fixture (signed)

| Field | Value |
|:---|:---|
| **Primary (CI / coder)** | Unit test **`water_surface_visual::tests::water_w1_ocean_001_dem_deep_band_fills_ocean_tiles`** |
| **Command** | `cargo test -p proc_A_dine01 --lib water_surface_visual::tests::water_w1_ocean_001_dem_deep_band_fills_ocean_tiles` |
| **Expected witness** | `water_ocean_tiles > 0` in `stage5_full_app_live.json` after catalog build uses DEM deep band |
| **Map note (visual)** | Use world with `biome_tuning.deep_water_height_max` band + border hydro lakes; or run test fixture in harness before `--test visual` |
| **World-gen seed (operator optional)** | *Any seed where coastal DEM exposes deep_water band* — refresh witness after place |

**Verified:** unit tests `water_w1_ocean_001_*` **ok** · visual `--test visual` **ok** (`water_ocean_tiles > 0`).

---

## Coder handoff

```
Track: FX-WATER — WATER-W1-OCEAN-001
Read: src/dev/water_ocean_fixture_request_v1.md (SIGNED)
First: wire visual proof / stage5 witness to build catalog from map with ocean band OR inject test hydro
Verify: cargo test -p proc_A_dine01 --lib water_surface_visual::tests::water_w1_ocean_001_dem_deep_band_fills_ocean_tiles stage5
Witness: stage5_full_app_live.json water_ocean_tiles > 0
```

---

## Sign-off

| Role | Date | Verdict |
|:---|:---|:---|
| Designer | 2026-05-24 | **SIGNED** |

**Unblocks:** **WATER-W1-OCEAN-001** — coder A uses test + map criteria above.

---

## Document history

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-24 | **WATER-DESIGN-002** signed; unit test fixture named |
