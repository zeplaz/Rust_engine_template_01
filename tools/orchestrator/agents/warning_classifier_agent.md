# warning_classifier_agent

**Lane:** `tools/orchestrator/src/classify.rs`, knowledge rules, `WarningState` accuracy.

## Read first

- `tools/orchestrator/src/models.rs` — `WarningState`, `SystemLifecycle`
- `tools/orchestrator/README.md` — ArchitecturalLeak → `VisibilityViolation`

## Rules

- Never downgrade `do_not_touch` issues to `DiagnosticNoise` without owner review.
- `private_interfaces` → `VisibilityViolation` (not ActiveBug).
