# WSS-DESIGN-GATE-001 — substrate slab design gate `v1`

| Field | Value |
|:---|:---|
| **Gate ID** | **WSS-DESIGN-GATE-001** (slab preflight) |
| **Parent gate** | [`wssr_design_gate_brief_v1.md`](wssr_design_gate_brief_v1.md) — identity + signoff deliverables |
| **Unblocks** | [`plan_wss_chunk_slab_exec_001_v1.md`](plan_wss_chunk_slab_exec_001_v1.md) (**WSS-CHUNK-SLAB-001**) |
| **Version** | `1.0.0` |
| **Date** | 2026-05-26 |
| **Owner** | `@designer` (UX/diagnostics) + `@sim-steward` (regression) |
| **Status** | **PENDING** — **FAIL until signed** |
| **Planner** | G5 module path ☑ in exec plan |

**Rule:** `@coder` must **not** be assigned `WSS-CHUNK-SLAB-001` until:

1. Parent gate: `wssr_design_signoff_v1.md` verdict **PASS** (or **PASS qualified**) for WSS-PLAN-002 row  
2. This preflight: all G1–G6 ☑ and `pass: true` below

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
| G1 | Hybrid migration note in diagnostics or dev doc — "ECS authoritative until PR-2" | `@designer` | ☐ |
| G2 | Debug overlay / diagnostics labels: `substrate_chunk_count`, `substrate_resident`, `substrate_dirty` | `@designer` | ☐ |
| G3 | Empty world + Simulation enter — no crash, no map wash regression | `@sim-steward` | ☐ |
| G4 | `chunk_environment_set` order test green with `SubstratePlugin` | `@sim-steward` | ☐ |
| G5 | Module path `src/substrate/` confirmed | `@planner` | ☑ |
| G6 | Witness flags `hybrid_ecs_*_authoritative: true` documented for operators | `@designer` | ☐ |

---

## G1 deliverable (designer)

Short section in diagnostics UI or collapsed dev panel:

```text
World Substrate (PR-1)
  Slab chunks: N   Resident: R   Dirty: D
  Sim authority: ChunkWeather + ChunkSurfaceFire (ECS) — slab is mirror only until PR-2
```

Reference: [`plan_wss_chunk_slab_exec_001_v1.md`](plan_wss_chunk_slab_exec_001_v1.md) § Hybrid migration matrix.

---

## G2 overlay names (designer)

| Label | Source field |
|:---|:---|
| `substrate_chunk_count` | `registry.chunks.len()` |
| `substrate_resident` | `resident_count` |
| `substrate_dirty` | `dirty_count` |
| `substrate_plugin` | env `RUST_ENGINE_SUBSTRATE` |

Align naming with [`fire_streaming_debug_overlay_names_v1.md`](fire_streaming_debug_overlay_names_v1.md) pattern.

---

## Steward regression (G3–G4)

```powershell
cargo test -p proc_A_dine01 --lib chunk_environment_set
cargo test -p proc_A_dine01 --lib fire_ecology
```

Optional: `--test visual` smoke if substrate plugin enabled in harness — **not** Stage 5 gate.

---

## Verdict block (fill on sign-off)

```yaml
gate: WSS-DESIGN-GATE-001
pass: false  # set true when all G1-G6 ☑
signed_at: null
designer: pending
steward: pending
notes: []
```

---

## On PASS

1. Set `pass: true` and date in this file.
2. Planner may add `WSS-CHUNK-SLAB-001` to `coder_active_queue.json` with status `ready`.
3. Invoke: `.\tools\orchestrator\scripts\invoke_slice.ps1` (when slice row exists).

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-26 | Gate opened — PENDING |
