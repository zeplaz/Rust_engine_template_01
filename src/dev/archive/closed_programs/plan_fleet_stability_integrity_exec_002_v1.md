# PLAN-FLEET-STABILITY-INTEGRITY-EXEC-002 — P1 de-hack boundary slices `v1`

| Field | Value |
|:---|:---|
| **Parent** | [`plan_fleet_stability_integrity_001_v1.md`](plan_fleet_stability_integrity_001_v1.md) |
| **Sweep** | [`production_jank_sweep_20260602_v1.md`](production_jank_sweep_20260602_v1.md) |
| **Queue ID** | **PLAN-FLEET-STABILITY-INTEGRITY-EXEC-002** |
| **Version** | `1.0.0` |
| **Date** | 2026-06-02 |
| **Owner** | `@planner` (sign-off) · `@coder A` + `@coder B` (implementation) |
| **Status** | **SIGNED — ACTIVE** |
| **Supersedes (open work)** | [`plan_fleet_stability_integrity_exec_001_v1.md`](plan_fleet_stability_integrity_exec_001_v1.md) — **historical**; exec-001 rows marked done in queue stay closed |
| **Prior exec** | exec-001 (PLAY-TRUTH, DEHACK-VIEW/FIRE, CONTAIN-D, WSS, STAB tails) |
| **Audit** | [`planner_status_audit_v17.md`](planner_status_audit_v17.md) |
| **Dispatch** | [`fleet_stability_coder_dispatch_v1.md`](fleet_stability_coder_dispatch_v1.md) |
| **Machine queue** | [`coder_active_queue.json`](../../tools/orchestrator/queues/coder_active_queue.json) |

---

## Phase posture (PHASE-STABLE sweep)

| Item | State |
|:---|:---|
| **PHASE-STABLE-2026-06** | **ACTIVE** — open coder work is **P1-A/B/C only** (this doc) |
| **P0 #1 harness gate** | **DONE** — `TestHarnessPlugin` only when `EngineLaunchArgs.test_mode()` (`engine_with_worldgen.rs`) |
| **PLAY-TRUTH-002 / 003** | **DONE** — `ProofGrade`, fixture vs visual keys; see exec-001 closure in queue |
| **Wave 7 PERF-VIS / DEV-CONTAIN-002–006** | **CLOSED** — do **not** reopen in `active[]` unless witness regression |
| **P2 env registry** | **DEHACK-ENV-001** — optional parallel doc slice; not a ship gate for P1 PRs |

---

## Slice sign-off rule (locked)

> **A slice is done only when non-test production code cannot import or call the removed API without `cfg` (feature or `#[cfg(test)]`).**
>
> Runtime guards (`if test_mode`, `ProofGrade::VisualCapture` no-op, env checks) are **necessary** but **not sufficient** for DEHACK-ENG / RENDER / LOG closure.

Verification per slice:

1. `cargo check -p proc_A_dine01` (default features, no `--test`).
2. Grep / compile-fail test: a new `src/dev/dehack_*_boundary_gate.rs` (or module test) that **fails to compile** if forbidden symbols are reachable from `src/main.rs` dependency graph without cfg.
3. Existing lane tests (`stage5`, `logistics`, `dehack_log_001_*`) remain green.

---

## Locked slices — one PR each

| Slice | ID | Owner | Blocks | PR scope |
|:---|:---|:---|:---|:---|
| Engine API boundary | **DEHACK-ENG-001** | A | — | Harness types not in default `engine` public surface |
| Render witness API boundary | **DEHACK-RENDER-001** | A | after **ENG-001** or **parallel** (disjoint files) | `refresh_*_live_witness` not in default `render::` re-exports |
| Logistics shortcut surface | **DEHACK-LOG-001** | B | **parallel** | `patch_s7p_*` / `apply_s7p_*` not callable from non-cfg production imports |

---

### DEHACK-ENG-001 — Engine API boundary

**Problem (P1-A):** `src/engine/mod.rs` still `pub mod test_harness` and re-exports `TestHarnessPlugin`, `TestWorldHarness`, `ActiveTestScene`, etc. `src/main.rs` may insert harness resources unconditionally.

**Files (primary):**

- `src/engine/mod.rs`
- `src/engine/test_harness.rs`
- `src/engine/engine_with_worldgen.rs` (plugin registration — already gated; keep)
- `src/main.rs`
- `Cargo.toml` — feature `dev_harness` (or `test_harness`) default **off**

