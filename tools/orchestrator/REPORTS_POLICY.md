# Reports policy (O-31)

## Generated artifacts

These paths are **refreshed** by `cargo orchestrate` and may be committed after meaningful runs:

- `reports/*.md` — human dashboards for agents
- `runbooks/*.md` — stable runbooks (edit source in repo; orchestrator may overwrite seeded sections)
- `queues/agent_queue.md`, `queues/continuation_queue.json`

## Volatile (gitignored)

- `state/last_run.json`
- `history/run_*.json`

## When to commit

1. After closing a migration (viewport, Stage 5 slice).
2. When `drift_summary.md` shows intentional resolution of a blocker class.
3. Do **not** commit empty registries without a matching code change.

## Nightly

See `ci/nightly_orchestrate.ps1` — artifacts optional; history kept locally for trend.
