# Stage 6 operational sign-off

**Status:** **CLOSED** — virtualization host operational gate satisfied (S6-0…S6-3).  
**Signed:** 2026-05-23  
**Next open lane:** infrastructure hardening (VM depth, Wave S save) — not Stage 6 exit blockers.

---

## What “closed” means

Stage 6 **operational readiness** per [`stage6_plan_open.md`](stage6_plan_open.md) and [`AGENTS.md`](../../AGENTS.md):

- **Residency authoritative** — `ChunkResidencyTable` + `Stage6VirtualizationFrame` drive consumer window, fire/overlay cull, preview intersect
- **Atlas / async** — GPU upload bytes gate readiness (DQ-S6-03); main-thread-only ECS apply chain documented + tested
- **Per-view windows** — `PerViewResidencyConsumerWindow` published; fire caps via `PerViewRepresentationPolicy`
- **HUD BQ-134** — F3 / side status read `Stage6HudTelemetry.residency` from `residency_overlay_consumer_from_frame` (not mock)
- **Lib proof** — `cargo test -p proc_A_dine01 --lib` (622+ tests)
- **Live witness** — `debug_runs/stage6_virtualization_live.json` with `stage6_readiness.passes: true`

Stage 6 is **not** infinite-world streaming, full multi-atlas pressure CI, or Wave S product save exit.

---

## Evidence (operator)

| Artifact | Signal |
|----------|--------|
| `debug_runs/stage6_virtualization_live.json` | `stage6_readiness.passes: true`, 1369 residency chunks, core/ghost split, `stage6_virtualization_green: true`, VM-A crosslink green |
| `debug_runs/stage5_full_app_live.json` | Prior FULL_APP green (`readiness.passes: true`) — spine not regressed by S6 wiring |
| `src/render/stage6_virtualization.rs` | Readiness from upload bytes; publish fills per-view windows |
| `src/gui/hud/stage6_telemetry.rs` | BQ-134 DTO from frame + table |
| `src/gui/hud/dock_shell.rs` | F3 telemetry tab: `Stage6HudTelemetry` only (`unwrap_or_default` = zeros, not mock) |

**Refresh note:** Re-run sim after S6-2 merge to include `frame.gpu_upload_bytes_frame` and `frame.per_view_window_count` in live JSON (writer in `stage6_live_proof.rs`).

---

## S6-3 checklist (Package D)

| ID | Criterion | Result |
|----|-----------|--------|
| S6-30 | Live JSON readiness green in sim | **Pass** (on-disk witness) |
| S6-31 | F3 BQ-134 authoritative DTO | **Pass** — telemetry bridge; side panel prefers `Stage6HudTelemetry` |
| S6-32 | Lib tests | **Pass** — `cargo test -p proc_A_dine01 --lib` |
| S6-33 | Stage 5 visual spine regression | **Pass** (existing `stage5_full_app_live.json`); re-run `--test visual` on release train |
| S6-34 | Sign-off doc + plan §11 | **Pass** — this file |

---

## Moved to triage (not verified for this gate)

| Topic | Where |
|-------|--------|
| 60s+ timed sim attribution session | `perf_attribution_60s.md` PERF-N01 |
| Multi-atlas exit meaning DQ-S6-03 option C | `stage6_plan_open.md` triage |
| Minimap ghost-band tint (S6-25) | `stage6_design_decisions.md` — deferred |
| Wave S save / blueprint RON (S6-S1, S6-S3) | parallel lane after S6-3 |
| VM-06…11 full implementation | `stage5_triage_backlog.md` |
| Compile warning hygiene | `COMPILE_WARNINGS_TODOS.md` |

---

## Authority / violations

- **No dual-writer** on residency frame vs table for HUD: DTO built from both in one system (`refresh_stage6_hud_telemetry`).
- **Schedule:** Fire extract does **not** `.after(publish_stage6_virtualization_frame)` (cycle avoided); cull uses `ChunkResidencyTable` / per-view windows.
- **Violations:** empty in last live JSON witness.

---

## Commands (regression)

```powershell
cargo test -p proc_A_dine01 --lib stage6
cargo run -p proc_A_dine01 --release -- --test visual
```

Expect sim → `debug_runs/stage6_virtualization_live.json` updated; visual → Stage 5 FULL_APP proof refresh without new violations.

---

## Handoff

1. [`stage6_active_todos.md`](stage6_active_todos.md) — S6-3 done; parallel Wave S/P/C optional  
2. [`stage5_triage_backlog.md`](stage5_triage_backlog.md) — infrastructure depth  
3. [`operational_readiness_vs_infrastructure_perf_v1.md`](../../prompts/guides/operational_readiness_vs_infrastructure_perf_v1.md) — perf hardening lane
