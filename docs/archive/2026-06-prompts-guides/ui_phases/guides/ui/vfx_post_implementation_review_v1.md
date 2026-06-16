# Post-implementation VFX review `v1`

| Field | Value |
|:---|:---|
| **Version** | `1.0.0` |
| **Date** | 2026-05-24 |
| **Owner** | `@designer` |
| **Status** | **DONE** — do **not** re-run; see [`vfx_design_review_record_v1.md`](../../../docs/archive/2026-06-src-dev/plans/vfx_design_review_record_v1.md) (**D-VFX**) |
| **Canonical queue ID** | **DESIGN-D-VFX-POST-001** |
| **Related** | **DESIGN-VFX-CAPTURE-001** (`VFX-CAPTURE-001`) — tactical PNGs · aliases **D-VFX** · **VFX-POST-REVIEW-DESIGN** |
| **When** | After **@coder A** **P2-FIRE-SPARK-011** + **P2-VFX-VISUAL-001** green (signed 2026-05-25) |
| **Coder queue** | [`vfx_coder_phase2_queue_v1.md`](../../../docs/archive/2026-06-src-dev/plans/vfx_coder_phase2_queue_v1.md) |
| **Machine queue** | `VFX-POST-REVIEW-DESIGN` in [`continuation_queue.json`](../../../tools/orchestrator/queues/continuation_queue.json) |

---

## Purpose

Confirm **in-engine** fire sparks and water surfaces match the **signed** design intent (D-F* / D-W*) using the reference mocks.

**Already completed elsewhere** — this brief is the **procedure**; the authoritative outcome lives in:

| Artifact | Role |
|:---|:---|
| [`vfx_design_review_record_v1.md`](../../../docs/archive/2026-06-src-dev/plans/vfx_design_review_record_v1.md) | **D-VFX** — fire **PASS** (2026-05-25) |
| [`water_vfx_review_record_v1.md`](../../../docs/archive/2026-06-src-dev/plans/water_vfx_review_record_v1.md) | **WATER-DESIGN-001** — water **PASS** |
| [`vfx_capture_status_20260525.md`](../../../assets/vfx/reference/review_captures/vfx_capture_status_20260525.md) | Capture audit (**VFX-CAPTURE-001**) |
| Interim PNGs | `review_captures/fire_tactical_20260524.png`, `water_*_20260524.png` |

**Do not** open a second **VFX-POST-REVIEW-DESIGN** pass unless re-baselining after major shader changes.

**Blocking:** nothing. **Output:** (historical) verdict + captures under `assets/vfx/reference/review_captures/`.

---

## Prerequisites (coder done)

| Slice | Minimum evidence |
|:---|:---|
| **P2-VFX-VISUAL-001** | `debug_runs/stage5_full_app_live.json` refreshed at **tactical** zoom (`zoom_alpha ≥ 0.65`) |
| **P2-FIRE-SPARK-011** (Coder A) | `fire_spark_011_green: true` @ `fire_spark_zoom_alpha: 0.85` — run **before** designer sign-off |
| Fire | `fire_spark_rows > 0` when tactical; sparks visible in sim on fire cells |
| Water | `water_w1_green: true`; rivers readable on map (not lake-only teal) |
| Witness | `cargo test -p proc_A_dine01 --lib stage5` green |

If particles are **zero** at tactical zoom, send back to **@coder** (`P2-VFX-VISUAL-001`) — do not tune shaders from strategic-zoom captures.

---

## Reference mocks (authority for compare)

| Domain | Mock | Signed decisions |
|:---|:---|:---|
| **Fire** | [`assets/vfx/reference/elemental_sparks/fire_spark_target_v1.png`](../../../assets/vfx/reference/elemental_sparks/fire_spark_target_v1.png) | [`fire_particle_spark_decision_worksheet_v1.md`](fire_particle_spark_decision_worksheet_v1.md) D-F01…D-F10 |
| **Water** | [`assets/vfx/reference/water/water_surface_target_v1.png`](../../../assets/vfx/reference/water/water_surface_target_v1.png) | [`water_surface_vfx_decision_worksheet_v1.md`](water_surface_vfx_decision_worksheet_v1.md) D-W01…D-W10 |

Side-by-side layout in `fire_spark_target_v1.png`: **left = blob (reject)**, **right = pinpoint (target)**.

---

## Capture procedure (~20 min)

1. Run sim with fire + water on map:
   ```powershell
   cargo run -p proc_A_dine01 --release
   ```
2. Enter **Simulation**; zoom to **tactical** (map fill ~40–70% of viewport — sparks/particles must not be strategic-culled).
3. Frame **three** captures:
   - **Fire** — active wildfire or test fire cells; sparks visible on terrain.
   - **River** — hydrology path with flow read (narrow strip, directional motion if W1 landed).
   - **Lake or ocean** — standing water + optional coast foam.
4. Save PNGs (1920×1080 or crop) to:
   ```
   assets/vfx/reference/review_captures/
   ├── fire_tactical_YYYYMMDD.png
   ├── river_tactical_YYYYMMDD.png
   └── water_lake_or_ocean_YYYYMMDD.png
   ```
5. Optional: paste captures beside mocks in Figma/Photoshop for § Review checklist.

