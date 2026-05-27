# WSS design sign-off `v1` (WSS-DESIGN-GATE-001)

| Field | Value |
|:---|:---|
| **Queue ID** | **WSS-DESIGN-GATE-001** |
| **Deliverable** | 4 of 4 — formal sign-off |
| **Parent brief** | [`wssr_design_gate_brief_v1.md`](wssr_design_gate_brief_v1.md) |
| **Companion deliverables** | [`wssr_identity_alignment_record_v1.md`](wssr_identity_alignment_record_v1.md) · [`wssr_readability_impact_v1.md`](wssr_readability_impact_v1.md) · [`wssr_migration_visual_contract_v1.md`](wssr_migration_visual_contract_v1.md) |
| **Version** | `1.0.0` |
| **Date** | 2026-05-26 |
| **Owner** | `@designer` |
| **Status** | **SIGNED — PASS (qualified)** |
| **Unblocks** | [`wssr_coder_hybrid_orders_v1.md`](wssr_coder_hybrid_orders_v1.md) → **WSS-CHUNK-SLAB-001**, **WSS-ATMOS-CLIPMAP-001**, **WSS-HYDRO-RUNTIME-001** |

---

## Gate verdict

**WSS-DESIGN-GATE-001: PASS (qualified)**

Substrate refactor direction is **approved** for coder spine entry. Qualification requires **hybrid coexistence** with superior incumbents (per-view fire extract, closed water W1/W2 witnesses, ViewManager isolation, D-F09/D-W09 cull). No deletion of closed VFX tracks without named successor + regression plan documented in hybrid assessment.

---

## Child plan sign-off

| Child plan | Designer verdict | Conditions |
|:---|:---|:---|
| **WSS-PLAN-002** chunk slabs | **PASS (qualified)** | Dual-write `ChunkWeather` ↔ slab until `dual_write_drift_max < epsilon`; `ActiveChunkRuntime` only for hot reasons; no render writes to slab; paging uses sim focus not per-view camera |
| **WSS-PLAN-003** hydrology | **PASS (qualified)** | `FluidDomain` / `OceanSystem` forbidden; maintain W1/W2 tactical look via [`water_surface_target_v1.png`](../assets/vfx/reference/water/water_surface_target_v1.png); `gpu_water_*` stays L3; ocean_tile witness slab-backed; construction triggers via `HydrologyDirtyReason` event bus only |
| **WSS-PLAN-004** atmosphere | **PASS (qualified)** | Legacy `AtmosphereField` bridges to L1 until clipmap witness green; sim clipmap ≠ render clipmap; contamination separate domain with pattern language; smoke stub removed only after extract node; D-F09/D-W09 cull preserved; CPU precip transitional until GPU path signed |
| **Hanabi spike** | **PASS** — experiments only | `experiments/hanabi_validation/` only; no main `EnginePlugin` merge until H-A report + W4-C started; Layer 3 embellishment bounds per migration contract; never weather/smoke authority |

---

## Mandatory hybrid defaults (coder)

From identity alignment — **default to HYBRID** unless assessment proves incumbent obsolete:

| Slice | Hybrid default |
|:---|:---|
| WSS-CHUNK-SLAB-001 | Types + registry; map existing components → future slots; no deletion |
| WSS-ATMOS-CLIPMAP-001 | New stack alongside 128² field; bridge alias |
| WSS-HYDRO-RUNTIME-001 | Gen hydrate → slab; gpu_water unchanged consumer |
| Hanabi | Spike report only |

Each slice requires Hybrid Assessment YAML per [`wssr_coder_hybrid_orders_v1.md`](wssr_coder_hybrid_orders_v1.md).

---

## Preserve — non-negotiable regressions

| Guard | Witness |
|:---|:---|
| FIRE7 per-view extract | `fire_streaming_live.json` |
| Tactical VFX closure | `stage5_full_app_live.json` → `tactical_vfx_witness` |
| F1 fire ecology | `fire_ecology_live.json` |
| Water W1/W2 tactical | `water_w1_green`, D-W09 strategic band |
| View isolation | `infrastructure_view_isolation_live.json` |
| Construction spine | `construction_stage_live.json` |
| Strategic spark/water cull | D-F09, D-W09 — zero rows at strategic zoom OK |

---

## Do-not list (designer enforcement)

- Mandate deletion of closed tracks (F7 exit, FX-WATER closure) without hybrid plan
- Approve new top-level `*OceanSystem*`, `*DustSystem*`, `*WeatherVfxSystem*`
- Disable strategic cull globally for witness greens
- Allow GPU/Hanabi to write gameplay without readback contract
- Route construction ghost authority through atmosphere or hydrology writers

---

## Witness expectations (post-coder)

Designer re-review triggers when first coder slice lands:

| Path | Keys |
|:---|:---|
| `debug_runs/wss_substrate_live.json` | `slab_registry_present`, `dual_write_drift_max`, `hydrology_hydrated`, `clipmap_levels_present`, `contamination_domain_present`, `green` rollup |

Qualified PASS may upgrade to full PASS when:

1. Dual-write drift green in CI fixture
2. Tactical VFX witnesses unchanged at default zoom
3. No minimap particle bleed in capture round

---

## Routing

| Next agent | Action |
|:---|:---|
| `@planner` | Acknowledge sign-off; keep WSS rows in active queue |
| `@coder` | Begin **WSS-CHUNK-SLAB-001** with hybrid assessment |
| `@sim-steward` | Regression bundle on each WSS PR |
| `@designer` | Optional: tray mock if construction+WSS overlay overlap found in playtest |

---

## Sign-off table

| Role | Name | Verdict | Date |
|:---|:---|:---|:---|
| **Designer** | WSS-DESIGN-GATE-001 | **PASS (qualified)** | 2026-05-26 |
| Planner | — | Pending ack | — |
| Orchestrator | — | Unblock coder orders on designer PASS | — |

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-26 | Four deliverables complete; gate PASS (qualified) |
