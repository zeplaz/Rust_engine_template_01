# Infra slice 3 — OPS-F01 + WC-D04 plan `v1` (PLAN-INFRA-SLICE2-001 · Part B · Coder B lane)

| Field | Value |
|:---|:---|
| **Queue ID** | **PLAN-INFRA-SLICE2-001** (Part B) |
| **Version** | `1.0.0` |
| **Date** | 2026-05-25 |
| **Owner** | `@planner` |
| **Status** | **ACTIVE** — **WC-D04-CODER-B** **DONE** · **OPS-F01** + **OPS-F03** (operator) open |
| **VM-09 closure** | [`vm09_slice2_closure_signoff_v1.md`](vm09_slice2_closure_signoff_v1.md) — **CLOSED** |
| **Wave C** | [`stages/wave_c_depth_plan_v1.md`](stages/wave_c_depth_plan_v1.md) |
| **Board** | [`post_stage6_active_todos.md`](post_stage6_active_todos.md) Phase D + F |
| **Witness** | [`debug_runs/stage6_virtualization_live.json`](../debug_runs/stage6_virtualization_live.json) · [`debug_runs/perf_attribution_60s.md`](../debug_runs/perf_attribution_60s.md) |

**No Rust in this doc.** Launch plan for **Wave C residency churn** after VM-09 slice 2 is closed. **Coder B** = Wave C / Stage 6 lane (disjoint from **TRIAGE-VM-09-CODER-B**).

---

## Lane split

| Lane | ID | Owner | Blocks WC-D04? |
|:---|:---|:---|:---:|
| **Operator** | **OPS-F01** | operator | **Yes** — dated 60s perf sample required |
| **Coder B** | **WC-D04** / **WC-DEPTH-003** | `@coder` | Implements churn tune + witness fields |
| **Operator** | **OPS-F03** | operator | After WC-D04 — refresh `stage6_virtualization_live.json` in sim |

**Do not** assign **OPS-F01** to @coder — capture only. **Do not** touch `view_representation.rs` in this slice (VM-09 closed).

---

## Gate chain

```text
CLOSED   VM-09 slice 2 — vm09_slice2_closure_signoff_v1.md
         wave_c_live.json — wave_c_green, open_backlog_items: 0
         │
         ▼
NOW      OPS-F01 — operator 60s perf → perf_attribution_60s.md (2026-05-25+ section)
         │
         ▼
THEN     WC-D04 — Coder B — FrameBudgetDiagnostics / stage6 witness
         │
         ▼
OPS      OPS-F03 — operator sim refresh stage6_virtualization_live.json
```

---

## OPS-F01 — operator checklist (prerequisite for Coder B)

| # | Action |
|:---:|:---|
| 1 | Repo root; enter **Simulation** (prefer live session over harness-only) |
| 2 | `$env:RUST_LOG="warn,perf=info,perf_scope=info,stall=info"` · `$env:STALL="1"` |
| 3 | Run ~60s; capture `perf_scope`, `STALL culprit=*`, `emit_frame_perf_summary` |
| 4 | Append **dated** block to [`debug_runs/perf_attribution_60s.md`](../debug_runs/perf_attribution_60s.md) |
| 5 | Note any **`ResidencyChurn`** lines from HUD / `FrameBudgetDiagnostics` (feeds WC-D04 threshold) |

**Done when:** `perf_attribution_60s.md` has a **2026-05-25+** sample (not only 2026-05-22 template).

### Copy-paste — OPS-F01 (operator)

```
Lane: OPS-F01 — 60s perf attribution
Read: src/dev/infra_slice3_wc_d04_ops_f01_plan_v1.md
      debug_runs/perf_attribution_60s.md
Run: $env:STALL="1"; $env:RUST_LOG="warn,perf=info,perf_scope=info,stall=info"
     cargo run -p proc_A_dine01 --release
Session: BaseState::Simulation ~60s
Deliver: append dated sample + top-3 buckets + ResidencyChurn notes
Unblocks: WC-D04 (Coder B)
```

---

## WC-D04 — Coder B implementation contract

**Alias:** **WC-DEPTH-003** in [`wave_c_depth_plan_v1.md`](stages/wave_c_depth_plan_v1.md).

### Goal (S6-26 / board WC-D04)

Tune **residency churn** reporting so tactical sim sessions show stable `gpu_upload_bytes_frame` when atlas is active, without false-positive churn storms.