**Do not** use headless `stage5_full_app_live.json` screenshots alone — they run at strategic zoom by default.

---

## Review checklist — fire (D-F*)

Compare capture vs **right panel** of `fire_spark_target_v1.png`.

| ID | Check | Pass if |
|:---|:---|:---|
| **D-F01** | Silhouette | Pinpoint / ≤2px cores — **not** soft orange blobs |
| **D-F04** | Age | Dark ash → hot orange along particle life |
| **D-F05** | Twinkle | Position-based flicker (not whole-map pulse) |
| **D-F07** | Density | Many small points, low α — shower not 3–5 large discs |
| **D-F08** | Blend | Hot cores read additive; edges not muddy alpha soup |
| **D-F09** | Zoom | Sparks **visible** at tactical; absent or faint at strategic zoom-out |
| **D-F10** | Smoke | Sparks read **on top of** smoke haze, not buried |

**Phase B (if landed):** motion toward fire cores feels organic (D-F02 B) — optional note.

| Verdict | Fire |
|:---|:---|
| ☐ **PASS** | Matches target mock intent |
| ☐ **TUNE** | Close — file numbered tickets below |
| ☐ **FAIL** | Still blob-like or invisible at tactical — escalate to `@coder` |

---

## Review checklist — water (D-W*)

Compare capture vs `water_surface_target_v1.png` panels (lake / river / ocean).

| ID | Check | Pass if |
|:---|:---|:---|
| **D-W01** | River read | River **visible** as channel — not identical flat lake tile |
| **D-W02** | Lake | Slow ripple or tonal variation — not static flat teal slab |
| **D-W03** | River motion | Directional flow along path (scroll or streak bias) |
| **D-W04** | Ocean | Deeper tone / swell or haze vs lake |
| **D-W05** | Particles (W2) | If enabled: pinpoint glints/streaks — same family as fire |
| **D-W07** | River particles | Downstream streaks on centerline (if W2 landed) |
| **D-W09** | Zoom | Water particles fade when zoomed out (tactical only test) |

| Verdict | Water |
|:---|:---|
| ☐ **PASS** | Lake / river / ocean distinguishable per mock |
| ☐ **TUNE** | River still weak or ocean flat — numbered tickets |
| ☐ **FAIL** | Rivers still “missing” — **P2-WATER-POLISH-001** / W1 |

---

## Tune tickets (if TUNE)

Use this table; `@coder` picks slice from [`vfx_coder_phase2_queue_v1.md`](../../../docs/archive/2026-06-src-dev/plans/vfx_coder_phase2_queue_v1.md).

| # | Domain | Issue | Suggested slice | Notes |
|:---|:---|:---|:---|:---|
| T-01 | Fire | | P2-FIRE-SPARK-011 | e.g. density, twinkle strength |
| T-02 | Fire | | P2-FIRE-SPARK-010 | e.g. under smoke |
| T-03 | Water | | P2-WATER-POLISH-001 | e.g. ribbon width, flow speed |
| T-04 | Water | | FX-WATER-SHADER-001 | W1 overlay only |

**Not in scope:** new hydrology sim, Hanabi, second fire extract, Phase 4 icon atlas (separate optional lane).

---

## Sign-off record (this review)

**Latest record (D-VFX):** [`docs/archive/2026-06-src-dev/plans/vfx_design_review_record_v1.md`](../../../docs/archive/2026-06-src-dev/plans/vfx_design_review_record_v1.md) — fire + water summary

**Water-only (FX-WATER track):** [`docs/archive/2026-06-src-dev/plans/water_vfx_review_record_v1.md`](../../../docs/archive/2026-06-src-dev/plans/water_vfx_review_record_v1.md) — **WATER-DESIGN-001** SIGNED TUNE (2026-05-24)

**Captures folder:** [`assets/vfx/reference/review_captures/`](../../../assets/vfx/reference/review_captures/) — PNGs pending

| Field | Value |
|:---|:---|
| **Reviewer** | Design pass (Auto) |
| **Date** | 2026-05-24 |
| **Build / commit** | `b2341a6` |
| **Captures** | Pending — `fire_tactical_*`, `river_tactical_*`, `water_lake_or_ocean_*` |
| **Fire verdict** | ☑ **TUNE** · ☐ PASS · ☐ FAIL |
| **Water verdict** | ☑ **TUNE** · ☐ PASS · ☐ FAIL |
| **Overall** | ☑ **TUNE ROUND** · ☐ **ACCEPTED** |

---

## @designer copy-paste

```text
@designer VFX-POST-REVIEW-DESIGN — ALREADY DONE (alias D-VFX)

Do NOT re-run unless re-baselining shaders.

Authority (complete):
  docs/archive/2026-06-src-dev/plans/vfx_design_review_record_v1.md — SIGNED PASS 2026-05-25
  docs/archive/2026-06-src-dev/plans/water_vfx_review_record_v1.md — water PASS
  assets/vfx/reference/review_captures/vfx_capture_status_20260525.md
  Queue: DESIGN-VFX-CAPTURE-001 done

Optional polish only: P2-FIRE-SPARK-011 (F-T03), operator refresh PNGs
```

---

## Document history

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-24 | Initial post-impl review brief |
