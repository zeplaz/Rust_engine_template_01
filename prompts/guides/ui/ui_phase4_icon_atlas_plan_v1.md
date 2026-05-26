# UI Phase 4 — icon atlas plan `v1` (PLAN-UI-P4-ATLAS-001)

| Field | Value |
|:---|:---|
| **Queue ID** | **PLAN-UI-P4-ATLAS-001** |
| **Version** | `1.0.1` |
| **Date** | 2026-05-25 |
| **Owner** | `@planner` |
| **Status** | **SIGNED** — P4.1 **CLOSED** · P4.2 **PARTIAL** · **UI-OH-P4-001** mapped |
| **UI-OH lane** | [`ui_oh_p4_001_plan_v1.md`](../../../src/dev/ui_oh_p4_001_plan_v1.md) |
| **Design brief** | [`ui_phase4_icon_atlas_brief_v1.md`](ui_phase4_icon_atlas_brief_v1.md) — grid, trace rules, JSON |
| **Petroleum UX** | [`petroleum_industry_ui_snippet_v1.md`](petroleum_industry_ui_snippet_v1.md) |
| **Phase 4 shell** | [`ui_phase4_handoff_plan_v1.md`](ui_phase4_handoff_plan_v1.md) · [`stages/ui_phase4_execution_plan_v1.md`](../../../src/dev/stages/ui_phase4_execution_plan_v1.md) |
| **Witness** | [`debug_runs/ui_shell_migration_live.json`](../../../debug_runs/ui_shell_migration_live.json) → `phase4` |

**No Rust.** Planner rollup for **build-rail icon atlas** + **petroleum tab affordance** + optional **vehicle row** consumers. Code spine landed 2026-05-23; this doc unblocks Phase 4.2 prioritization.

---

## Track map

| Track | ID | Owner | Status |
|:---|:---|:---|:---:|
| **P4.1** — loader, rail icons, witness | **UI-P4-ATLAS-CODE** | `@coder` | **CLOSED** |
| **P4.2a** — petroleum tab (`P5_BR`) | **P4-P5-01** | `@coder` | **CLOSED** |
| **P4.2b** — traced atlas PNG | **P4-ART-01** | `@designer` | **OPEN** (optional) |
| **P4.2c** — vehicle row consumers | **P4-VEH-01** | `@coder` | **OPEN** (deferred) |
| **P4.2d** — rail hover border parity | **P4-F03** | `@coder` | **OPEN** (optional) |
| **P4.2e** — Mi / Ec cells | **P4-MI-EC** | `@designer` | **DEFERRED** |

**Boundary:** Presentation only — icons reflect `ToolContext` / panel state; **no** `BuildStripState` authority changes ([`ui_boundary_guide_v1.md`](../ui_boundary_guide_v1.md)).

---

## Master gate chain

```text
UI-P2B-GATE (sim HUD Bevy shell)                 ☑
        │
        ▼
P4.1  IconAtlasPlugin + build rail RD…CV         ☑ 2026-05-23
        │
        ▼
P4-P5-01  Petroleum tab IconId::P5Br             ☑ 2026-05-25
        │
        ├─► P4-ART-01  traced PNG (optional)      ☐
        ├─► P4-VEH-01  TRUCK/URAL/BUS chips       ☐
        └─► IND panel  petroleum sim UI           ☐  (snippet — separate lane)
```

**Does not block:** World Preview LAYOUT-002/003, Stage 5 FULL_APP, minimap compositor.

---

## P4.1 — icon atlas spine (**CLOSED**)

**Authority:** [`ui_phase4_icon_atlas_brief_v1.md`](ui_phase4_icon_atlas_brief_v1.md) §0–§11.1.

### Deliverables (landed)

