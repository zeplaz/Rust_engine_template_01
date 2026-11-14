# M1 weak-order / dual-writer note (Bevy 0.20)

**Date:** 2026-11-13  
**Owner:** @sim-steward / @coder  
**Status:** SIGN-OFF support for M1

## Finding

Bevy 0.20 orders some built-in schedule SystemSets **weakly**. Our production graph must not rely on accidental order when producers write through interior mutability / channels / `NonSend` without a `Res` edge.

## Actions taken in M0/M1 compile pass

- Mechanical ECS API migrate: `ExtractResource`/`ExtractComponent` + `#[extract_app(RenderApp)]`, `NextState::set_if_different`, pointer flatten (`PointerClick`), `run_if(...).or_else(...)`, UI `ui_widgets::Button`.
- No new dual writers introduced.
- Authority map updated to **Bevy 0.20** with weak-order warning.

## Residual risks (do not silently “fix”)

| Area | Risk | Mitigation |
|:---|:---|:---|
| Fire extract vs SyncViewManager | Still depends on named `.after(ViewAuthority…)` / FireVisualFrame sets | Keep strict edges; SCH Wave 2+ stays **DR-SCHED-W2** |
| Atmosphere / weather GPU extract | Multiple ExtractResource plugins | Each resource sole writer; extract_app(RenderApp) required |
| StrategicFieldPipeline | Contested with combat/logistics | Existing set chain; prefer Res edges over string `.after` |

## Dual-writer scan (spot)

| Resource | Sole writer (unchanged) |
|:---|:---|
| ViewManager | `sync_view_manager_bridge` |
| MissionBook | `apply_drone_mission_intents` |
| LogisticsOrderSim | order-sim plugin |
| ChunkSimLod | `chunk_sim_lod_refresh` |

No second writer added during 0.20 bump.
