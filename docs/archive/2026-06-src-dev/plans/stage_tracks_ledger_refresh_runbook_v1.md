# Stage tracks — ledger refresh runbook `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **PLAN-LEDGER-REFRESH** (cycles **001**, **002**, …) |
| **Version** | `1.2.0` |
| **Date** | 2026-05-25 |
| **Owner** | `@orchestrator` · any lane owner at cycle end |
| **When** | **After each primary track cycle** (or any merge touching witnesses) — output: [`stage_open_todos_v1.md`](stage_open_todos_v1.md) |

**Goal:** [`stage_tracks_signoff_ledger_v1.md`](stage_tracks_signoff_ledger_v1.md), workboards, and [`continuation_queue.json`](../../tools/orchestrator/queues/continuation_queue.json) stay aligned with `debug_runs/*.json`.

---

## Checklist (15 min)

| # | Action | Output |
|:---:|:---|:---|
| 1 | Run spine tests | `cargo test -p proc_A_dine01 --lib stage5` |
| 2 | Refresh visual witness if render/UI touched | `cargo run -p proc_A_dine01 --release -- --test visual` |
| 3 | Read witness JSON | Compare to ledger matrix |
| 4 | Update ledger | DONE / OPEN / STALE / SIGNED rows |
| 5 | Update [`stage_coder_workboard_v1.md`](stage_coder_workboard_v1.md) | Remove completed slices |
| 6 | Update [`stage_designer_workboard_v1.md`](stage_designer_workboard_v1.md) | Mark SIGNED items |
| 7 | Sync [`continuation_queue.json`](../../tools/orchestrator/queues/continuation_queue.json) | `status: done` for landed slices |
| 8 | Bump ledger **Version** + changelog row | audit date |
| 9 | `cargo orchestrate --skip-cargo` | reports current |

---

## STALE handling

If witness contradicts code (e.g. `phase2b_closed: false` but `egui_pass_count: 0`):

| Step | Action |
|:---|:---|
| 1 | Label ledger row **STALE** — not **regressed** |
| 2 | Queue **UI-SHELL-REFRESH-001** or operator replay |
| 3 | Do **not** mass-reopen closed gates without proof |

---

## Per-track witness map

| Track | Primary JSON |
|:---|:---|
| S7-PLAY | `industrial_activation_live.json`, `construction_stage_live.json` |
| VFX-P2 / FX-WATER | `stage5_full_app_live.json` |
| UI-P2 | `ui_shell_migration_live.json` |
| UI-P3 | `minimap_compositor_live.json` |
| UI-P4 / WP | manual + `wave_p_live.json` |
| INFRA | `infrastructure_view_isolation_live.json` |
| WAVE-C | `wave_c_live.json`, `stage6_virtualization_live.json` |

---

## Changelog entry template

```markdown
| vX.Y.Z | YYYY-MM-DD | Audit: <tracks touched> — <witness files refreshed> |
```

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.2.0 | 2026-05-25 | PLAN-LEDGER-REFRESH-002 — witness↔done matrix; queue restore if empty |
| v1.1.0 | 2026-05-25 | PLAN-LEDGER-REFRESH-001 urgent — restore empty continuation_queue |
| v1.0.0 | 2026-05-24 | PLAN-LEDGER-REFRESH |
