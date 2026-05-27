# WSS identity alignment record `v1` (WSS-DESIGN-GATE-001)

| Field | Value |
|:---|:---|
| **Queue ID** | **WSS-DESIGN-GATE-001** |
| **Deliverable** | 1 of 4 — identity alignment |
| **Parent brief** | [`wssr_design_gate_brief_v1.md`](wssr_design_gate_brief_v1.md) |
| **Plans evaluated** | WSS-PLAN-002 · WSS-PLAN-003 · WSS-PLAN-004 |
| **Version** | `1.0.0` |
| **Date** | 2026-05-26 |
| **Owner** | `@designer` |
| **Status** | **SIGNED** |

---

## Overall verdict

| Scope | Verdict |
|:---|:---|
| **WSS substrate direction (L1→L2→L3)** | **PASS (qualified)** |
| **Chunk slabs (WSS-PLAN-002)** | **PASS (qualified)** |
| **Hydrology runtime (WSS-PLAN-003)** | **PASS (qualified)** |
| **Atmosphere unification (WSS-PLAN-004)** | **PASS (qualified)** |
| **Contamination split** | **PASS** |
| **Hanabi spike** | **PASS** — experiments only |

**Qualified** = hybrid coexistence mandatory for incumbent superior paths (fire per-view extract, closed water witnesses, ViewManager isolation). No wholesale deletion of closed VFX tracks without named successor + regression plan.

---

## Ethos fit

WSS serves the project identity when it reads as **one persistent planetary industrial archive** — terrain, water, atmosphere, contamination, and fire as facets of the same chunked world-state — rather than a bundle of feature silos (`OceanSystem`, `DustSystem`, weather particles). That aligns with the north star in [`wssr_index_v1.md`](wssr_index_v1.md): Dwarf Fortress persistence, W&R logistics pressure, ecological stress, and wargame command overlays on a **materially grounded** map.

The refactor is **not** generic engine middleware. It should feel like **state infrastructure overgrown by reality** ([`design_theme.md`](../prompts/guides/ui/design_theme.md)): layered environmental archive, warm oxidized industrial palette, magenta as drafting ink (not neon), ecological intrusion (smoke residue, water marks, contamination stains) interrupting clean geometry. WSS L1 truth supports that ethos by making decay, flood, plume, and ash **persistent and saveable** — the map accumulates history instead of resetting decorative VFX each frame.

Operational vs infrastructure distinction holds: WSS is **infrastructure hardening** (substrate, paging, persist). It must **not** become Stage 5 gate work or block FULL_APP spine. Presentation remains L3-transient; sim remains authoritative.

---

## Preserve list (do not delete without hybrid assessment + regression)

| Path / pattern | Reason |
|:---|:---|
| `src/render/fire_view_extract.rs` + `FireVisualFramesByView` | **Superior incumbent** — single-writer per-view fire extract (FIRE7-PLAN-001, F7 exit). WSS must **read** substrate snapshot; must not replace extract graph. |
| `src/render/gpu_weather_fire_field.rs` | **Transitional spine (B)** — Layer 3 field compute; consumes render clipmap / extraction, not sim authority. |
| `src/render/gpu_particles.rs` + D-F09 strategic cull | Closed spark track; tactical zoom band witness. WSS atmosphere must **preserve** cull policy, not global-disable for greens. |
| `src/render/view_runtime/*` · ViewManager · per-view residency | VM-08 / isolation authority. WSS paging uses **sim focus**, not per-view camera — views stay downstream. |
| `assets/vfx/reference/water/water_surface_target_v1.png` + W1/W2 closure | **SIGNED** water visual language (lake / river / ocean reads). Hydrology slab **feeds** extract; does not replace shader/particle vocabulary. |
| `src/systems/fire/*` · `terrain/fire/*` · `fire_ecology_live.json` | F1 ecology sim truth — orthogonal to WSS substrate; fuel/ignition gates stay ECS until explicit migration row. |
| `src/construction/*` ghost + commit funnel | Construction invariants: preview never mutates gameplay. WSS must not write occupation from L3 fields. |
| `src/strategic/site/*` commit + `BuildingDefinitionRef` | Gameplay sites remain ECS; slab coupling via events only (hydrology dirty reasons). |
| `ChunkEnvironmentSet` schedule order (Lod → Weather → Ecology → Fire) | Proven sim ordering; migrate writers to slab **incrementally**, not reorder blindly. |
| `debug_runs/stage5_full_app_live.json` → `tactical_vfx_witness` | Regression guard for tactical readability; WSS witness is **additive** (`wss_substrate_live.json`). |

