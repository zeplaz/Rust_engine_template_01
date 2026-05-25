# Orchestrator completion TODO

**Status: COMPLETE** — last run `20260520_041052`, **0 issues**, BROKEN=0.

Re-run: `cargo orchestrate -- --skip-clippy --skip-test`

---

## P0 — Unblock the spine ✅

- [x] **O-01** Run full pipeline baseline
- [x] **O-02** Fix ActiveBug/Fatal misclassified warnings → **0 compiler issues**
- [x] **O-03** Symbol extraction (`unused variable`, private type names)
- [x] **O-04** Dedupe diagnostics (message-keyed)

---

## P1 — Orchestrator tool hardening ✅

### Automation & CI

- [x] **O-10** CI script `tools/orchestrator/ci/run.ps1`
- [x] **O-11** README workflow docs
- [x] **O-12** Post-build hook `hooks/post_build.ps1`
- [x] **O-13** `tools/orchestrator/target/` in `.gitignore`

### Parser & classifier quality

- [x] **O-20** Static `#[deprecated]` scan → `deprecation_tracker.md`
- [x] **O-21** `private_interfaces` → `VisibilityViolation`
- [x] **O-22** `unused_mut` / `unused_variables` → `DiagnosticNoise`
- [x] **O-23** ArchitecturalLeak alias documented in README

### Reports & drift

- [x] **O-30** `reports/drift_summary.md`
- [x] **O-31** `REPORTS_POLICY.md`
- [x] **O-32** Unit tests (JSON parse, subsystem trace)

---

## P2 — Knowledge graph & runbooks ✅

### Knowledge base

- [x] **K-01** `knowledge/map_view_spine.json`
- [x] **K-02** `knowledge/render_pipeline.json`
- [x] **K-03** `knowledge/stage5_readiness.json`
- [x] **K-04** Multi-file knowledge loader

### Runbooks

- [x] **K-10** `viewport_pipeline.md` (file map + COMPLETE status)
- [x] **K-11** `ui_pipeline.md` (egui ordering)
- [x] **K-12** `render_pipeline.md` (extraction nodes)
- [x] **K-13** `runbooks/stage5_convergence.md`

### Agent queue artifacts

- [x] **K-20** `agents/viewport_cleanup_agent.md`
- [x] **K-21** All agent briefs under `agents/`
- [x] **K-22** `queues/stage5_todo_crossref.md`

---

## P2 — Source annotations rollout ✅

- [x] **A-01** `sim_view_sync_debug.rs`
- [x] **A-02** `authoritative_viewport.rs`
- [x] **A-03** `viewport_authority_debug.rs`
- [x] **A-04** `gui/mod.rs` (deprecated exports removed)
- [x] **A-05** `map_view/mod.rs`
- [x] **A-06** `reports/marker_triage.md` (generated each run)

---

## P1 — Viewport authority migration ✅

- [x] **V-01** No call sites for deprecated wrappers
- [x] **V-02** Removed deprecated re-exports + shim functions
- [x] **V-03** Tagged rescue floor / `frozen_exceeds_semantic_authority`
- [x] **V-04** Visibility on trace types
- [x] **V-05** `debug_runs/viewport_authority_migration_witness.json`
- [x] **V-06** `headless_full_app_readiness_fixture_is_green` passes
- [x] **V-07** `knowledge/viewport_authority.json` migration closed

---

## P3 — Runtime integration ✅

- [x] **R-01** `OrchestratorHealthPlugin` + `OrchestratorThreadHealthExport`
- [x] **R-02** Export to `debug_runs/orchestrator_thread_health.json` (`ORCHESTRATOR_EXPORT_HEALTH=1`)
- [x] **R-03** `--runtime-snapshot PATH` CLI
- [x] **R-04** Drift repro documented in viewport runbook

---

## P3 — Operational readiness ✅

- [x] **S-01** Orchestrator step in `AGENTS.md`
- [x] **S-02** Registry clean (0 issues); triage via `marker_triage.md` on regressions
- [x] **S-03** `ci/nightly_orchestrate.ps1`

---

## Quick commands

```powershell
cargo orchestrate -- --skip-clippy --skip-test
.\tools\orchestrator\ci\run.ps1
.\tools\orchestrator\ci\nightly_orchestrate.ps1
$env:ORCHESTRATOR_EXPORT_HEALTH = "1"  # then run app
cargo orchestrate -- --skip-cargo --runtime-snapshot debug_runs/orchestrator_thread_health.json
```
