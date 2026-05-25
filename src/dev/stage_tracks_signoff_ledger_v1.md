# Stage tracks — sign-off ledger `v1`

| Field | Value |
|:---|:---|
| **Version** | `1.0.0` |
| **Date** | 2026-05-24 |
| **Authority** | Witness JSON wins over markdown checkboxes |
| **Hub** | [`stage_tracks_execution_index_v1.md`](stage_tracks_execution_index_v1.md) |
| **Refresh** | Re-run `--test visual` + sim session when a row says **STALE** |

**Legend:** **CLOSED** = exit criteria met · **SIGNED** = designer review recorded · **DONE** = coder slice landed · **OPEN** = active work · **STALE** = proof JSON out of date vs code

---

## Executive summary (2026-05-24)

| Area | Verdict | Next |
|:---|:---|:---|
| **Stage 5 / 6 gates** | **CLOSED** (historical sign-offs) | Maintain regression only |
| **VFX tactical proof** | **DONE** | Tune + captures (designer TUNE) |
| **Water FX** | **DONE** first pass · **OPEN** closure | W-T01…W-T07 |
| **Fire FX** | **DONE** core · **OPEN** polish | F-T01…F-T03 |
| **Industrial / S7 Play** | **DONE** coder · **OPEN** designer scenario | S7P-DESIGN-001 |
| **UI shell 2B** | **STALE witness** | **UI-SHELL-REFRESH-001** — do not re-implement |
| **UI Phase 3 minimap** | **DONE** | M3 overlays optional |
| **UI Phase 4** | **OPEN** | UI4-DESIGN-001 → LAYOUT-002 |
| **Infra / Wave C / Fire P7 / Behavioral** | **OPEN** | per track plans |

---

## Closed gates (do not reopen)

| ID | Milestone | Evidence | Status |
|:---|:---|:---|:---:|
| **G-S5** | Stage 5 FULL_APP operational | `stage5_full_app_live.json` → `readiness.passes: true` | **CLOSED** |
| **G-S6** | Stage 6 virtualization | `stage6_operational_signoff.md` | **CLOSED** |
| **G-CON-OP** | Construction operational | `construction_stage_live.json` → `operational_green: true` | **CLOSED** |
| **G-WAVE-S** | Wave S save spine code | `wave_s_open.md` | **CLOSED** (code) |

---

## Track sign-off matrix

### Stage 7 Play (`S7-PLAY`)

| Slice / gate | Agent | Status | Evidence |
|:---|:---|:---|:---|
| **S7P-IND-001** | coder | **DONE** | `industrial_activation_live.json` → `activation_green: true`, board I1/SC rows Done |
| **S7P-DESIGN-001** | designer | **OPEN** | [`stage7_play_scenario_v1.md`](stage7_play_scenario_v1.md) DRAFT |
| **CON P9** | coder | **DONE** | `construction_stage_live.json` → `con_e01_p9_green: true` |
| **Track exit** | — | **OPEN** | Needs designer scenario **SIGNED** + operator run |

### VFX Phase 2 — fire (`VFX-P2`)

| Slice / gate | Agent | Status | Evidence |
|:---|:---|:---|:---|
| **FX-FIRE-SPARK-001…006** | coder | **DONE** | queue + code landed |
| **P2-VFX-VISUAL-001** | coder | **DONE** | `fire_spark_rows: 308`, `zoom_alpha: 0.85`, tactical gates green |
| **P2-VFX-WITNESS-001** | coder | **PARTIAL** | lib tests green; mark **done** after README operator line |
| **P2-FIRE-SPARK-010** | coder | **DONE** | `fire_sparks_above_smoke: true` in witness |
| **P2-FIRE-SPARK-011** | coder | **OPEN** | F-T01/F-T03 tune vs mock |
| **D-VFX** (`VFX-POST-REVIEW`) | designer | **SIGNED — TUNE** | [`vfx_design_review_record_v1.md`](vfx_design_review_record_v1.md) |
| **Track exit** | — | **OPEN** | TUNE tickets + optional PNG captures → ACCEPTED |

### Water VFX (`FX-WATER`)

| Slice / gate | Agent | Status | Evidence |
|:---|:---|:---|:---|
| **FX-WATER-SHADER-001/002** | coder | **DONE** | `water_w1_green: true` |
| **FX-WATER-PARTICLE-001/002** | coder | **DONE** | `water_particle_rows: 76`, streaks 27 |
| **P2-VFX-VISUAL-001** (shared) | coder | **DONE** | tactical water gates green |
| **WATER-DESIGN-001** | designer | **SIGNED — TUNE** | [`water_vfx_review_record_v1.md`](water_vfx_review_record_v1.md) |
| **WATER-W1-OCEAN-001** | coder | **OPEN** | `water_ocean_tiles: 0` |
| **WATER-W1-RIVER-001** | coder | **OPEN** | W-T01 strategic ribbon read |
| **WATER-W2-FOAM-001** | coder | **PARTIAL** | `river_foam: 1`, `coast_foam: 0` |
| **WATER-STRATEGIC-001** | coder | **OPEN** | D-W09 strategic band not in visual proof |
| **Track exit** | — | **OPEN** | W-T01…W-T07 + witness exit |

