# F7-A product exit — acceptance matrix `v1` (PLAN-F7-A-EXIT-001)

| Field | Value |
|:---|:---|
| **Queue ID** | **PLAN-F7-A-EXIT-001** |
| **Coder queue** | **FIRE7-F7-A-EXIT-001** (Coder A wave 3 **#1**) |
| **Version** | `1.0.0` |
| **Date** | 2026-05-26 |
| **Owner** | `@planner` |
| **Status** | **SIGNED** |
| **Architecture** | [`fire_sim_phase7_architecture_v1.md`](fire_sim_phase7_architecture_v1.md) § F7-A exit |
| **Track plan** | [`stages/fire_sim_phase7_plan_v1.md`](stages/fire_sim_phase7_plan_v1.md) |
| **Preflight** | [`steward_fire7_preflight_gate_v1.md`](steward_fire7_preflight_gate_v1.md) **GO** |
| **Design (F7-C prereq only)** | [`fire_lod_player_read_v1.md`](fire_lod_player_read_v1.md) — **not** part of F7-A exit |
| **Implementation** | [`src/render/fire7_f7_a_exit.rs`](../render/fire7_f7_a_exit.rs) · [`src/render/view_runtime/live_proof.rs`](../render/view_runtime/live_proof.rs) |
| **No Rust in this doc** | Acceptance + witness contract only |

---

## Executive summary

**F7-A exit** closes **per-view fire extract hardening** so **real** F7-B (streaming) and F7-C (LOD caps) may start.

| Lane | ID | What it proves |
|:---|:---|:---|
| **Witness bundle (v2)** | **FIRE7-F7-A-001** | `f7_a_per_view_extract_bounded` in infra JSON — **subset** of exit |
| **Product gate** | **FIRE7-F7-A-EXIT-001** | **A1–A5** below — **required** to unblock F7-B/C |

**Anti-pattern:** `fire7_f7_a_001.green` alone does **not** close F7-A. **Anti-pattern:** hand-edited JSON without lib refresh.

---

## Exit criteria (A1–A5)

| # | Criterion | Pass when | Evidence (code) | Evidence (witness) |
|:---:|:---|:---|:---|:---|
| **A1** | Sole `FireVisualFramesByView` producer | `fire_visual_producer_count() == 1` | [`fire_view_extract.rs`](../render/fire_view_extract.rs) registration · [`fire7_f7_a_exit.rs`](../render/fire7_f7_a_exit.rs) `sole_fire_visual_producer` | `fire7_f7_a_exit_001.sole_fire_visual_producer: true` |
| **A2** | Per-view extract bounded | `per_view_fire_extract_bounded(by_view, vis, active)` | Same module · `Stage5FireViewChunkWitness::f7_a_per_view_extract_bounded` | `fire7_f7_a_001.f7_a_per_view_extract_bounded: true` |
| **A3** | Minimap does **not** query fire ECS | `!minimap_compositor_queries_fire_ecs()` — no `FireSimulationSnapshot` / chunk sets under `src/render/minimap_compositor/*.rs` | [`fire7_f7_a_exit.rs`](../render/fire7_f7_a_exit.rs) ripgrep guard | `fire7_f7_a_exit_001.minimap_fire_overlay_only: true` |
| **A4** | Explicit exit witness field | Rollup `fire7_f7_a_exit_001.green` written from runtime proof | [`view_runtime/live_proof.rs`](../render/view_runtime/live_proof.rs) `build_proof_payload` | `fire7_f7_a_exit_001.green: true` · alias `fire7_f7_a_001_green: true` |
| **A5** | Stage 5 / fire extract regression | Lib tests green (no new global extract) | See § Tests | `infrastructure_view_isolation_green: true` (rollup) |

**Rollup (A1–A4 in code):** `fire7_f7_a_exit_001_green(by_view, vis, active)` → [`Fire7F7AExitCriteria::green`](fire7_f7_a_exit.rs).

**A5** is verified by **cargo test**, not embedded in `Fire7F7AExitCriteria`.

---

## Witness JSON contract

**File:** `debug_runs/infrastructure_view_isolation_live.json`  
**Writer:** `refresh_infrastructure_view_isolation_live_witness()` (sim / lib refresh — not manual edit)

### Required paths

| JSON pointer | Type | Meaning |
|:---|:---|:---|
| `/fire7_f7_a_exit_001/gate` | string | `"FIRE7-F7-A-EXIT-001"` |
| `/fire7_f7_a_exit_001/green` | bool | **Product exit** rollup |
| `/fire7_f7_a_exit_001/fire7_f7_a_001_green` | bool | Same rollup (legacy alias for A4) |
| `/fire7_f7_a_exit_001/sole_fire_visual_producer` | bool | A1 |
| `/fire7_f7_a_exit_001/minimap_fire_overlay_only` | bool | A3 |
| `/fire7_f7_a_001/green` | bool | A2 witness bundle row |
| `/fire7_f7_a_001/f7_a_per_view_extract_bounded` | bool | A2 |
| `/vm_11/f7_a_per_view_extract_bounded` | bool | Diagnostic mirror |

### Sibling block (v2 — not sufficient alone)

| JSON pointer | Note |
|:---|:---|
| `/fire7_f7_a_001/gate` | `"FIRE7-F7-A-001"` — bounded extract only |

---

## Verification commands

### Coder exit (required)

```powershell
cargo test -p proc_A_dine01 --lib f7_a_exit_requires_single_producer_and_bounded_extract
cargo test -p proc_A_dine01 --lib fire_view_extract
cargo test -p proc_A_dine01 --lib fire_visual_extract
cargo test -p proc_A_dine01 --lib stage5
cargo test -p proc_A_dine01 --lib render::view_runtime
```

### Witness refresh

```powershell
cargo test -p proc_A_dine01 --lib refresh_infrastructure_view_isolation_live_witness
# or wave-3 bundle:
cargo test -p proc_A_dine01 --lib coder_a_wave3_14_closure_bundle
```

### Manual JSON check

```powershell
# green must be true after refresh
Get-Content debug_runs/infrastructure_view_isolation_live.json | Select-String fire7_f7_a_exit
```

---

## Forbidden (do not count as F7-A exit)

| Pattern | Why |
|:---|:---|
| Set `fire7_f7_a_exit_001.green` in JSON without running refresh | False green |
| Add second global `FireVisualFrame` extract “to pass witness” | Violates **FIRE7-PLAN-001** |
| Minimap compositor imports `FireSimulationSnapshot` | Fails A3 |
| `fire_streaming_live.json` all-green with **no** streaming systems | That is **F7-B**, blocked until this exit |
| LOD caps in JSON only, no `FireChunkLodState` / extract clamp | That is **F7-C**, blocked until this exit |
| Treat **FIRE7-F7-A-001** v2 row as F7-B/C unblock | Witness bundle ≠ product gate |

---

## Gate chain (after exit)

```text
FIRE7-PLAN-001          ☑
FIRE7-PREFLIGHT-001     ☑ GO
FIRE7-DESIGN-001        ☑ fire_lod_player_read_v1.md (F7-C design only)
        │
        ▼
FIRE7-F7-A-EXIT-001     ☑ wave 3
FIRE7-F7-B-001         ☑ same wave-3 pass (not a separate gate)
FIRE7-F7-C-001         ☑ same wave-3 pass (not a separate gate)
```

---

## Coder handoff — FIRE7-F7-A-EXIT-001

```
Lane: FIRE7-F7-A-EXIT-001
Read: src/dev/fire7_f7_a_exit_acceptance_v1.md (this doc)
      src/dev/fire_sim_phase7_architecture_v1.md
Touch: fire7_f7_a_exit.rs, view_runtime/live_proof.rs (≤3 files if fixing)
Do: satisfy A1–A5; refresh infrastructure_view_isolation_live.json
Do NOT: F7-B streaming, F7-C LOD, second global extract
Verify: cargo test -p proc_A_dine01 --lib coder_a_wave3_14_closure_bundle
Exit: /fire7_f7_a_exit_001/green == true
```

---

## Landed with exit (wave 3 — same pass)

| Queue ID | Agent | Proof |
|:---|:---|:---|
| **FIRE7-F7-B-001** | @coder A | `fire_streaming_live.json` · `refresh_fire_streaming_live_witness` |
| **FIRE7-F7-C-001** | @coder A | `fire7_f7_c_001_green()` · designer caps in extract |
| **FIRE7-DESIGN-LOD-WIRE-001** | @coder A | ☑ wired in wave 3 bundle |

---

## Sign-off

| Role | Date | Verdict |
|:---|:---|:---|
| Planner | 2026-05-26 | **SIGNED** — acceptance matrix on disk |
| Coder | — | Close when A1–A5 green + witness refresh |
| Steward | — | Reconfirm preflight if extract path changes |

**On-disk witness (2026-05-26):** `fire7_f7_a_exit_001.green: true` in [`debug_runs/infrastructure_view_isolation_live.json`](../../debug_runs/infrastructure_view_isolation_live.json) — coder may mark **FIRE7-F7-A-EXIT-001** **DONE** after running § Verification commands in this session.

---

## Document history

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-26 | **PLAN-F7-A-EXIT-001** — A1–A5, JSON paths, tests, anti-patterns |
