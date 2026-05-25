# UI Phase 2 sprint queue (index)

**Status:** **CLOSED** (2026-05-24) — see [`ui_overhaul_plan.md`](ui_overhaul_plan.md) v1.1.0

**Master plan:** [`ui_overhaul_plan.md`](ui_overhaul_plan.md) · **UI guides:** [`prompts/guides/ui/README.md`](../prompts/guides/ui/README.md)

**Archive queue:** [`ui_phase2_coder_queue_v1.md`](../prompts/guides/ui/ui_phase2_coder_queue_v1.md) · **Active:** [`ui_phase3_coder_queue_v1.md`](../prompts/guides/ui/ui_phase3_coder_queue_v1.md)

---

## Final sprint status

| Sprint | Status | Notes |
|:---:|:---|:---|
| **1 — 2A-Tail** | **Done** | F-01–F-11 · §1.6 witness clicks green |
| **2 — 2B-Build** | **Done** | Build rail → `BuildStripState` |
| **3 — 2B-Dedupe** | **Done** | egui gates · `phase2b_closed` |
| **4 — Sign-off** | **Done** | Designer **SIGNED** v2.1.1 · `--test visual` green |
| **2C — Left chrome** | **DEFERRED** | `@designer` → **2C-A/B/C/D** · [`ui_phase2_coder_queue_v1.md`](../prompts/guides/ui/ui_phase2_coder_queue_v1.md) § Sprint 2C |

**Witness:** `debug_runs/ui_shell_migration_live.json` — profile `UI_SHELL_MIGRATION_2B` · `phase2a_closed` + `phase2b_closed` true.

**Tail (optional):** `ops_zone_hover_token` · `build_rail_authoritative` — interaction replay slices in continuation queue.

---

## Phase 3 handoff

| Lane | Entry |
|:---|:---|
| **UX-E01 M1/M1.5** | **Done** — `minimap_compositor_live.json` |
| **UX-E01 M2** | [`ui_phase3_coder_queue_v1.md`](../prompts/guides/ui/ui_phase3_coder_queue_v1.md) §3.4 |
| **IND-E01** | [`post_stage6_active_todos.md`](post_stage6_active_todos.md) |

```powershell
cargo test -p proc_A_dine01 --lib minimap_compositor stage5
cargo run -p proc_A_dine01 --release -- --test visual
```
