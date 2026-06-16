# Next work queue (post-orchestrator)

**Orchestrator routing (2026-05-26):** read [`queues/HANDOFF.md`](queues/HANDOFF.md) and [`docs/archive/2026-06-src-dev/plans/orchestrator_signoff_snapshot_20260526_v1.md`](../docs/archive/2026-06-src-dev/plans/orchestrator_signoff_snapshot_20260526_v1.md) first — steward preflights and wave 3 coders are **CLOSED**.

**Gate:** `cargo orchestrate --skip-clippy --skip-test` → 0 issues (last: `20260522_021031`)

---

## P0 — Instrumentation & drift ✅

- [x] **N-01** Wire `sim_view_sync_debug` → `ViewportRectSanity`, `CommandLeftStackState`, `MapCameraDesired`, `MinimapShellState`
- [x] **N-02** `frozen_exceeds_semantic_authority` in `publish_simulation_map_viewport` heal path
- [x] **N-03** Marker scanner: regex boundaries (`TEMP` no longer matches `temperature`)
- [x] **N-04** Removed invalid `#[must_use]` on `viewport_pipeline` test module

## P1 — Stage 5 exit

- [x] **N-10** `cargo test -p proc_A_dine01 --lib stage5` — 19 passed
- [x] **N-11** `tools/orchestrator/scripts/visual_full_app.ps1` (`cargo run -- --test visual`)
- [x] **N-12** `ci/run.ps1` runs stage5 tests before orchestrate

## P2 — Map view spine

- [x] **N-20** Test: `resolved_frames_do_not_alias_world_preview_to_simulation_map`
- [x] **N-21** Flip `map_view/mod.rs` to STABLE after live minimap + preview witness (witness: `debug_runs/stage5_full_app_live.json`)

## P3 — Ops

- [x] **N-30** `REPORTS_POLICY.md` (commit policy documented)
- [x] **N-31** `.github/workflows/orchestrator.yml`

---

## Compile warnings (2026-05-20)

- Registry: [`docs/archive/2026-06-src-dev/plans/compile_warnings_registry.md`](../docs/archive/2026-06-src-dev/plans/compile_warnings_registry.md)
- Todos: [`src/dev/COMPILE_WARNINGS_TODOS.md`](../src/dev/COMPILE_WARNINGS_TODOS.md)
- **Build:** 0 rustc warnings after CW-01..CW-08

## Post-PLAY follow-up board

**Closed:** [`post_play_followup_todos.md`](../docs/archive/2026-06-src-dev/plans/post_play_followup_todos.md). **Active:** [`next_action_todos.md`](../src/dev/next_action_todos.md)

## Still manual (operator)

1. **Live FULL_APP:** `.\tools\orchestrator\scripts\visual_full_app.ps1` → check `debug_runs/stage5_full_app_live.json`
2. **Viewport drift:** `SIM_VIEW_SYNC_DEBUG=1` while resizing HUD / preview (optional spot-check)
3. ~~**N-21:** map_view STABLE~~ ✅
4. ~~**Viewport migration queue**~~ ✅ (`migration_tasks` empty after orchestrate `20260522_021459`)

## Commands

```powershell
.\tools\orchestrator\ci\run.ps1
.\tools\orchestrator\scripts\visual_full_app.ps1
cargo orchestrate --skip-clippy --skip-test
```