---

## Replace list (only with named successor + regression plan)

| Incumbent | Successor | Regression plan |
|:---|:---|:---|
| Fixed 128² `AtmosphereField` as sole sim grid | `AtmosphereClipmapStack` L0–L3 | **Bridge:** alias legacy field → L1 until `wss_substrate_live.json` clipmap flags green; then deprecate with witness `legacy_atmosphere_field_removed`. |
| `fire_visual_emit_smoke_stub` | Projection-graph smoke extract node → Layer B render clipmap | Maintain `tactical_vfx_witness` smoke-visible row; remove stub only when extract node wired. |
| `DynamicTerrainOverlay` resource HashMaps | `WorldChunkState.dynamic` per chunk | Dual-write M1; drift metric in witness; construction mud/congestion reads unchanged at UX level. |
| “Ocean as VFX-only” queue language | `HydrologyState.ocean_mask` + slab-backed extract | Keep `water_ocean_tiles` witness; source becomes slab count, not VFX-only counter. |
| `WeatherVisualPlugin` CPU mesh precip as default rain | GPU precip / field composite (L3) | **Transitional:** `WeatherVisualSettings` default off when GPU path ready; designer sign-off before delete. |
| Monolithic `FluidDomain` (anti-pattern) | `ChunkSlab<HydrologyState>` + scheduled tasks | Explicitly **reject** — no replacement module named `OceanSystem`. |

**Not approved for deletion (closed tracks):**

| Track | Policy |
|:---|:---|
| FIRE7 F7-A/B/C exit | Preserve per-view streaming + neighbor wake witnesses |
| FX-WATER W1/W2 sign-off | Preserve tactical water witness; hydrology **extends** authority behind same look |
| VFX-P2 closure / D-F09 / D-W09 | Strategic cull is product intent, not test annoyance |

---

## Hybrid list (coexist until explicit gate)

| Domain | Hybrid shape |
|:---|:---|
| **Chunk weather** | Slab owns `WorldChunkState.atmosphere.local`; ECS retains `ChunkWeather` dual-write until clipmap sample refs wired (W2-B). |
| **Fire front** | Slab owns `thermal` + ecology fuel; ECS retains `ChunkSurfaceFire` + optional `ActiveChunkRuntime` for hot propagation; extract reads snapshot. |
| **Smoke** | Layer A: fold chunk smoke gen → L0 clipmap + contamination airborne; Layer B: existing `gpu_weather_fire_field` + future Hanabi wisps. |
| **Hydrology** | Slab owns depth/flow/masks; L3 `gpu_water_*` unchanged consumer; `HydrologyResult` gen → hydrate only (no second extract). |
| **Dust** | `AtmosphereClipmapStack.ash_density` transport + `ContaminationState.soil` deposit; vehicle kicks = **events** → field increment, not `DustSystem` module. |
| **Atmosphere GPU** | Sim clipmap stack (L0–L3) **≠** render clipmap upload; bridge is sole L1→L3 path for fields. |
| **Hanabi** | Spike in `experiments/hanabi_validation/` now; main plugin **after** W4-C + multiview stable — Layer 3 embellishment only. |
| **Minimap fire** | Heat-only channel from compressed L2/L3 sample; **no** ECS fire query in minimap compositor (charter rule). |

