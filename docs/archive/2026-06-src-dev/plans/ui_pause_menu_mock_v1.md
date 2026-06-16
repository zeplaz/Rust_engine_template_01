# Wave 3 pause menu mock — `DESIGN-W3-P5-UX-001` `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **DESIGN-W3-P5-UX-001** |
| **Track** | Wave 3 / UI Phase 5 pause UX |
| **Version** | `1.0.0` |
| **Date** | 2026-05-25 |
| **Owner** | `@designer` |
| **Status** | **SIGNED — PASS** |
| **Implementation path** | `src/gui/pause_menu_bevy.rs` |
| **Plan reference** | `docs/archive/2026-06-prompts-guides/ui_phases/guides/ui/ui_phase5_pause_menu_plan_v1.md` |
| **Witness JSON** | `debug_runs/ui_shell_migration_live.json` (`ui_w3_p5_001.green`, `ui_p5_pause_001_green`) |

---

## Deliverable intent

Wave 3 pause mock closes the UX polish and Bevy migration expectations for the in-simulation pause shell.

Verdict: `SIGNED — PASS`. Pause menu UX is aligned to shell language and witness flags are green.

---

## UX contract

| Item | Decision |
|:---|:---|
| Title and hierarchy | Centered card with explicit paused state copy |
| Primary action | Resume is visually emphasized |
| Destructive actions | Routed through confirm flow |
| Input model | Escape collapse/resume semantics preserved |
| Runtime policy | No egui pause overlay in Simulation |

---

## Acceptance checklist

| # | Item | Result |
|:---:|:---|:---:|
| 1 | Bevy pause menu path is active | PASS |
| 2 | `ui_w3_p5_001.green` is true | PASS |
| 3 | `ui_p5_pause_001_green` is true | PASS |
| 4 | `phase5.pause_menu_bevy` witness field is true | PASS |
| 5 | `egui_pass_count_in_sim == 0` policy remains satisfied | PASS |

---

## Verification commands

```powershell
cargo test -p proc_A_dine01 --lib ui_p5_pause_001
cargo test -p proc_A_dine01 --lib simulation_shell_phase2::tests::ui_w3_p5_001_live_witness_refresh
```

---

## Notes

- Save/load spine remains a separate deferred lane and is intentionally not reopened by this deliverable.
- This record is for design/UX closure and handoff clarity; behavior contract remains in existing coder lanes.

---

## History

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-25 | Initial Wave 3 pause menu mock deliverable record |
