# PLAN-REPLAY-PARITY-001 — replay + editor parity impl plan `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **PLAN-REPLAY-PARITY-001** |
| **Coder lane** | **REPLAY-PARITY-001** (Coder B #7) |
| **Version** | `1.0.0` |
| **Date** | 2026-05-26 |
| **Owner** | `@planner` |
| **Status** | **SIGNED** — lib witness **CLOSED** · live sim ring **PARTIAL** |
| **Code** | [`src/dev/replay_editor_parity.rs`](../dev/replay_editor_parity.rs) |
| **Witness** | `debug_runs/replay_editor_parity_live.json` |
| **Minimap scrub** | [`minimap_replay_scrub_visual_spec_v1.md`](minimap_replay_scrub_visual_spec_v1.md) — **orthogonal** |

**No Rust in this deliverable.**

---

## Scope

| In scope | Out of scope |
|:---|:---|
| `CommittedSimReplayRing` depth ≥ 2 | Full scenario determinism replay |
| Scenario plugin + editor panel wired | Editor undo/redo parity |
| Infra isolation JSON present | Stage 5 FULL_APP exit |
| Lib + sim witness writers | Minimap click-to-scrub |

---

## PASS gate (witness)

| # | Criterion | JSON path |
|:---:|:---|:---|
| P1 | Ring depth | `replay_ring_len >= 2` |
| P2 | Scenario plugin | `scenario_plugin_wired: true` |
| P3 | Editor panel | `editor_scenario_panel: true` |
| P4 | Infra JSON exists | `infrastructure_isolation_json: true` |
| P5 | Rollup | `parity_green: true` · `replay_parity_001_green: true` |

**Lib refresh:** `refresh_replay_editor_parity_live_witness()` (seed len 4).

**Sim path:** `write_replay_editor_parity_live_proof_system` in **Simulation** when ring grows ≥ 2.

---

## Lib vs live sim

| Path | Verdict |
|:---|:---|
| Lib test refresh only | **CLOSED** for infrastructure hardening row |
| Live Simulation commits | **PARTIAL** — product depth |
| Hand-edited JSON | **Forbidden** |

---

## Verification

```powershell
cargo test -p proc_A_dine01 --lib replay_editor_parity
```

Bundle: `coder_b_wave3_bundle_proof.rs` reads `replay_editor_parity_live.json`.

---

## Anti-patterns

| Forbidden | Why |
|:---|:---|
| Conflate with **UI-P3-M3-REPLAY-001** | Minimap scrub ≠ editor parity |
| `parity_green` without ring | Witness-only stub |

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-26 | **PLAN-REPLAY-PARITY-001** signed |