**Exit (cfg boundary):**

- Default build: `main` and product plugins do not `use crate::engine::TestHarnessPlugin` / `TestWorldHarness` without `cfg(feature = "dev_harness")` or `#[cfg(test)]`.
- `--test` / CI harness: enable feature or `test_mode()` path only.
- `cargo check -p proc_A_dine01` green; harness lib tests still run with `--features dev_harness` or test cfg as documented in PR.

**Tests:** `cargo test -p proc_A_dine01 --lib stage5 construction` (harness feature documented in PR body).

---

### DEHACK-RENDER-001 — Render witness API boundary

**Problem (P1-B):** `src/render/mod.rs` re-exports many `refresh_*_live_witness` helpers used by `src/dev/*_bundle_proof.rs` — easy to call from production code.

**Files (primary):**

- `src/render/mod.rs`
- `src/render/stage5_full_app_harness.rs` (witness writers stay; refresh helpers move)
- `src/dev/*_bundle_proof.rs`, `src/dev/runtime_witness/` — update imports to `#[cfg(...)]` dev path
- `Cargo.toml` — feature `dev_witness_tools` default **off** (or `#[cfg(test)]` module re-export)

**Exit (cfg boundary):**

- Default `use crate::render::refresh_log_e01_*` **does not compile**.
- Runtime witness **writers** in `runtime_witness` schedule remain authoritative; refresh = dev/test only.
- `cargo test -p proc_A_dine01 --lib stage5` with dev feature or test cfg per PR.

**Blocks:** None strictly; prefer **parallel** with ENG-001 (no file overlap).

---

### DEHACK-LOG-001 — Logistics shortcut surface

**Problem (P1-C):** `apply_s7p_*` / `patch_s7p_*` still exported from `src/economy/logistics/mod.rs`. PLAY-TRUTH-002 made `VisualCapture` a no-op; **public surface** still violates compile-time boundary.

**Files (primary):**

- `src/economy/logistics/mod.rs`
- `src/economy/logistics/witness.rs`
- `src/economy/logistics/witness_collectors.rs`
- `src/engine/test_harness.rs` (fixture alignment only, behind harness cfg)
- `src/render/stage5_full_app_harness.rs` (if any direct shortcut import)

**Exit (cfg boundary):**

- Default build: non-test code cannot import `patch_s7p_logistics_throughput_witness_for_play_proof` or `apply_s7p_logistics_throughput_witness_shortcut`.
- Fixture / lib proof: `#[cfg(test)]` or `dev_harness` module only.
- `dehack_log_001_*` tests green; `cargo test -p proc_A_dine01 --lib logistics` (48/48).

**Note:** Behavioral guard from PLAY-TRUTH-002 is **already landed**; this PR completes the **export / import** gate.

---

## Optional P2 (not in `active[]` unless planner promotes)

| ID | Owner | Doc | Exit |
|:---|:---|:---|:---|
| **DEHACK-ENV-001** | planner + A review | [`runtime_env_policy_registry_v1.md`](runtime_env_policy_registry_v1.md) | Every runtime env from sweep §D classified: `debug-only` \| `ops-runbook` \| `ship-config` \| `deprecated` |

---

## Explicitly out of scope (exec-002)

| ID | Reason |
|:---|:---|
| PERF-VIS-P1BC, P2*, P3/4, GPU default tails | Wave 7 closed — witness regression only |
| DEV-CONTAIN-002–007 | Closed — HardFail CI |
| DEHACK-VIEW-001, DEHACK-FIRE-001, CONTAIN-D-001, DEHACK-WSS-001 | exec-001 done per queue |
| STAB-PERF-001, STAB-VT-001, OPS-* | Horizon C — not P1 sweep |

---

## Regression (every PR)

```powershell
cargo check -p proc_A_dine01
cargo test -p proc_A_dine01 --lib stage5 logistics construction
.\tools\orchestrator\scripts\check_live_proof_containment.ps1
```

After DEHACK-RENDER-001 (if dev feature added):

```powershell
cargo test -p proc_A_dine01 --lib stage5 --features dev_witness_tools
```

(Document exact feature flags in each PR.)

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-06-02 | PHASE-STABLE P1 sweep — three locked PRs; P0 harness gate done; cfg sign-off rule |
