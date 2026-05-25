# Stage 5 close checklist (operational gate only)

**Rule:** Only items in **§A — Mission-critical gate** block Stage 5 closure. Everything in [`stage5_triage_backlog.md`](stage5_triage_backlog.md) is **explicitly out of scope** for this gate (future stages / dedicated workers).

**Authority:** [`prompts/guides/stage5_convergence_directive_v1.md`](../../prompts/guides/stage5_convergence_directive_v1.md) §9–§14, [`operational_readiness_vs_infrastructure_perf_v1.md`](../../prompts/guides/operational_readiness_vs_infrastructure_perf_v1.md).

**Live boards (code):** `STAGE5_TODOS` (13 spine rows) + `STAGE5_FINISH_TODOS` (8 UX rows) in `src/dev/stage5_live_todos.rs`, `src/dev/stage5_finish_todos.rs`.

---

## A — Mission-critical gate (must be green)

Work **top to bottom**. Do not start Tier B until A is green in the **running app**.

| # | Gate | Proof | Command / artifact |
|---|------|-------|-------------------|
| A1 | Lib spine tests | `stage5` + representation tests pass | `cargo test -p proc_A_dine01 --lib stage5` ✅ 2026-05-23 |
| A2 | `stage5_readiness_passes` | All flags true, `violations` empty | `AppStage5ReadinessReport` in app |
| A3 | FULL_APP profile active | `Stage5ReadinessProfile::FULL_APP` during sim / visual | Engine launch / sim state |
| A4 | Visual live proof JSON | `readiness.passes=true`, boards all Done | `cargo run -p proc_A_dine01 -- --test visual` → `debug_runs/stage5_full_app_live.json` ✅ 2026-05-23 (refreshed) |
| A5 | Spine todo board | 13× `TODO-01`…`TODO-13` → Done (predicates, not manual) | JSON `readiness.live_todo_board.all_done` |
| A6 | Finish todo board | 8× `FINISH-UX-*` → Done; UX-06 streak ≥ 120 | JSON `readiness.live_finish_todo_board` + `finish_ux06_streak` |
| A7 | Mandatory closures A–F | See directive §13 | JSON flags: `vt4_ok`, `vt5_ok`, `single_fire_extract`, `gpu_field_authoritative`, `overlay_from_shared_buffers_only`, `particle_lod_scales`, `phase_f_*`, `preview_render_target_active`, `projection_domains=3` |
| A8 | Render anomalies clean | All `render_anomalies` false in live JSON | `stage5_full_app_live.json` |
| A9 | No authority regression | `duplicate_visual_scan_count == 0` | Same JSON + grep audit per cycle |

**Root spine order (when reopening failures):** `TODO-01` → `TODO-04` → `TODO-06` → then GPU/fire/preview/LOD (`STAGE5_ROOT_GATE_SEQUENCE`).

---

## B — Verification pass (sign-off, not new features)

Run once per “Stage 5 CLOSED” claim:

- [x] **B1** Refresh `debug_runs/stage5_full_app_live.json` (visual harness).  
  `.\tools\orchestrator\scripts\visual_full_app.ps1` or `cargo run -p proc_A_dine01 -- --test visual` ✅ 2026-05-23 (`stage5_closure`, `_agent_meta.written_at_epoch_secs` fresh)
- [x] **B2** Refresh `debug_runs/agent_debug_index.json` (auto on proof write).
- [x] **B3** `cargo rustc -p proc_A_dine01 --lib -- -D warnings` (package-only).
- [x] **B4** `cargo run --manifest-path tools/orchestrator/Cargo.toml -- --skip-clippy --skip-test` → 0 issues.
- [x] **B5** Update [`prompts/guides/base_visual_dev01_plan_status.md`](../../prompts/guides/base_visual_dev01_plan_status.md) — Stage 5 row = **CLOSED (operational)**.
- [x] **B6** Record date + commit in this file §E sign-off table.

---

## C — Explicitly NOT in Stage 5 gate

Do **not** block closure on:

| Topic | Where tracked |
|-------|----------------|
| VM-06…VM-11 full per-view isolation | [`stage5_triage_backlog.md`](stage5_triage_backlog.md) → Stage 5.5 / infra |
| GPU tile debug gizmo removal / WGSL polish | Triage → render hardening |
| Full fire streaming / neighbor wake / sleep budget | Triage → fire sim stage |
| `MapCameraDesired` invert-only (VM-09b v2) | Triage → view runtime |
| Construction toolbox / roads / rail | `construction_stage_live.json` — **parallel stage** |
| Shell perf (200ms+ wall, logging cost) | [`operational_readiness_vs_infrastructure_perf_v1.md`](../../prompts/guides/operational_readiness_vs_infrastructure_perf_v1.md) §2 |
| Stage 6 virtualization, Wave S/P/C | Directive §10 |
| `log_rows=0` logistics visual (data/scenario) | Triage → logistics lane |

---

## D — Agent workflow (every cycle until closed)

1. Read `debug_runs/agent_debug_index.json` then `stage5_full_app_live.json`.
2. If A1–A9 fail → fix **highest authority** violation only (TODO-01/04/06 first).
3. If A green but sticky infra appears → **add to triage**, do not expand `STAGE5_TODOS`.
4. Run B verification when claiming CLOSED.

---

## E — Sign-off log

| Date | `passes` | visual proof | commit / notes |
|------|----------|--------------|----------------|
| 2026-05-23 | true | `--test visual` exit 0; `wrote stage5 FULL_APP live proof` streak=120 | **CLOSED** — §B complete; see [`stage5_operational_signoff.md`](stage5_operational_signoff.md) |

**Status:** **CLOSED** — Stage 5 operational gate signed off 2026-05-23. **§B sign-off complete** (A1–A9 + B1–B6).

**Closure run (agent):** A1 ✅ · A4/B1 ✅ (fresh JSON + `stage5_closure`) · B2 ✅ · B3 ✅ · B4 ✅ · B5 ✅ · B6 ✅.  
**Unblocks shipped:** VR-07 (`alpha` WGSL) · VR-08 (`VERTEX_FRAGMENT` bind layout) · VR-09 (fire witness ↔ `instanced_dispatch_ok`).