### Primary files (max 3 per session)

| File | Change |
|:---|:---|
| [`src/gui/hud/frame_budget_diagnostics.rs`](../gui/hud/frame_budget_diagnostics.rs) | `ResidencyChurn` threshold / hysteresis (WC-D04) |
| [`src/render/stage6_live_proof.rs`](../render/stage6_live_proof.rs) | Witness fields / violation strings |
| [`src/dev/stage6_virtualization.rs`](../dev/stage6_virtualization.rs) or stage6 module tests | Acceptance helpers |

**Read-only context:** `ChunkResidencyTable`, `stage6_virtualization_live.json` schema, OPS-F01 perf notes.

### Acceptance

| # | Criterion |
|:---:|:---|
| C1 | OPS-F01 sample present in `perf_attribution_60s.md` |
| C2 | Churn hysteresis documented in code or plan note (consecutive-frame threshold) |
| C3 | `cargo test -p proc_A_dine01 --lib stage6 wave_c` green |
| C4 | After **OPS-F03** sim run: `stage6_virtualization_live.json` — `gpu_upload_bytes_frame > 0` when atlas active; no `gpu_upload_inactive` violation |
| C5 | `infrastructure_view_isolation_green` still **true** (no viewport regression) |

### Witness (lib proof — **WC-D04-CODER-B** refreshed)

| File | Field | Value |
|:---|:---|:---|
| `stage6_virtualization_live.json` | `stage6_virtualization_green` | `true` (lib seed) |
| | `wc_d04.green` | `true` |
| | `gpu_upload_bytes_frame` | `4096` (lib test commit) |
| **OPS-F03** | operator sim refresh | replaces lib seed with live GPU path |
| `wave_c_live.json` | `wave_c_green` | `true` |
| | `open_backlog_items` | `0` |

**WC-D04 is not VM-09** — defer if a large `view_runtime` refactor is in the same PR.

### Copy-paste — WC-D04 (Coder B)

```
Lane: INFRA-SLICE3 — WC-D04 (WC-DEPTH-003) — Coder B
Read: src/dev/infra_slice3_wc_d04_ops_f01_plan_v1.md
      src/dev/stages/wave_c_depth_plan_v1.md
      post_stage6_active_todos.md WC-D04
Prereq: OPS-F01 dated perf sample in perf_attribution_60s.md
First: frame_budget_diagnostics.rs — ResidencyChurn threshold/hysteresis
Max files: 3 — frame_budget_diagnostics.rs, stage6_live_proof.rs, stage6 tests
Do NOT: view_representation.rs; MapCameraDesired writers; Stage 5 readiness formula edits
Verify: cargo test -p proc_A_dine01 --lib stage6 wave_c stage5
Witness: stage6_virtualization_live.json — gpu_upload_bytes_frame > 0 (after OPS-F03 sim refresh)
Handoff: OPS-F03 operator refresh
```

---

## OPS-F03 — operator (post Coder B)

| # | Action |
|:---:|:---|
| 1 | Simulation session ~30–60s after WC-D04 merge |
| 2 | Confirm `debug_runs/stage6_virtualization_live.json` timestamp updated |
| 3 | Confirm `stage6_virtualization_green: true` or document remaining violation |

```powershell
cargo test -p proc_A_dine01 --lib stage6
# Optional: cargo run -p proc_A_dine01 --release  # live sim writer
```

---

## Parallel lanes (OK)

| Track | Disjoint? |
|:---|:---:|
| UI-P3-M4 · S7B-DESIGN-001 · FX-WATER closed | ☑ |
| TRIAGE-VM-09-v2 | ☑ (separate — do not mix with WC-D04 session) |
| World Preview optional polish | ☑ |

**Coordinate** if touching `stage6_virtualization.rs` + `view_runtime.rs` same session — prefer **WC-D04 only**.

---

## Regression

```powershell
cargo test -p proc_A_dine01 --lib stage5 stage6 wave_c view_runtime
cargo orchestrate --skip-cargo
```

---

## Sign-off

| Role | Date | Status |
|:---|:---|:---|
| Planner | 2026-05-25 | **SIGNED** — Part B launch plan |
| Operator | — | **OPS-F01** / **OPS-F03** open |
| Coder B | — | **WC-D04** open (after OPS-F01) |

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-25 | PLAN-INFRA-SLICE2-001 Part B — OPS-F01 + WC-D04 for Coder B |