---

## Evaluation worksheets (per domain)

### chunk_slab

```yaml
domain: chunk_slab
current_system: Chunk entity + ChunkWeather + DynamicTerrainOverlay + scattered domain components
wss_proposal: Resource slabs keyed by ChunkKey; WorldSubstrateRegistry single writer; optional ActiveChunkRuntime for hot regions
superior_incumbent: partial
recommendation: hybrid
hybrid_shape: "ChunkSlab<WorldChunkState> owns persist; ChunkWeather + fire components dual-write until W2-B drift < epsilon; ActiveChunkRuntime only for FireFront/FloodSolve/Construction"
identity_risk: med
witness_impact: wss_substrate_live.json → slab_registry_present, dual_write_drift_max, resident_count
```

### hydrology

```yaml
domain: hydrology
current_system: terrain/generation HydrologyResult (gen-only) + gpu_water_* L3 + RiverMarker presentation gap
wss_proposal: ChunkSlab HydrologyState with scheduled background + event deep solve; ocean_mask in slab not OceanSystem
superior_incumbent: partial
recommendation: hybrid
hybrid_shape: "slab owns water_depth/flow/masks; gpu_water_* + water_surface_target_v1 look unchanged; HydrologyVisualExtract node feeds existing shaders"
identity_risk: low
witness_impact: wss_substrate_live.json → hydrology_hydrated, ocean_tile_count; stage5 water_w1/w2 rows unchanged
```

### atmosphere

```yaml
domain: atmosphere
current_system: AtmosphereField 128² + ChunkWeather + gpu_weather_fire_field + WeatherVisualPlugin precip
wss_proposal: AtmosphereClipmapStack L0-L3 sim + separate render clipmap; fold smoke/dust into fields
superior_incumbent: partial
recommendation: hybrid
hybrid_shape: "new clipmap stack owns sim; legacy AtmosphereField aliases L1 until bridge green; gpu_weather_fire_field consumes render clipmap only"
identity_risk: med
witness_impact: wss_substrate_live.json → clipmap_levels_present, sim_vs_render_resolution_ratio; fire_ecology atmosphere rows
```

### contamination

```yaml
domain: contamination
current_system: toxicity partially in AtmosphereCell only — insufficient for soil/water/structure
wss_proposal: separate ContaminationState per chunk + AtmosphereCoupling bridge
superior_incumbent: no
recommendation: adopt
hybrid_shape: "adopt as designed; AtmosphereCell.toxic_hazard remains derived sample only, not storage"
identity_risk: low
witness_impact: wss_substrate_live.json → contamination_domain_present, toxic_hazard_sample
```

### hanabi

```yaml
domain: hanabi
current_system: none in main app; gpu_particles + field compute for elemental VFX
wss_proposal: experiments/hanabi_validation spike; later Layer 3 local particles (embers, wisps)
superior_incumbent: partial
recommendation: hybrid
hybrid_shape: "spike now; adopt only for event embellishment after H-A report + W4-C; never weather/smoke authority"
identity_risk: high
witness_impact: hanabi_spike_report_present; no main plugin until signoff row closed
```

---

## Identity risks flagged

| Risk | Severity | Mitigation |
|:---|:---:|:---|
| Substrate refactor reads as “engine rewrite” not “living archive” | MED | Keep water/fire **look** contracts; change authority behind same visual language |
| Neon arcade particles via Hanabi | HIGH | Style bounds in migration contract; industrial micro-spark vocabulary only |
| Strategic map clutter at planetary zoom | MED | Preserve D-F09/D-W09; minimap uses compressed channels |
| Silo reintroduction (`*OceanSystem*`) | HIGH | Planner policy + designer reject in preserve/replace tables |
| Construction ghost bleed from substrate overlays | MED | Construction authority unchanged; WSS overlays route through representation graph |

---

## Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer` | **SIGNED** — PASS (qualified), hybrid default | 2026-05-26 |
