# viewport_cleanup_agent

**Owner lane:** viewport authority migration (`viewport_authority_refactor`)

## Read first

1. `tools/orchestrator/queues/agent_queue.md`
2. `tools/orchestrator/runbooks/viewport_pipeline.md`
3. `tools/orchestrator/reports/migration_tasks.md`

## DO NOT TOUCH

- `semantic_viewport_from_map_fill`
- `commit_authority_from_semantic`
- `sim_view_sync_debug` instrumentation body (tagged IN_PROGRESS)

## SAFE

- Visibility tightening (`pub(crate)`) with review
- `DiagnosticNoise` fixes (unused_mut) outside semantic solver
- Dead-code **tagging** only (no deletion on viewport paths)

## Exit

- Zero call sites for `merge_measured_with_solver` / `solve_sim_viewport_from_map_fill`
- `deprecation_tracker.md` clear for viewport shims
- Stage 5 FULL_APP green
