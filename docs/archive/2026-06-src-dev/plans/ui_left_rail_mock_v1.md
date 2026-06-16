# Wave 3 left rail mock — `DESIGN-W3-2C-MOCK-001` `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **DESIGN-W3-2C-MOCK-001** |
| **Track** | Wave 3 / UI 2C left rail |
| **Version** | `1.0.0` |
| **Date** | 2026-05-25 |
| **Owner** | `@designer` |
| **Status** | **SIGNED — PASS** |
| **Canonical layout option** | `2C-B` |
| **Mock source** | `docs/archive/2026-06-prompts-guides/ui_phases/guides/ui/ui_phase0_panel_mocks_v1.md` (P4 amended) |
| **Witness JSON** | `debug_runs/ui_shell_migration_live.json` (`ui_w3_2c_001.green`) |

---

## Deliverable intent

This document records the Wave 3 left rail mock closure for 2C. The accepted design is `2C-B`: dual-column collapsed chrome with context rail and build rail.

Verdict: `SIGNED — PASS`. The design contract is in place and witness predicates are green.

---

## Layout contract

| Element | Value |
|:---|:---|
| Context rail width | `48px` |
| Build rail width | `52px` |
| Collapsed left chrome aggregate | `106px` (with gap) |
| Behavioral contract | Left rail remains Bevy-authoritative in Simulation |

---

## Acceptance checklist

| # | Item | Result |
|:---:|:---|:---:|
| 1 | 2C option explicitly locked to `2C-B` | PASS |
| 2 | Context rail and build rail widths match spec | PASS |
| 3 | No reopen of deprecated single-column expectation | PASS |
| 4 | `ui_w3_2c_001.green` present and true | PASS |
| 5 | Phase 2 shell closure remains consistent (`phase2b_closed`) | PASS |

---

## Verification commands

```powershell
cargo test -p proc_A_dine01 --lib simulation_shell_phase2::tests::ui_w3_2c_001_live_witness_refresh
```

---

## Notes for planner/coder

- This is a mock/signoff deliverable only; no additional layout migration is required for closure.
- If rail widths change, create a new explicit layout decision row instead of mutating this record.

---

## History

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-25 | Initial Wave 3 2C left rail mock deliverable record |