| Artifact | Path |
|:---|:---|
| Rust module | [`src/gui/hud/icon_atlas.rs`](../../../src/gui/hud/icon_atlas.rs) |
| RON manifest | [`assets/configs/ui/icon_atlas_phase4.icon_atlas.ron`](../../../assets/configs/ui/icon_atlas_phase4.icon_atlas.ron) |
| Atlas PNG | [`assets/textures/ui/icon_atlas_phase4_v1.png`](../../../assets/textures/ui/icon_atlas_phase4_v1.png) — **placeholder** |
| Knowledge | [`tools/orchestrator/knowledge/ui_texture_assets.json`](../../../tools/orchestrator/knowledge/ui_texture_assets.json) → `phase4_icon_atlas` |
| Rail host | [`in_game_hud.rs`](../../../src/gui/in_game_hud.rs) `BuildRailToolIcon` |
| Plugin | [`simulation_shell_phase2.rs`](../../../src/gui/hud/simulation_shell_phase2.rs) `IconAtlasPlugin` |

### Grid (authoritative)

**256×128** · **8×4** cells · **32×32** px · row 0 = **RD RL UT IN CV** (build rail).

**Critical QA:** **UT** = oil-cooled **transformer** — **never** generator trailer (`UT_MG` row 1 only). See brief §5.

### P4.1 witness

| Field | Expected (lib / sim refresh) |
|:---|:---|
| `phase4.icon_atlas_loaded` | `true` when atlas assets resolve |
| `phase4.rail_icons` | `["RD","RL","UT","IN","CV"]` |
| `phase4.atlas_texture` | `textures/ui/icon_atlas_phase4_v1.png` |
| `phase4.manifest_ron` | `configs/ui/icon_atlas_phase4.icon_atlas.ron` |

**Note:** On-disk `ui_shell_migration_live.json` may show `icon_atlas_loaded: false` until operator refresh after asset load — lib witness tests assert `true`.

### P4.1 regression

```powershell
cargo test -p proc_A_dine01 --lib icon_atlas
cargo test -p proc_A_dine01 --lib simulation_shell_phase2::tests::ui_p2a_001_live_witness_refresh
```

**Frozen constants (do not rename without proof migration):**

```rust
// icon_atlas.rs
ICON_ATLAS_TEXTURE_PATH = "textures/ui/icon_atlas_phase4_v1.png"
ICON_ATLAS_MANIFEST_PATH = "configs/ui/icon_atlas_phase4.icon_atlas.ron"
```

---

## P4.2a — petroleum tab (**CLOSED**)

**UX snippet:** [`petroleum_industry_ui_snippet_v1.md`](petroleum_industry_ui_snippet_v1.md) — full refinery panels **Pending**; tab affordance is Phase 4 scope only.

### P4-P5-01 — landed

| Item | Detail |
|:---|:---|
| Cell | **P5_BR** (3,0) — barrel silhouette |
| API | `IconAtlasUi::image_node_for_id(..., IconId::P5Br)` |
| Visibility | `petroleum_panel_tab_visible` — `ToolContext::Industry` + tray not collapsed |
| Systems | `sync_petroleum_panel_tab_system` |
| Components | `PetroleumPanelTabRoot`, `PetroleumPanelTabIcon` |
| Witness | `phase4.p5_br_tab_wired: true` |

### Acceptance — met

| # | Criterion |
|:---:|:---|
| P5-1 | Tab hidden when not Industry or tray collapsed |
| P5-2 | `P5Br` UV from manifest when visible |
| P5-3 | Tint follows `build_rail_icon_tint` idle/hover |
| P5-4 | Unit test `p4_p5_01_petroleum_panel_tab_visible_when_industry_and_tray_open` |

### Copy-paste — archive (P4-P5-01 done)

```
Lane: P4-P5-01 — petroleum tab IconId::P5Br
Read: ui_phase4_icon_atlas_plan_v1.md § P4.2a
      petroleum_industry_ui_snippet_v1.md (tab only)
Verify: cargo test -p proc_A_dine01 --lib simulation_shell_phase2
Witness: phase4.p5_br_tab_wired: true
```

### Petroleum panel body (out of atlas plan)

Refinery sliders, policy resources, warnings → **IND-*** / economy activation lanes when product schedules sim UI. **Do not** fold into atlas PNG work.

---

## P4.2b — traced atlas art (**OPEN**, optional)

**ID:** **P4-ART-01** · **Owner:** `@designer`

