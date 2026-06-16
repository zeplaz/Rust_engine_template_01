# PLAN-G-PLAY-CLOSE-001 — operator closure checklist `v1`

| Field | Value |
|:---|:---|
| **Slice ID** | **PLAN-G-PLAY-CLOSE-001** |
| **Gate** | **G-PLAY-01** |
| **Runbook** | [`play_scenario_acceptance_runbook_v1.md`](play_scenario_acceptance_runbook_v1.md) |
| **Audit** | [`planner_status_audit_v19.md`](planner_status_audit_v19.md) |
| **Status** | **SIGNED** — awaiting **Operator EXECUTED** row |
| **Date** | 2026-06-03 |

**Rule:** Lib witness green ≠ G-PLAY-01 close. Only operator sign-off below closes the gate.

---

## Preconditions (verify before clock starts)

| ☐ | Check |
|:---:|:---|
| ☐ | `cargo build -p proc_A_dine01 --release` succeeds |
| ☐ | Launch **without** `--test visual`, `test_harness`, `RUST_ENGINE_STAGE7_PLAY_SEED` |
| ☐ | Audit v19: **G-CONTAIN-01** and **G-STAB-01** **CLOSED** (no re-open) |

---

## Operator session (§1–8 from runbook)

| ☐ | # | Pass | Fail note |
|:---:|:---:|:---:|:---|
| ☐ | 1 | Simulation entered; PLAY-01 chrome | |
| ☐ | 2 | No egui Construction window; build rail only | |
| ☐ | 3 | Mine placed near Portland | |
| ☐ | 4 | Kiln + mixer placed | |
| ☐ | 5 | Site progress toward Operational visible | |
| ☐ | 6 | Logistics on minimap (`logistics_rows > 0` if witness refreshed) | |
| ☐ | 7 | Pause → Resume OK | |
| ☐ | 8 | 10 min pan/zoom without soft-lock / viewport blink | |

**Session time:** _____ min (target ≥ 10 min play after sim enter)

---

## Stop rules (do not sign if any)

- Panic / unwrap dialog
- Requires harness env seed to progress
- Construction commit invisible on map
- Pause menu broken

---

## Optional witness refresh (after session)

```powershell
cargo test -p proc_A_dine01 --lib play_scenario stage7_play construction logistics
```

| Witness | Field | Expected |
|:---|:---|:---|
| `play_scenario_live.json` | `green` | `true` |
| `stage7_play_live.json` | `ind_e02_green` | `true` |
| `minimap_compositor_live.json` | `logistics_rows` | `> 0` |

---

## Sign-off (closes G-PLAY-01)

| Role | Verdict | Date | Signature / initials |
|:---|:---|:---|:---|
| Operator | ☐ **EXECUTED** | | |
| `@planner` | **SIGNED** (checklist) | 2026-06-03 | PLAN-G-PLAY-CLOSE-001 |
| `@designer` | **PASS (qualified)** | 2026-05-28 | runbook script |

**On operator EXECUTED:** update [`play_scenario_acceptance_runbook_v1.md`](play_scenario_acceptance_runbook_v1.md) § Sign-off + audit v19 gate rollup **G-PLAY-01 → CLOSED**.

---

## Orchestrator paste

```text
G-PLAY-01 remains OPEN until operator fills plan_g_play_close_001_checklist_v1.md.
Runbook: play_scenario_acceptance_runbook_v1.md §1–8.
No coder slice — operator task only.
```
