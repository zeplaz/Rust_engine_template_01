# UI overhaul — Phase 4 closure plan `v1` (UI-OH-P4-001)

| Field | Value |
|:---|:---|
| **Lane ID** | **UI-OH-P4-001** |
| **Planner queue** | **PLAN-UI-P4-ATLAS-001** |
| **Version** | `1.0.0` |
| **Date** | 2026-05-25 |
| **Owner** | `@planner` |
| **Status** | **SIGNED** — P4.1 + P4-P5-01 **CLOSED** · tails **OPEN** |
| **Atlas plan (authoritative)** | [`ui_phase4_icon_atlas_plan_v1.md`](../prompts/guides/ui/ui_phase4_icon_atlas_plan_v1.md) |
| **Master lane** | [`ui_overhaul_plan.md`](ui_overhaul_plan.md) |
| **Phase 2+3 closure** | [`ui_overhaul_phase23_closure_plan_v1.md`](ui_overhaul_phase23_closure_plan_v1.md) |
| **Live rollup** | [`witness_status_live_v1.md`](witness_status_live_v1.md) |
| **Witness** | `debug_runs/ui_shell_migration_live.json` → `phase4` |

**No Rust in this deliverable.** Maps **PLAN-UI-P4-ATLAS-001** into the UI-OH lane ID **UI-OH-P4-001**.

---

## Executive summary

| Track | Verdict |
|:---|:---|
| **P4.1** — loader, rail RD…CV, manifest | **PASS** |
| **P4-P5-01** — petroleum tab `P5Br` | **PASS** |
| **P4-ART-01** — traced PNG | **DONE** — [`ui_oh_p4_art_signoff_record_v1.md`](ui_oh_p4_art_signoff_record_v1.md) · [`ui_icon_atlas_sheet_v1.md`](ui_icon_atlas_sheet_v1.md) |
| **P4-VEH-01** / **P4-F03** / **P4-MI-EC** | **OPEN** / deferred |
| **UI-OH-P4-001 rollup** | **PASS (qualified)** |

**Qualified:** If `phase4.icon_atlas_loaded` is `false` on disk while lib tests are green, run witness refresh below — **STALE JSON only**; does **not** reopen **UI-P4-ATLAS-CODE** or **P4-ART-01** art.

---

## Gate chain (PLAN-UI-P4-ATLAS-001)

```text
UI-P2B-GATE (sim Bevy shell)              ☑
        │
        ▼
P4.1  IconAtlasPlugin + rail RD…CV        ☑ UI-P4-ATLAS-CODE
        │
        ▼
P4-P5-01  Petroleum tab IconId::P5Br       ☑
        │
        ├─► P4-ART-01  traced PNG           ☑ DONE
        ├─► P4-VEH-01  vehicle row          ☐ deferred
        └─► P4-F03     hover border           ☐ optional
```

---

## PASS gate — P4.1 spine

| # | Criterion | Witness path | Required | 2026-05-25 |
|:---:|:---|:---|:---:|:---:|
| P4-1 | Rail icon set | `phase4.rail_icons` | `["RD","RL","UT","IN","CV"]` | ☑ |
| P4-2 | Atlas texture path | `phase4.atlas_texture` | `textures/ui/icon_atlas_phase4_v1.png` | ☑ |
| P4-3 | Manifest path | `phase4.manifest_ron` | `configs/ui/icon_atlas_phase4.icon_atlas.ron` | ☑ |
| P4-4 | Atlas loaded (lib) | `phase4.icon_atlas_loaded` | `true` in lib witness test | ☑ test |
| P4-4b | Atlas loaded (disk) | `phase4.icon_atlas_loaded` | `true` after refresh | ☑ (`ui_shell_migration_live.json`) |
| P4-5 | Build rail uses atlas | code `BuildRailToolIcon` + `tool_context_uses_icon_atlas` | wired | ☑ |

**Lib anchor:**

```powershell
cargo test -p proc_A_dine01 --lib icon_atlas
cargo test -p proc_A_dine01 --lib stage5_ui_shell_migration_phase4_witness_fields
```

---

## PASS gate — P4-P5-01 (petroleum tab)

| # | Criterion | Witness path | Required | 2026-05-25 |
|:---:|:---|:---|:---:|:---:|
| P5-1 | Tab wired | `phase4.p5_br_tab_wired` | `true` | ☑ |
| P5-2 | Industry visibility | `sync_petroleum_panel_tab_system` | hidden unless Industry + tray open | ☑ lib |
| P5-3 | Cell UV | `IconId::P5Br` @ (3,0) | manifest match | ☑ lib |

