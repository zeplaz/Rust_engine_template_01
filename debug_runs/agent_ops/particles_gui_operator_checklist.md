# Operator verify — Particles + GUI drain (G-PLAY + scenario pixels)

**Audience:** human with display  
**Drain:** `tools/orchestrator/queues/particles_gui_drain_queue.json` · HUD native `$ref:tools/orchestrator/queues/production_sim_hud_native_queue.json`  
**Date:** 2026-09-23 · **refreshed:** machine slices HUD-A-* / VSS-T3-003/004 **done**

## Session A0 — clean env

```powershell
cd C:\dev\github\Rust_engine_template_01
Remove-Item Env:UI_LAYOUT_DEBUG,Env:STAGE5_VERBOSE,Env:CONSTRUCTION_PLACEMENT_DEBUG,Env:TACTICAL_VFX_PROOF -ErrorAction SilentlyContinue
```

## Particles (product path)

```powershell
cargo run -p proc_A_dine01 --release -- --scenario assets/scenarios/play/default_industrial_demo_fire.scenario.ron
```

**Pass when (α≈0.42 operational zoom):**
- [ ] Heat overlay visible on burn cells
- [ ] Sparks readable (`spark_witness.rows > 0` in live diagnostics / stage5 if captured)
- [ ] Smoke column readable near fire
- [ ] No shader panic

Lib gate (agents): `ignition_single_funnel: true` + `fire_save_roundtrip_hash: true` in `debug_runs/sim_effect_fire_consumer_live.json`

## G-PLAY-OPERATOR-01 (10 min) — HUD-OPS-001

- [ ] Enter Simulation — HUD collapses idle chrome (PLAY-01); ops strip ~30 px
- [ ] Arm Industry building — context tray peeks on **Build** tab with modifiers
- [ ] Esc clears ghost; second Esc / RMB clears tool (not stuck Place NxN)
- [ ] No floating Construction placement debug Window in sim (editor-only)
- [ ] Build rail hits feel ≥48 px; iso buildings ~1.65 draw scale
- [ ] Dock panels stay responsive under mild load (budget grace)

## Machine closed (do not re-pick)

| Slice | Witness |
|:---|:---|
| VSS-T3-001 / T4-005b | `sim_effect_fire_consumer_live.json` · `artist_vfx_pipeline_live.json` |
| VSS-T3-003 | `debug_runs/fire_save_roundtrip_live.json` |
| VSS-T3-004 | SMOKE_VOLUME_WGSL const pruned |
| HUD-A-001/002/003 | ops/rail + tray + `hud_a_sim_window_ban_live.json` |
| HUD-OPS-002 | queues unblocked · HANDOFF scrubbed |

## Next machine pick

**Human:** HUD-OPS-001 / G-PLAY · **Designer:** VSS-T4-005 EffectSpec panel

**Machine Phase A–C closed** — NAT-001..007 · witness `debug_runs/production_sim_hud_native_live.json`

**Close:** update `tools/orchestrator/queues/HANDOFF.md` G-PLAY line when human signs.

Agent cannot self-certify pixels — this checklist is the P-C / G4 exit.
