# PLAN-UI-2C-001 — left command table / mode rail `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **PLAN-UI-2C-001** |
| **Version** | `1.0.0` |
| **Date** | 2026-05-25 |
| **Owner** | `@planner` |
| **Status** | **SIGNED** — **2C-B** **CLOSED** (2026-05-24) |
| **Designer sign-off** | [`ui_phase2_designer_signoff_v1.md`](../prompts/guides/ui/ui_phase2_designer_signoff_v1.md) v2.2.0 |
| **Mock authority** | [`ui_phase0_panel_mocks_v1.md`](../prompts/guides/ui/ui_phase0_panel_mocks_v1.md) § P4 |
| **Witness** | `debug_runs/ui_shell_migration_live.json` → `phase2c` |
| **Closure** | [`ui_overhaul_phase23_closure_plan_v1.md`](ui_overhaul_phase23_closure_plan_v1.md) |

**No Rust in this deliverable.** Canonical planner record for Phase **2C** left chrome.

---

## Executive summary

| Option | Verdict |
|:---|:---|
| **2C-A** merge columns | Not chosen |
| **2C-B** dual column **48 + 52** (+ gap) | **SIGNED** — canonical |
| **2C-C** zero map occlusion | Not chosen |
| **2C-D** deferred rail | Not chosen |

**Product:** Left **context rail** (mode / command table) + **build rail** as **absolute overlay**; map hole stays full-width (documented occlusion).

---

## Layout contract (2C-B)

| Element | Width | Bevy root | Role |
|:---|:---:|:---|:---|
| **Left context rail** | **48px** | `LeftContextRail` | Mode / command table |
| **Build rail** | **52px** | `BuildRailRoot` | Tool + build icons |
| **Gap** | documented in mock | between columns | collapse target |
| **Map hole** | remainder | underlay | full-width terrain |

**Witness fields:**

| Path | Expected |
|:---|:---|
| `phase2c.layout_option` | `"2C-B"` |
| `phase2c.phase2c_closed` | `true` |
| `phase2c.left_context_rail_px` | `48` (or mock equivalent) |
| `phase2c.build_rail_px` | `52` |

---

## PASS gate

| # | Criterion | Witness | 2026-05-25 |
|:---:|:---|:---|:---:|
| 2C-1 | Layout signed | `phase2c.phase2c_closed` | ☑ |
| 2C-2 | Option id | `phase2c.layout_option` | **2C-B** |
| 2C-3 | P4 designer PASS | sign-off v2.2 § P4 | ☑ |
| 2C-4 | Phase 2+3 closure | `ui_overhaul_phase23_closure_plan_v1.md` | ☑ |

**Lib anchor:**

```powershell
cargo test -p proc_A_dine01 --lib simulation_shell_phase2 -- --test-threads=1
```

---

## Mode rail ergonomics (forward, non-blocker)

| ID | Notes | Priority |
|:---|:---|:---|
| **P4-VEH-01** | Vehicle row on build rail | deferred |
| **P4-F03** | Hover border polish | optional |
| Runbook | [`ui_operational_direction_runbook_v1.md`](../prompts/guides/ui/ui_operational_direction_runbook_v1.md) | reference |

**Do not** reopen **2C-B** without designer amendment to mock § P4.

---

## Forbidden

| Wrong | Correct |
|:---|:---|
| Merge 48+52 into single 48 without mock update | **2C-A** requires new sign-off |
| egui left rail in Simulation | Bevy `LeftContextRail` / `BuildRailRoot` only |
| Re-layout during UI-OH closure | maintain witness only |

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-25 | **PLAN-UI-2C-001** rollup for closed **2C-B** |
