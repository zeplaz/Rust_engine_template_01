# Elemental VFX & environmental systems — planner domain charter `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **PLAN-ELEMENTAL-VFX-DOMAIN-001** |
| **Version** | `1.0.0` |
| **Date** | 2026-05-26 |
| **Owner** | `@planner` (fire · weather · water · terrain-linked particles) |
| **Skills** | bevy-simulation-grade · cleanup-completion-intelligence · debug-intelligence |
| **Status** | **SIGNED** — routing baseline for Wave 6+ VFX (docs only; no Rust) |

**Rule:** Witness JSON wins. Do **not** re-plan closed tracks: FIRE7-PLAN-001, VFX-P2 closure, FX-WATER W1/W2 sign-off, F7-A/B/C exit.

---

## Executive summary

Simulation truth: ECS fire/weather/atmosphere + terrain fuel/hydrology. Presentation: **one** fire extract path → `FireVisualFramesByView` → projection graph → GPU particles + `gpu_weather_fire_field`. Phase 2 VFX + F7 are **closed**; active work = **F2 projection graph**, smoke/atmosphere completion, optional water witness tails, **weather sim v2** (unqueued until runbook signed).

---

## Authority map (mandatory)

| Layer | Owner | Must not |
|:---|:---|:---|
| **Sim** | `systems::fire`, `systems::weather`, `systems::atmosphere`, `terrain::fire`, hydrology markers | GPU writes gameplay |
| **Extract** | `fire_view_extract` (one writer/frame), `ClimateVisualAggregate`, projection graph nodes | Second global fire extract; minimap ECS fire query |
| **GPU** | `gpu_weather_fire_field`, `gpu_particles`, water raster | Sample field into sim without readback contract |
| **Forbidden** | — | `MapCameraDesired` as fire cull authority; witness-only greens (FIRE7-PLAN-001) |

Full stack: [`docs/archive/2026-06-prompts-guides/runbooks/guides/vfx_architecture_bevy_wgpu_v1.md`](../docs/archive/2026-06-prompts-guides/runbooks/guides/vfx_architecture_bevy_wgpu_v1.md) (if present) · [`fire_sim_phase7_architecture_v1.md`](fire_sim_phase7_architecture_v1.md).

---

## Cleanup classification

| Module | Class | Action |
|:---|:---|:---|
| `gpu_weather_fire_field` | B transitional | **Preserve** — GPU field spine |
| `weather_visual` mesh precip | B transitional | **Preserve** until GPU/Hanabi |
| `fire_visual_emit_smoke_stub` | D incomplete | **Completion plan** → smoke extract |
| `terrain/fire/*` ontology | C dormant anchor | **Preserve** — F1 ecology |
| Witness-only greens | A obsolete | **Forbidden** per F7 exit policy |

---

## Implementation phases (planner routing)

| Phase | ID | Owner | Status |
|:---:|:---|:---|:---|
| 0 | Regression | all | **Always on** — `stage5`, `fire_streaming`, `fire_ecology`, tactical_vfx |
| 1 | **F7-STREAM-DEEP-001** | @coder A | **CLOSED** 2026-05-26 — `neighbor_wake_observed` |
| 2 | Water tails (W1/W2 foam/ocean) | @coder A | **Optional P2** — track CLOSED; tails if witness zero |
| 3 | **PLAN-FIRE-F2-EXTRACT-001** | @coder A | **P1 next** — `fire_instance_buffer_rows`, VX-P2-01 / F-T02 |
| 4 | **WEATHER-SIM-PLAN-001** | @planner | **P1 planner** — runbook v1 → v2 signed |
| 5 | Smoke / atmosphere (P2-H) | @coder + @designer | After F2 + smoke spec |
| 6 | Terrain dust / Hanabi | future | Hanabi 0.18 audit gate |

---

## Diagnostics

| Witness | Path |
|:---|:---|
| Tactical VFX | `debug_runs/stage5_full_app_live.json` → `tactical_vfx_witness` |
| Fire streaming | `debug_runs/fire_streaming_live.json` |
| Fire ecology F1 | `debug_runs/fire_ecology_live.json` |
| View isolation | `debug_runs/infrastructure_view_isolation_live.json` |
| Stage 6 | `debug_runs/stage6_virtualization_live.json` |
| Captures | `assets/vfx/reference/review_captures/` |

```powershell
cargo test -p proc_A_dine01 --lib gpu_particles fire_streaming stage5
```

---

## Open questions (planner backlog)

1. Hanabi adoption timeline vs Bevy 0.18 pin  
2. Weather sim priority vs construction parametric / M3 depth  
3. Ocean fixture: world-gen scenario vs catalog stub  
4. Smoke: particles vs field density vs hybrid (designer spec)  
5. Terrain dust: this domain vs logistics/vehicle lane  

---

## Next planner deliverables (priority order)

| Priority | Queue ID | Artifact |
|:---:|:---|:---|
| **1** | **PLAN-FIRE-F2-EXTRACT-001** | [`plan_fire_f2_extract_exec_001_v1.md`](plan_fire_f2_extract_exec_001_v1.md) (expand from [`fire_ecology_f1_todos.md`](fire_ecology_f1_todos.md) F2-*) |
| **2** | **WEATHER-SIM-PLAN-001** | [`weather_simulation_runbook_v2_plan_v1.md`](weather_simulation_runbook_v2_plan_v1.md) from [`prompts/guides/weather_simulation_runbook_v1.md`](../prompts/guides/weather_simulation_runbook_v1.md) |
| **3** | **PLAN-SMOKE-ATMOSPHERE-001** | Wire stub → `ChunkSmokeField` + GPU bridge (after design) |

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-26 | Domain charter filed for orchestrator routing |
