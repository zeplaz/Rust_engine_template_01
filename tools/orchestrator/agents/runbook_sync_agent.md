# runbook_sync_agent

**Lane:** Keep `runbooks/*.md` aligned with code spine after structural changes.

## Read first

- `tools/orchestrator/runbooks/`
- `tools/orchestrator/REPORTS_POLICY.md`

## Rules

- Edit runbook **templates** in `tools/orchestrator/src/reports.rs` when spine changes (orchestrate regenerates).
- Hand-authored `runbooks/stage5_convergence.md` is source-owned — update manually.