### UI Phase 2 shell (`UI-P2`)

| Slice / gate | Agent | Status | Evidence |
|:---|:---|:---|:---|
| **UI-P2B-001 / 2A / 2C** | coder | **DONE** (code) | prior green proofs |
| **UI-P2-GATE** | sim-steward | **DONE** | CONDITIONAL historical |
| **ui_shell_migration_live.json** | operator | **STALE** | `phase2b_closed: false` — incomplete proof frame, not necessarily code regression |
| **UI-SHELL-REFRESH-001** | operator+coder | **OPEN** | Replay sim interactions; refresh witness |
| **UI-P2A-F03 / P4-AUTH** | coder | **OPEN** | passive proof flags false in stale JSON |

### UI Phase 3 minimap (`UI-P3`)

| Slice / gate | Agent | Status | Evidence |
|:---|:---|:---|:---|
| **UI-P3-M1/M2/001** | coder | **DONE** | `minimap_compositor_live.json` → `composite_ok`, `logistics_rows: 2`, `ui_p3_001_green: true` |
| **Plan v1** | planner | **APPROVED** | `ui_phase3_minimap_compositor_plan_v1.md` |
| **UI-P3-M3** | coder | **OPEN** | `ui_p3_m3_green: false` in compositor witness |

### UI Phase 4 (`UI-P4`)

| Slice / gate | Agent | Status | Evidence |
|:---|:---|:---|:---|
| **UI-WP-DESIGN** | designer | **SIGNED** | layout decision doc |
| **UI-WP-LAYOUT-001** | coder | **DONE** | unified workspace tests |
| **UI4-DESIGN-001** | designer | **OPEN** | D-04 slide sheet spec |
| **UI-WP-LAYOUT-002** | coder | **OPEN** | blocked on UI4-DESIGN-001 |

### Infra 5.5+ (`INFRA-55`) · Wave C · Fire P7 · Behavioral

| Track | Status | Note |
|:---|:---|:---|
| **INFRA-55** | **OPEN** | INFRA-PREFLIGHT-001 queued |
| **WAVE-C** | **OPEN** | stage6 witness refresh ops |
| **FIRE-P7** | **PLANNING** | planner arch doc first |
| **S7-BEHAV** | **GATED** | prerequisites not met |

---

## Designer workboard (active only)

| Priority | ID | Track | Deliverable | Status |
|:---:|:---|:---|:---|:---:|
| 1 | **S7P-DESIGN-001** | S7-PLAY | Sign [`stage7_play_scenario_v1.md`](stage7_play_scenario_v1.md) | **OPEN** |
| 2 | **UI4-DESIGN-001** | UI-P4 | `slide_sheet_spec_v1.png` + D-04 answers | **OPEN** |
| 3 | **Operator captures** | VFX | PNGs in `review_captures/` (fire, river, lake) | **OPEN** |
| 4 | **WATER-DESIGN-002** | FX-WATER | Ocean fixture seed name (optional) | **OPEN** |
| — | **D-VFX / WATER-DESIGN-001** | VFX | **SIGNED TUNE** — re-review after coder tune | **DONE** |
| — | **UI-P2-DESIGN** | UI-P2 | v2.2 SIGNED historical | **CLOSED** |
| 5 | **S7B-DESIGN-001** | S7-BEHAV | Worksheet (after UI-P4 + VM-09) | **GATED** |

---

## Coder workboard (active only)

| Priority | ID | Track | First action | Status |
|:---:|:---|:---|:---|:---:|
| 1 | **WATER-W1-OCEAN-001** | FX-WATER | Ocean tiles + swell witness | **OPEN** |
| 2 | **WATER-W2-FOAM-001** | FX-WATER | Coast foam + more bend foam | **PARTIAL** |
| 3 | **WATER-W1-RIVER-001** | FX-WATER | Strategic river ribbon read | **OPEN** |
| 4 | **WATER-STRATEGIC-001** | FX-WATER | Strategic particle cull test | **OPEN** |
| 5 | **P2-FIRE-SPARK-011** | VFX-P2 | Spark tuning vs mock | **OPEN** |
| 6 | **UI-WP-LAYOUT-002** | UI-P4 | After UI4-DESIGN-001 | **BLOCKED** |
| 7 | **UI-SHELL-REFRESH-001** | UI-P2 | Refresh shell witness (replay) | **OPEN** |
| 8 | **UI-P2A-F03 / P4-AUTH** | UI-P2 | Interaction replay proof | **OPEN** |
| — | **S7P-IND-001** | S7-PLAY | — | **DONE** |
| — | **P2-VFX-VISUAL-001** | VFX-P2 | — | **DONE** |
| — | **P2-FIRE-SPARK-010** | VFX-P2 | — | **DONE** |

---

## Witness refresh commands

```powershell
# Spine + VFX + industrial
cargo test -p proc_A_dine01 --lib stage5
cargo run -p proc_A_dine01 --release -- --test visual

# Industrial board
cargo test -p proc_A_dine01 economy::activation --lib

# UI shell (needs sim interactions: tray, rail, minimap)
cargo run -p proc_A_dine01 --release
# then play through stage7_play_scenario steps → triggers ui_shell_migration_live.json
```

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-24 | Initial reconciliation vs debug_runs + review records |