**Out of scope:** refinery panel body → **IND-*** / economy lanes ([`petroleum_industry_ui_snippet_v1.md`](../prompts/guides/ui/petroleum_industry_ui_snippet_v1.md)).

---

## UI-OH-P4-001 rollup (target witness block)

**Coder slice (optional witness writer):** add `ui_oh_p4_001` to shell JSON when refreshing proof.

| Path | Green when |
|:---|:---|
| `ui_oh_p4_001.gate` | `"UI-OH-P4-001"` |
| `ui_oh_p4_001.green` | P4-1…P4-3 + P5-1 **true** (P4-4b STALE allowed) |
| `ui_oh_p4_001.p4_1_green` | rail + paths |
| `ui_oh_p4_001.p5_br_green` | `p5_br_tab_wired` |
| `ui_oh_p4_001.icon_atlas_loaded` | mirrors `phase4.icon_atlas_loaded` |

**Formula (planner):**

```text
ui_oh_p4_001.green :=
  phase4.rail_icons == ["RD","RL","UT","IN","CV"]
  AND phase4.p5_br_tab_wired
  AND phase4.atlas_texture / manifest_ron paths match constants
  AND (icon_atlas_loaded OR lib test stage5_ui_shell_migration_phase4_witness_fields green)
```

Until `ui_oh_p4_001` block exists on disk, use **phase4** fields + lib tests as authority.

---

## Witness field table

| Phase | File | Field | Role |
|:---|:---|:---|:---|
| P4.1 | `ui_shell_migration_live.json` | `phase4.rail_icons` | Build rail row 0 |
| P4.1 | `ui_shell_migration_live.json` | `phase4.icon_atlas_loaded` | Loader resolved |
| P4.1 | `ui_shell_migration_live.json` | `phase4.atlas_texture` | PNG path |
| P4.1 | `ui_shell_migration_live.json` | `phase4.manifest_ron` | RON path |
| P4-P5 | `ui_shell_migration_live.json` | `phase4.p5_br_tab_wired` | Petroleum tab |
| OH rollup | `ui_shell_migration_live.json` | `ui_oh_p4_001.green` | **Future** — optional writer |
| Spec | [`ui_shell_witness_spec_v1.md`](ui_shell_witness_spec_v1.md) | `phase4` block | Field catalog |

---

## Open tails (not UI-OH-P4-001 blockers)

| ID | Owner | plan_doc section |
|:---|:---|:---|
| **P4-ART-01** | @designer | **DONE** — traced atlas + sheet record |
| **P4-VEH-01** | @coder | § P4.2c |
| **P4-F03** | @coder | § P4.2d |
| **P4-MI-EC** | @designer | § P4.2e — **DEFERRED** |

---

## Forbidden (from atlas plan)

| Pattern | Reason |
|:---|:---|
| `ToolContext` from icon click alone | Build strip authority |
| **UT** cell from generator trailer art | Brief §5 |
| egui build rail in sim | PLAY-01 / 2B gate |
| GPU minimap mutation | Phase 3 spine |

---

## Copy-paste — witness refresh (@coder / operator)

```
Lane: UI-OH-P4-001 — phase4 witness refresh
Read: prompts/guides/ui/ui_phase4_icon_atlas_plan_v1.md
      src/dev/ui_oh_p4_001_plan_v1.md
Verify: cargo test -p proc_A_dine01 --lib icon_atlas stage5_ui_shell_migration_phase4
Witness: phase4.p5_br_tab_wired true; icon_atlas_loaded true after refresh
Optional: add ui_oh_p4_001 block to ui_shell_migration_live.json
```

---

## Copy-paste — P4-ART-01 (@designer)

See [`ui_phase4_icon_atlas_plan_v1.md`](../prompts/guides/ui/ui_phase4_icon_atlas_plan_v1.md) § P4.2b — replace PNG in place; keep RON grid.

---

## Sign-off

| Role | Date | Status |
|:---|:---|:---|
| Planner | 2026-05-25 | **SIGNED** — UI-OH-P4-001 / PLAN-UI-P4-ATLAS-001 |
| Coder P4.1 | 2026-05-23 | **CLOSED** |
| Coder P4-P5-01 | 2026-05-25 | **CLOSED** |

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-25 | UI-OH lane mapping for PLAN-UI-P4-ATLAS-001 |
