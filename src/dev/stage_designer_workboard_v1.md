# Designer workboard `v1` (active)

| Field | Value |
|:---|:---|
| **Version** | `1.0.0` |
| **Date** | 2026-05-24 |
| **Sign-off ledger** | [`stage_tracks_signoff_ledger_v1.md`](stage_tracks_signoff_ledger_v1.md) |

**Rule:** **SIGNED** items are complete for design. **OPEN** items block coders only where noted.

---

## Done — no rework

| ID | Track | Deliverable | Record |
|:---|:---|:---|:---|
| **FX-WATER-DESIGN** | FX-WATER | D-W01…D-W10 worksheet | design plan §11 SIGNED |
| **WATER-DESIGN-001** | FX-WATER | Review | [`water_vfx_review_record_v1.md`](water_vfx_review_record_v1.md) **SIGNED — TUNE** |
| **D-VFX** | VFX-P2 | Fire+water review | [`vfx_design_review_record_v1.md`](vfx_design_review_record_v1.md) **SIGNED — TUNE** |
| **FX-FIRE-SPARK-DESIGN** | FX-FIRE | D-F01…D-F10 | fire spark design plan SIGNED |
| **UI-WP-DESIGN** | UI-P4 | D-01…D-12 layout | world_map_preview_layout_decision SIGNED |
| **UI-P2-DESIGN** | UI-P2 | Phase 2 shell | ui_phase2_designer_signoff v2.2 SIGNED |

---

## Active — do these

### 1. S7P-DESIGN-001 — Stage 7 Play scenario (**blocks S7-PLAY exit**)

**Read:** [`stages/stage7_play_plan_v1.md`](stages/stage7_play_plan_v1.md)

**Edit:** [`stage7_play_scenario_v1.md`](stage7_play_scenario_v1.md)

| Step | Action |
|:---|:---|
| 1 | Run sim or `--test visual` with concrete chain |
| 2 | Check boxes in scenario table |
| 3 | Set header **Status: SIGNED** when reproducible |

**Unblocks:** operator sign-off on Stage 7 Play track.

---

### 2. UI4-DESIGN-001 — World preview D-04 slide sheet (**blocks UI-WP-LAYOUT-002**)

**Read:** [`stages/ui_phase4_execution_plan_v1.md`](stages/ui_phase4_execution_plan_v1.md) · [`world_map_preview_layout_decision_v1.md`](../prompts/guides/ui/world_map_preview_layout_decision_v1.md) § D-04

**Deliver:**

- `assets/ui/world_preview/slide_sheet_spec_v1.png` (or annotated mock)
- Sheet height %, dimmed map treatment, entry control (tab/button)

**Unblocks:** `@coder` **UI-WP-LAYOUT-002**.

---

### 3. Operator VFX captures (**optional for ACCEPTED, not blocking coders**)

**Read:** [`vfx_post_implementation_review_v1.md`](../prompts/guides/ui/vfx_post_implementation_review_v1.md)

| PNG | Path |
|:---|:---|
| Fire tactical | `assets/vfx/reference/review_captures/fire_tactical_20260524.png` |
| River tactical | `assets/vfx/reference/review_captures/water_river_tactical_20260524.png` |
| Lake tactical | `assets/vfx/reference/review_captures/water_lake_tactical_20260524.png` |

**After captures:** bump review records to **PASS** where mocks match.

---

### 4. WATER-DESIGN-002 — Ocean fixture seed (optional)

**Read:** [`water_vfx_review_record_v1.md`](water_vfx_review_record_v1.md) § re-review

Name one world-gen seed or test fixture with `water_ocean_tiles > 0` for **@coder** **WATER-W1-OCEAN-001**.

---

## Gated — do not start yet

| ID | Prerequisite |
|:---|:---|
| **S7B-DESIGN-001** | UI-WP-LAYOUT-002 done + INFRA-VM09-001 |
| **UI4-DESIGN-003** (WP-L4 map look) | After LAYOUT-002 |
| **P4-ART-01** (icon atlas PNG) | Optional anytime |

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-24 | Active designer queue from sign-off ledger |
