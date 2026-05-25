# World Map Preview — designer next steps `v1`

| Field | Value |
|:---|:---|
| **Version** | `1.1.0` |
| **Date** | 2026-05-24 |
| **Owner** | `@designer` |
| **Status** | **COMPLETE** — SIGNED 2026-05-24 · **UI-WP-LAYOUT-001** unblocked |
| **Coder gate** | [`ui_world_preview_coder_queue_v1.md`](ui_world_preview_coder_queue_v1.md) |
---

## What you are signing

You are **not** signing terrain art or GPU compositing. You are signing **chrome layout + motion + materiality** for WorldGen / World Map Preview so `@coder` may refactor panel shells.

**Coders may now (no sign-off):** raster/GPU bugfixes per [`world_preview_runbook_v1.md`](../world_preview_runbook_v1.md).

**Coders blocked until SIGNED:** panel shells, asymmetry offsets, motion systems, merged workspace, paper frames.

---

## Designer workflow (≈45–90 min)

```text
1. Read §1–§4  (north star + wireframe)     ~15 min
2. Walk §5 D-01…D-12 on worksheet           ~20 min
3. Commit 1920×1080 mock PNG                ~30 min
4. Mark §11 checklist + SIGNED row            ~10 min
5. Notify @coder → UI-WP-LAYOUT-001 opens
```

---

## Step 1 — Read context (15 min)

| Section | Why |
|:---|:---|
| §1 North star | Archive table, not esports HUD |
| §2 Current code | Dual floating windows today |
| §3 REMOVE / ADD | What coders must tear down |
| §4 Target wireframe | Default proportions to validate or amend |

**Palette:** [`design_theme.md`](design_theme.md) + §1 token table (registration magenta = pigment, not glow).

---

## Step 2 — Walk §5 D-01…D-12 (20 min)

Open **[`world_preview_layout_decision_worksheet_v1.md`](world_preview_layout_decision_worksheet_v1.md)**.

For each row:

1. Read Option A / B / C in the authority doc §5.
2. Circle **A**, **B**, or **C** — or write a short amend note.
3. If you accept the **recommended** column, you may leave defaults and only mark overrides.

**Recommended smooth-flow defaults (override freely):**

| ID | Rec | ID | Rec |
|:---|:---|:---|:---|
| D-01 | A | D-07 | A |
| D-02 | A | D-08 | A |
| D-03 | A | D-09 | A |
| D-04 | A | D-10 | A |
| D-05 | B | D-11 | B |
| D-06 | A | D-12 | A |

Copy final choices into §11 item 1 on the authority doc (or worksheet transfer table).

---

## Step 3 — Commit layout mock (30 min)

**Required path:** `assets/ui/world_preview/layout_mock_v1.png`

**Canvas:** **1920×1080** px, sRGB, flat PNG (no layers required in repo v1).

**Must show on mock:**

| Element | Notes |
|:---|:---|
| Header band | Operational index; sparse layer chips if D-05 A/B |
| Field index | Left stack per D-03 A (or your choice) |
| Central map | ≥62–65% width; breathing room per D-02 / D-11 |
| Annotations strip | Bottom status / queue |
| Optional field notes | Right margin if kept |
| Generator sheet | Ghost overlay if D-04 **A** (dimmed slide-over) |
| Registration marks | Magenta ticks — **no glow** (D-10) |
| Asymmetry | Visible 4–12 px offsets if D-09 A/C |

**Optional companion:** `assets/ui/world_preview/layout_mock_v1_notes.md` — offset table only (do not duplicate full spec).

**Annotation checklist (§11 item 3):** arrow labels for header height, index width, map margin %, corner overview size (if D-07 A).

---

## Step 4 — Complete §11 checklist (10 min)

On [`world_map_preview_layout_decision_v1.md`](world_map_preview_layout_decision_v1.md) §11:

| # | Item | Action |
|:---|:---|:---|
| 1 | D-01…D-12 recorded | Transfer from worksheet |
| 2 | Mock committed | `layout_mock_v1.png` in repo |
| 3 | 1080p annotation | On mock or sidecar notes |
| 4 | Motion §6 | Accept table or amend durations in sign-off notes |
| 5 | Magenta boundary | Confirm §1 palette |
| 6 | §9 flow | Walk Open → Generate → Enter sim |
| 7 | §3 REMOVE | Acknowledge current chrome retirement |

**Verdict line:** change **DRAFT** → **SIGNED** (or **CONDITIONAL** with blocker list).

