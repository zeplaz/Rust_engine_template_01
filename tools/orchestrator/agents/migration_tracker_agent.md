# migration_tracker_agent

**Lane:** `deprecation_tracker.md`, `migration_tasks.md`, `knowledge/*.json`.

## Read first

- `tools/orchestrator/reports/deprecation_tracker.md`
- `tools/orchestrator/queues/continuation_queue.json`
- `debug_runs/viewport_authority_migration_witness.json` (closed template)

## Rules

- Update knowledge JSON when migrations open/close.
- Static `#[deprecated]` scan + compiler warnings must agree before closing a migration.
