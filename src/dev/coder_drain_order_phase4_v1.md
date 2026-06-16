# Coder drain order — Phase 4 `v1`

**Authority:** [`tools/orchestrator/queues/coder_drain_queue.json`](../tools/orchestrator/queues/coder_drain_queue.json)  
**Rule:** Pick **seq 1** ready row → implement → witness → **Q✓** → next seq. Skip `blocked` / `deferred`.

---

## Single handoff (@coder)

```text
Drain coder_drain_queue.json top→bottom.
Wave 1: src/sim/effects/ per plan_sim_effect_spine_exec_001_v1.md P0–P1;
        hydro dedupe + ember waist; JSONL debug_runs/sim_effects/effects.jsonl;
        NO dev Postgres in proc_A_dine01;
        witness debug_runs/sim_effect_spine_live.json.
Wave 2 (parallel B): two-click build + unified cursor per design_build_ux_redesign_v1.md.
Wave 3: FIRE-IGNITION-P0-001 then fire play visibility.
Wave 4+: zoom smooth, vfx lock, terrain blob.
Regression each wave: cargo test -p proc_A_dine01 --lib sim_effects fire_ecology construction stage5
```

---

## Drain table (run-through)

| Seq | Wave | ⟨ID⟩ | Owner | Status | Exit / witness |
|:---:|:---:|:---|:---|:---|:---|
| 1 | 1 | **SIM-EFFECT-QUEUE-001** | A | ⚡ready | `sim_effect_spine_live.json` · `queue_drain_ok` |
| 2 | 1 | **SIM-EFFECT-TEL-001** | A | ready | `causal_chain_depth_max ≥ 1` · `effects.jsonl` |
| 3 | 2 | **TRIAGE-BUILD-CLICK-PLACE-001** | B | ⚡ready | two-click + Ctrl/Shift adjust |
| 4 | 2 | **TRIAGE-CURSOR-UNIFY-001** | B | ready | single game cursor |
| 5 | 3 | **FIRE-IGNITION-P0-001** | A | ready | `fire_ecology_live.json` · lightning/grid producers |
| 6 | 3 | **TRIAGE-FIRE-PLAY-VIS-001** | A | ready | operator sees fire in normal play |
| 7 | 4 | **TRIAGE-MAP-ZOOM-SMOOTH-001** | A | ready | `map_zoom_coherence_live.json` |
| 8 | 4 | **P0-VFX-ZOOM-LOCK-001** | B | ready | vfx zoom not locked |
| 9 | 4 | **P0-TERRAIN-BLOB-001** | B | ready | terrain seam |
| 10 | 5 | **SCENARIO-TRIGGER-001** | A | ready | `EmitSimEffect` scenario step |
| 11 | 5 | EVENT-LOG-UI-001 | B | ready | DESIGN-EVENT-LOG-001 PASS · Events tray |
| 12 | 5 | FACTION-REACT-001 | A | deferred | after P3 event log |
| 13 | 6 | SIM-EFFECT-EMBED-DB-001 | — | 🧊 | GAME-STORE-GATE only |
| 14 | 6 | NARRATIVE-GEN-001 | designer | 🧊 | P6 |

---

## Territory map (no toe-stepping)

| Owner | Files |
|:---|:---|
| **coder_a** | `src/sim/effects/` · `src/systems/fire/` · `src/gui/map_camera.rs` · `src/scenario/` |
| **coder_b** | `src/construction/` · `src/gui/hud/simulation_pointer_gate.rs` · harness zoom · `tile_world_fallback.rs` |

**Parallel OK:** Wave 1 (A) + Wave 2 (B) simultaneously.

---

## Witness index

| Path | Slice |
|:---|:---|
| `debug_runs/sim_effect_spine_live.json` | QUEUE + TEL |
| `debug_runs/sim_effects/effects.jsonl` | TEL |
| `debug_runs/fire_ecology_live.json` | FIRE-IGNITION |
| `debug_runs/construction_stage_live.json` | BUILD-CLICK |
| `debug_runs/map_zoom_coherence_live.json` | MAP-ZOOM |

---

## Done (do not re-pick)

- PLAN-SIM-EFFECT-SPINE-001 · PLAN-PRODUCT-POLISH-001
- DESIGN-BUILD-UX-REDESIGN-001 PASS
- TRIAGE-MAP-PICK-CLOSURE-001 · TRIAGE-FIRE-PRODUCT-001

---

## CLI test worlds (after wave 3)

```powershell
cargo run -p proc_A_dine01 --release -- --test fire
cargo run -p proc_A_dine01 --release -- --test weather
cargo run -p proc_A_dine01 --release -- --test vfx
cargo run -p proc_A_dine01 --release
```