Fill sign-off table (Designer row + date).

---

## Step 5 — Handoff to @coder

When §11 is **SIGNED**:

1. Add / unblocks slice **UI-WP-LAYOUT-001** in [`continuation_queue.json`](../../../tools/orchestrator/queues/continuation_queue.json).
2. Coder reads signed §5 + mock path; implements **D-01 shell only first** (≤3 files).

**Copy-paste for designer → coder chat:**

```
World Preview layout SIGNED — UI-WP-LAYOUT-001 unblocked
Read: prompts/guides/ui/world_map_preview_layout_decision_v1.md (§5 choices + §11)
Mock: assets/ui/world_preview/layout_mock_v1.png
First slice: D-01 shell only — window.rs / world_gen_ui.rs (max 3 files)
Do NOT: raster graph, GenerateWorldEvent, motion until WP-L2/L3
Runbook: prompts/guides/world_preview_runbook_v1.md (pipeline only)
```

---

## Until SIGNED — role boundaries

| Role | Allowed | Blocked |
|:---|:---|:---|
| **@designer** | Mocks, tokens, worksheet, motion storyboard | — |
| **@coder** | Raster/GPU bugs, viewport contract, witness JSON | Panel shells, asymmetry offsets, motion, workspace merge |
| **@planner** | Sequencing WP-L* after sign-off | Layout architecture assuming unsigned D-* |

---

## Quick links

| Doc | Purpose |
|:---|:---|
| [`world_preview_layout_decision_worksheet_v1.md`](world_preview_layout_decision_worksheet_v1.md) | One-page D-01…D-12 + §11 |
| [`world_preview_d01_shell_signoff_v1.md`](world_preview_d01_shell_signoff_v1.md) | **D-01** — shell sign-off (done) |
| [`world_preview_d02_map_dominance_signoff_v1.md`](world_preview_d02_map_dominance_signoff_v1.md) | **D-02** — map ≥65% sign-off; coder **optional** |
| [`world_preview_d_wp_track_signoff_v1.md`](world_preview_d_wp_track_signoff_v1.md) | **D-WP** — full track rollup D-01…D-12 |
| [`minimap_d_m1_signoff_v1.md`](../../../src/dev/minimap_d_m1_signoff_v1.md) | **D-MINIMAP-M1** — GPU minimap foundation |
| [`minimap_d_m2_signoff_v1.md`](../../../src/dev/minimap_d_m2_signoff_v1.md) | **D-MINIMAP-M2** — strategic overlays |
| [`world_preview_runbook_v1.md`](../world_preview_runbook_v1.md) | Coder pipeline lane (parallel) |
| [`ui_boundary_guide_v1.md`](../ui_boundary_guide_v1.md) | egui vs Bevy in WorldGen |
| [`tools/orchestrator/knowledge/ui_texture_assets.json`](../../../tools/orchestrator/knowledge/ui_texture_assets.json) | Future paper 9-slice paths |

---

## Document history

| Version | Date | Notes |
|:---|:---|:---|
| v1.1.0 | 2026-05-24 | SIGNED complete; Figma export spec for WP-L1 |
| v1.0.0 | 2026-05-24 | Designer workflow + mock contract + handoff gate |

---

## Appendix — Figma / export template spec (WP-L1, optional)

Use when replacing egui flat frames with paper textures. **Not required for UI-WP-LAYOUT-001.**

| Layer (bottom → top) | Export | Notes |
|:---|:---|:---|
| `00_void` | — | Fill `#000000` |
| `10_map_hero` | PNG slice or live viewport hole | 12% min margin (D-11 B) |
| `20_paper_index` | `@2x` PNG + alpha | Left 250px; offset +8px |
| `21_paper_notes` | `@2x` PNG + alpha | Right 180px; offset −6px |
| `30_header` | PNG 1920×64 | Offset +4px; registration ticks on own layer |
| `31_layer_strip` | PNG stretch | Map-top strip (D-05 B) |
| `40_annotations` | PNG 1920×72 | Torn top edge alpha |
| `50_registration` | SVG → PNG | `#D946EF` @ 60%, no effects |
| `60_hand_notes` | PNG stamps | Field notes graphite overlays |

**Export:** 1920×1080 `@1x` mock + `@2x` paper assets; sRGB; premultiplied alpha for torn edges.

**Coder import:** `assets/ui/world_preview/paper_*.png` + optional `ui_texture_assets.json` row (Phase WP-L1).