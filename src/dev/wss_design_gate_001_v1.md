# WSS-DESIGN-GATE-001 — substrate slab design gate `v1`

| Field | Value |
|:---|:---|
| **Gate ID** | **WSS-DESIGN-GATE-001** (slab preflight) |
| **Parent gate** | [`wssr_design_gate_brief_v1.md`](wssr_design_gate_brief_v1.md) — **CLOSED** — [`wss_design_gate_parent_closure_v1.md`](wss_design_gate_parent_closure_v1.md) |
| **Unblocks** | [`plan_wss_chunk_slab_exec_001_v1.md`](plan_wss_chunk_slab_exec_001_v1.md) (**WSS-CHUNK-SLAB-001**) |
| **Version** | `1.1.0` |
| **Date** | 2026-05-26 |
| **Owner** | `@designer` (UX/diagnostics) + `@sim-steward` (regression) |
| **Status** | **G1–G2 CLOSED** · **G3–G4 steward pending** · rollup `pass: false` |
| **Planner** | G5 module path ☑ in exec plan |

**Rule:** `@coder` may start **WSS-CHUNK-SLAB-001** when:

1. Parent gate: `wssr_design_signoff_v1.md` **PASS (qualified)** ☑  
2. Designer preflight **G1–G2** ☑ ([`wss_substrate_diagnostics_copy_v1.md`](wss_substrate_diagnostics_copy_v1.md), [`wss_substrate_debug_overlay_names_v1.md`](wss_substrate_debug_overlay_names_v1.md))  
3. Full preflight `pass: true` — after **G3–G4** steward ☑

---

## Purpose

PR-1 introduces **`WorldSubstrateRegistry`** alongside **unchanged** ECS chunk components. Operators and agents must understand:

- sim truth still lives in `ChunkWeather`, `ChunkSurfaceFire`, etc.
- slab is hydrate + witness skeleton only
- no player-facing presentation change expected

---

## Checklist

| # | Check | Owner | Result |
|:---:|:---|:---|:---:|
| G1 | Hybrid migration note in diagnostics or dev doc — "ECS authoritative until PR-2" | `@designer` | ☑ [`wss_substrate_diagnostics_copy_v1.md`](wss_substrate_diagnostics_copy_v1.md) |
| G2 | Debug overlay / diagnostics labels: `substrate_chunk_count`, `substrate_resident`, `substrate_dirty` | `@designer` | ☑ [`wss_substrate_debug_overlay_names_v1.md`](wss_substrate_debug_overlay_names_v1.md) |
| G3 | Empty world + Simulation enter — no crash, no map wash regression | `@sim-steward` | ☐ |
| G4 | `chunk_environment_set` order test green with `SubstratePlugin` | `@sim-steward` | ☐ |
| G5 | Module path `src/substrate/` confirmed | `@planner` | ☑ |
| G6 | Witness flags `hybrid_ecs_*_authoritative: true` documented for operators | `@designer` | ☑ [`wss_substrate_debug_overlay_names_v1.md`](wss_substrate_debug_overlay_names_v1.md) § G6 |

---

## G1 deliverable (designer) — CLOSED

See [`wss_substrate_diagnostics_copy_v1.md`](wss_substrate_diagnostics_copy_v1.md).

```text
World Substrate (PR-1)
  Hybrid (PR-1): ECS components are authoritative for weather and fire. WorldSubstrateRegistry is hydrate + witness only.
  WSS slab_chunks=N resident=R dirty=D
  WSS sim authority: ChunkWeather + ChunkSurfaceFire (ECS)
  WSS slab mirror only until PR-2 cutover
```

Reference: [`plan_wss_chunk_slab_exec_001_v1.md`](plan_wss_chunk_slab_exec_001_v1.md) § Hybrid migration matrix.

---

## G2 overlay names (designer) — CLOSED

See [`wss_substrate_debug_overlay_names_v1.md`](wss_substrate_debug_overlay_names_v1.md).

| Label | Source field |
|:---|:---|
| `substrate_chunk_count` | `/chunk_count` in witness |
| `substrate_resident` | `/resident_count` |
| `substrate_dirty` | `/dirty_count` |
| `substrate_plugin` | `/substrate_plugin_enabled` |

Align naming with [`fire_streaming_debug_overlay_names_v1.md`](fire_streaming_debug_overlay_names_v1.md) pattern.

---

## Steward regression (G3–G4)

```powershell
cargo test -p proc_A_dine01 --lib chunk_environment_set
cargo test -p proc_A_dine01 --lib fire_ecology
```

Optional: `--test visual` smoke if substrate plugin enabled in harness — **not** Stage 5 gate.

---

## Verdict block

```yaml
gate: WSS-DESIGN-GATE-001
parent_gate_closed: true
designer_g1_g2_g6: true
pass: false  # set true when G3-G4 steward ☑
signed_at: 2026-05-26
designer: G1-G2-G6 signed
steward: pending
notes:
  - Parent 4/4 deliverables closed — wss_design_gate_parent_closure_v1.md
  - Coder WSS-CHUNK-SLAB-001 unblocked on qualified parent + G1-G2
```

---

## On full PASS

1. Set `pass: true` after G3–G4 ☑.
2. Upgrade slab preflight in [`wssr_design_signoff_v1.md`](wssr_design_signoff_v1.md) if needed.
3. `WSS-CHUNK-SLAB-001` remains `ready` in `coder_active_queue.json`.

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.1.0 | 2026-05-26 | G1–G2/G6 closed; parent closure record; steward G3–G4 open |
| v1.0.0 | 2026-05-26 | Gate opened — PENDING |
