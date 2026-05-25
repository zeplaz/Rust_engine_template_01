# dead_code_agent

**Lane:** Deferred cleanup only — **never** delete viewport-tagged or `do_not_touch` symbols.

## Read first

- `tools/orchestrator/reports/warning_registry.md`
- `tools/orchestrator/reports/marker_triage.md`

## Rules

1. Only act on `DiagnosticNoise` or explicit owner approval.
2. Skip `IN_PROGRESS`, `LEGACY_TRANSITION`, `@orchestrator-do-not-cleanup`.
3. Prefer `#[allow(dead_code)]` + orchestrator tag over deletion when intent is unclear.
