# VFX tactical capture status — wave 5 (DESIGN-VFX-CAPTURE-WAVE5-001) `v1`

| Field | Value |
|:---|:---|
| **Date** | 2026-05-26 |
| **Queue** | **DESIGN-VFX-CAPTURE-WAVE5-001** |
| **Variant** | wave 5 capture matrix |
| **Verdict** | **PASS (qualified)** — matrix + witness **PASS**; flip to **PASS** when wave5 dated PNGs on disk |
| **Prior round** | [`vfx_capture_status_wave4.md`](vfx_capture_status_wave4.md) · [`vfx_capture_status_20260525.md`](vfx_capture_status_20260525.md) |
| **Witness** | `debug_runs/stage5_full_app_live.json` → `/tactical_vfx_witness/all_green` |
| **Do not break** | `/tactical_vfx_witness/all_green == true` |

---

## Capture matrix (what operators should capture in sim)

Capture style: **tactical zoom stills** (no editor replay panel).

| Domain | Expected PNG name pattern | Comparison notes |
|:---|:---|:---|
| Tactical Fire | `fire_tactical_YYYYMMDD.png` | Compare fire sparks vs `elemental_sparks/fire_spark_target_v1.png` |
| Tactical Water (river) | `river_tactical_YYYYMMDD.png` | Check river streaks + motion continuity |
| Tactical Water (lake/ocean) | `water_lake_or_ocean_YYYYMMDD.png` | Verify panel readability + no missing foam |
| Logistics overlay | `logistics_tactical_YYYYMMDD.png` | Confirm logistics rows render above terrain |
| Construction MV delta | `construction_mv_tactical_YYYYMMDD.png` | Corridor phase overlays should not hide tactical water/fire |

---

## Witness audit (qualified baseline)

| Domain | Witness | Verdict |
|:---|:---|:---|
| **Harness rollup** | `tactical_vfx_witness.all_green: true` | **PASS** |
| **Fire** | tactical spark rows @ zoom α 0.85 | **PASS** (D-F09) |
| **Water** | `water_witness_rollup_green`, W1/W2 rows | **PASS** |
| **Strategic cull** | zero particle rows at strategic zoom | **PASS** by design |

**Designer rule:** Witness green + signed W1/W2 closure = **qualified** unblocks operator/coder polish; fresh wave5 PNGs are **optional** refresh (same policy as wave 4 round-002).

---

## Acceptance (designer) — checklist complete

1. ☑ Capture matrix covers tactical fire + water + overlay + construction coexistence.
2. ☑ Construction MV overlay must not occlude VFX panels (documented in matrix).
3. ☑ Interim PNGs from wave 4 acceptable until operator run (see below).

---

## On disk (interim — wave 4 baseline)

| File | Status | Notes |
|:---|:---|:---|
| `fire_tactical_20260524.png` | **ON DISK** (interim) | From wave 4 round; witness green |
| `water_river_tactical_20260524.png` | **ON DISK** (interim) | River read witness green |
| `water_lake_tactical_20260524.png` | **ON DISK** (interim) | Lake/ocean witness green |

**Wave 5 operator optional refresh:**

| Pattern | Status |
|:---|:---|
| `fire_tactical_20260526.png` | TBD — optional |
| `river_tactical_20260526.png` | TBD — optional |
| `water_lake_or_ocean_20260526.png` | TBD — optional |
| `logistics_tactical_20260526.png` | TBD — optional |
| `construction_mv_tactical_20260526.png` | TBD — optional |

---

## Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer` | **PASS (qualified)** | 2026-05-26 |

**Unblocks:** operator capture lane **VFX-CAPTURE-WAVE5** (optional PNG round); does not reopen closed VFX design reviews.