| Task | Output |
|:---|:---|
| Replace placeholder | Same path `icon_atlas_phase4_v1.png` |
| Keep grid | 256×128, cell indices unchanged |
| Trace rules | Brief §4–§7 — monochrome silhouettes |
| UT blind test | §5 — ≥4/5 “transformer” at 48px rail |

**Blocks:** nothing — placeholder is shippable.

### Copy-paste — P4-ART-01 (@designer)

```
Lane: P4-ART-01 — traced icon atlas PNG
Read: ui_phase4_icon_atlas_brief_v1.md §4–§7
Drop: assets/textures/ui/icon_atlas_phase4_v1.png (replace in place)
Keep: RON manifest cell map unchanged
Handoff: notify @coder for witness refresh + optional UT vs UT_MG hash test
```

### Copy-paste — coder after art drop

```
Lane: UI-P4-ATLAS-CODER-ART (post P4-ART-01)
Read: ui_phase4_icon_atlas_plan_v1.md § P4.2b
First: confirm atlas loads; run icon_atlas + witness tests
Do NOT: change IconId enum without manifest + JSON sync
Verify: phase4.icon_atlas_loaded true in refreshed ui_shell_migration_live.json
```

---

## P4.2c — vehicle row (**OPEN**)

**ID:** **P4-VEH-01** · Cells **TRUCK**, **URAL**, **BUS** (row 2)

| Consumer | When |
|:---|:---|
| Logistics inspector chips | `HudInfoTab::Logistics` |
| Convoy overlay | transport presentation lane |

**Pattern:** Reuse `image_node_for_id` — same as P5.

**Max files:** 3 — host panel + `icon_atlas.rs` (if helper only) + witness test.

---

## P4.2d / P4.2e — optional tails

| ID | Goal |
|:---|:---|
| **P4-F03** | Build-rail slot `BorderColor` → `accent_hot` on hover (ops-strip F-03 parity) |
| **P4-MI-EC** | Row 0 cols 5–6 for `Military` / `Ecology` when product requests |

---

## Interaction tokens (summary)

| State | Token | Application |
|:---|:---|:---|
| Idle | `fg_muted` @ ~72% | Icon tint |
| Hover | `accent_hot` | Icon tint (border optional P4-F03) |
| Selected | `accent_gold` + `bg_vellum` | Icon + slot chrome |

Full table: brief §8 · code `build_rail_icon_tint`.

---

## Forbidden

| Pattern | Reason |
|:---|:---|
| `ToolContext` selection from icon click alone | Authority in build strip / input routing |
| Literal source-art RGB on rail icons | Token tint only |
| **UT** cell from generator trailer PNG | §5 product misread |
| GPU minimap / `RepresentationResult` mutation | Phase 3 spine |
| egui build rail in sim (PLAY-01) | Construction catalog editor-only |

---

## Unified witness

| File | Keys |
|:---|:---|
| `ui_shell_migration_live.json` | `phase4.*`, shell chrome |
| Lib `ui_p2a_001_live_witness_refresh` | serializes `phase4` block |

```powershell
cargo test -p proc_A_dine01 --lib icon_atlas simulation_shell_phase2::tests::ui_p2a_001_live_witness_refresh stage5
# Optional operator:
cargo run -p proc_A_dine01 -- --test frame
```

---

## Sign-off

| Role | Date | Status |
|:---|:---|:---|
| Planner | 2026-05-25 | **SIGNED** — PLAN-UI-P4-ATLAS-001 |
| Coder P4.1 | 2026-05-23 | **CLOSED** |
| Coder P4-P5-01 | 2026-05-25 | **CLOSED** |
| Designer P4-ART-01 | — | **OPEN** (optional) |

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.1 | 2026-05-25 | **UI-OH-P4-001** closure lane — [`ui_oh_p4_001_plan_v1.md`](../../../src/dev/ui_oh_p4_001_plan_v1.md) |
| v1.0.0 | 2026-05-25 | Icons/petroleum rollup; P4.1 + P4-P5-01 closed; P4.2 tails open |
