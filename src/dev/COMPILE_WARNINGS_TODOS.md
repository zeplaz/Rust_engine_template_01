# Compile warnings — action todos

Synced with [`compile_warnings_registry.md`](compile_warnings_registry.md).  
**Gate:** `cargo build -p proc_A_dine01` → **0 warnings**; `cargo rustc -p proc_A_dine01 --lib -- -D warnings`. Re-run after every change batch — the P1–P4 checklist was policy/scaffold work, not a one-time rustc sweep.

---

## Done (this session)

- [x] **CW-01** Wire `frozen_exceeds_semantic_authority` in `publish_simulation_map_viewport`
- [x] **CW-02** Trim unused `pub use` in `construction/roads`, `rail`, `zones` mod.rs
- [x] **CW-03** Remove unused imports in `economy/activation/live_proof.rs`
- [x] **CW-04** Wire `shift_lmb_*` in `build_interaction`
- [x] **CW-05** Use `description` / `building_height` in `file_to_definition`
- [x] **CW-06** Log `ConstructionValidation.required_actions` on road reject
- [x] **CW-07** Wire `preview_gpu_authoritative` via `preview_gpu_authoritative_run_if`
- [x] **CW-08** Mark rail junction + `SnapTarget` scaffold (now wired — see P2)

---

## P1 — Clippy hygiene

- [x] **CW-10** Clippy policy: [`clippy_policy.md`](clippy_policy.md); `too_many_arguments` allowed in `Cargo.toml`
- [x] **CW-11** `empty_line_after_outer_attr` set to warn in `Cargo.toml` (batch fix on touch)
- [x] **CW-12** Precedence in roads pathing — no active clippy precedence hits (N/A)

---

## P2 — Construction scaffold

- [x] **CW-20** `RailJunctionAuthority::register_switch` on rail commit at road-network tiles
- [x] **CW-21** `snap_placement` / `SnapTarget` + HUD via `ConstructionPathFeedback`
- [x] **CW-22** `required_actions` + snap in panel + tool hints
- [x] **CW-23** Public surface documented at top of `construction/mod.rs`

---

## P3 — Viewport / visual

- [x] **CW-30** Removed empty `ViewportAuthorityDebugPlugin` (integrity plugin remains)
- [x] **CW-31** `in_game_ui` behind `legacy_engine` feature; not exported in default build
- [x] **CW-32** [`recovery_viewport.md`](recovery_viewport.md) migration status table
- [x] **CW-33** `debug_runs/viewport_drift.json` + `SIM_VIEW_SYNC_DEBUG=1` documented

---

## P4 — Ops

- [x] **CW-40** Orchestrator `build_report.md` includes rustc stderr warning counts
- [x] **CW-41** `ci/run.ps1`: `cargo rustc -p proc_A_dine01 --lib -- -D warnings` (package-only; not global `RUSTFLAGS`)

---

## CW-50 — Reconcile (open)

Terminal warnings from **2026-05-22** visual sessions (`TagSet`, `SiteFootprint`, `history.rs`, construction dead_code) are **not** tracked as open CW rows — the CW-01…41 board was marked done earlier.

- [ ] **CW-50** After each visual/GPU batch: `cargo build -p proc_A_dine01` and `cargo build -p proc_A_dine01 --release` → **0 warnings**; log to `debug_runs/compile_warnings.log`
- [ ] **CW-51** If warnings return, file a new CW-5x row with file:line (do not assume CW-01…41 still cover the site)
- [ ] **CW-52** Optional: wire `arm_visual_test_graceful_exit` or allow dead_code on visual-teardown scaffold

**Cross-link:** [`visual_run_blockers.md`](visual_run_blockers.md) **VR-03**.

---

## Commands

```powershell
cargo build -p proc_A_dine01 2>&1 | Tee-Object debug_runs/compile_warnings.log
cargo clippy -p proc_A_dine01 --lib 2>&1 | Tee-Object debug_runs/clippy_warnings.log
cargo test -p proc_A_dine01 construction:: --lib
cargo rustc -p proc_A_dine01 --lib -- -D warnings
```
