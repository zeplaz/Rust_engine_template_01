# Stage 5 convergence runbook

> Parent: [`prompts/guides/stage5_convergence_directive_v1.md`](../../../prompts/guides/stage5_convergence_directive_v1.md)

## Exit gate

`Stage5ReadinessProfile::FULL_APP` green in the **running app** (preview, overlays, GPU, camera, LOD) — not fixtures alone.

## Every cycle

1. `cargo test -p proc_A_dine01 --lib`
2. `cargo orchestrate --skip-clippy --skip-test`
3. FULL_APP readiness probe / visual test
4. TODO from `reports/migration_tasks.md` + `STAGE5_TODOS` failures only
5. Fix highest **authority** violation
6. Rerun FULL_APP; update docs

## Live board

- `STAGE5_TODOS` in `src/dev/stage5_live_todos.rs`
- Predicates: `sync_stage5_todo_board_predicates`
- Proof: `debug_runs/stage5_full_app_live.json`

## Distinction

| Lane | Meaning |
|------|---------|
| Operational readiness | FULL_APP green, spine valid, converged |
| Infrastructure hardening | VM backlog, multiview, replay — separate milestone |

See [`prompts/guides/operational_readiness_vs_infrastructure_perf_v1.md`](../../../prompts/guides/operational_readiness_vs_infrastructure_perf_v1.md).
