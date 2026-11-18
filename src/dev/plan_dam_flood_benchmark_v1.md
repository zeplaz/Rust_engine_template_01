# PLAN-DAM-FLOOD-BENCHMARK-v1 — river → dam → reservoir → strike → breach → flood → recession

| Field | Value |
|:---|:---|
| **Id** | **PLAN-DAM-FLOOD-BENCHMARK-v1** |
| **Status** | **SIGNED** — DF-0 locked. **DF-1/2 ADOPTED**. **DF-3 landed** (GPU field + visual frame; capture Accept open). **DF-4 landed**. **DF-5–9 landed in code** (ecology flood/moisture, cooling/hydro, warheads, dam barrier leak path, flood consequences). **DF-10: BLOCKED baseline** (honest — S0 gauge + S3–S9 matrix not green). **Harness:** `src/scenario/dam_flood/`. **APS:** `kit_dam_flood_aps_001`. |
| **Date** | 2026-11-16 |
| **Purpose** | One scripted **example** scene that proves the **general** water / structure / damage / ecology / fire / power stack works together. The scene is the acceptance harness — not a parallel dam-only simulation. The first fully scored run becomes **baseline v1**. |
| **Supersedes** | `plan_wss_hydro_runtime_exec_001_v1.md` **HY-005 / HY-006** (deep solve + visual extract; witness-era exits) → replaced by **DF-2 / DF-3** here |
| **Review path** | `sim_run_dump.json` counters → `tools/sim_review/score_dump.py` matrix + dashboard (AGENTS.md). Counts only in the dump; verdicts only in the matrix. No witness JSON, no `*_green`. |
| **Units** | Metric throughout: m, m³/s, MW, °C, game seconds/days. `1 tile = 16 m` (`terrain/world_scale_contract.rs:18`), sim step = 1 game second (`SIM_STEP_MICROS`, `systems/sim_control.rs:127`). |

---

## 0. Non-negotiable: the dam is an example of player play, not a cheat path

### Product loop (what the systems must support)

```text
Place dam (may or may not include a hydro plant)
  → reservoir forms from real water solve (stage / storage / inundation)
  → any industrial site that needs cooling can hook an intake to that reservoir
      (nuclear, coal, chemical, data hall, … — not a dam-only special)
  → optional: hydro uses the same head × turbine flow
  → if the dam is damaged / destroyed: reservoir drains / flood wave
      → hooked coolers lose water → derate / trip
      → flood hits ecology, roads, sites downstream
```

**Dam ≠ “power building.”** The dam’s first job is to **make a reservoir**. Power generation is optional. Cooling consumers are **general industrial attachments** to reservoir stage (or river intake), not hardwired to one nuclear prefab in the DamFlood scene.

The DamFlood scene is a **scripted example** of that loop (dam + reservoir + one hooked plant + strike/breach) so we can score it. It is not a parallel toy world.

**This plan exists to force production systems to behave correctly under that hard scenario.**  
`TestScene::DamFlood` + `dam_valley_v1.ron` are a **repeatable configuration** (topology, gauges, timeline, expected envelopes). They are **not** permission to invent dam-only truth.

| Rule | Required | Forbidden (cheat) |
|:---|:---|:---|
| **Ownership** | Water / class / save / construction / ecology / combat / power live as **general** sim modules with sole writers (§4). Same path as a player dam + plant in a normal game. | Hiding a second “dam flood solver” that only runs under `--test dam` and disagrees with production water. |
| **Reservoir first** | Placing/closing a dam forms storage via the water solve; consumers **read** stage/depth. | Painting a lake because the narrative stage says “impound,” or baking cooling into a dam-only flag. |
| **Cooling is a hook** | Industrial sites declare an intake (tile / reservoir id / min stage). Any cooler can attach to the same reservoir. | `if DamFlood { cooling = true }` or nuclear-only water hacks that other industries cannot reuse. |
| **Destruction cascades** | Breach/drain updates water truth → hooked coolers derate/trip; flood uses the same depth field as placement/ecology/routes. | Separate “dam destroyed” cheat that trips plants without changing reservoir stage. |
| **What the scene may do** | Seed ROI bounds, inflow hydrograph, gauge sites, scenario steps, build/strike orders — **configure and drive** the real systems (same APIs UI / AI will use). | Hardcode stage outcomes, fake gauge balance, skip construction/damage funnels, or paint depth from narrative stage names. |
| **How Accept is earned** | Events emerge from physics + ECS authority: close outlets → level rises; intake submerged → cooling ok; breach → stage falls → coolers trip; inundation → stands drown; etc. | Special-casing to force matrix rows green, or reading the fixture as runtime water truth. |
| **Reusability test** | Same water / barrier / intake / damage path works for another valley, a cooling-only dam (no hydro), or a mid-game player dam. | APIs that only make sense inside `scenario/dam_flood/*` narrative helpers (harness may configure; must not own truth). |
| **Honest exit** | ADOPTED only with a production `file:line` caller on the **general** path (`systems/water`, construction, power cooling intake, …). | Marking DF-2+ done because a dam harness stub “looks wet.” |

**Fixture role (D3):** `dam_valley_v1.ron` fixes **topology and placements** so the benchmark is repeatable. Seed only varies noise. The fixture is input to world-gen / scenario / ROI **request** — **never** a substitute for the surface-water solve, breach mechanics, or ecology phases.

**Smell test:** if removing `TestScene::DamFlood` would require deleting the water solver, reservoir book, or cooling-intake path, those were cheats. The scene goes away; a player can still place a dam, fill a reservoir, and hook any cooler to it.

---

## 1. The scene (what the operator should see)

| Stage | Game time (approx.) | What happens | What must be visible |
|:---|:---|:---|:---|
| **S0 River** | 0 → steady | A river flows down a valley: forest on the floodplain, a town upstream, a downstream forest and road bridge, a thermal (or nuclear) plant on the valley side. | River width and flow animation match the discharge. Gauges G1 (upstream), G2 (dam site) and G3 (downstream) agree within mass balance. |
| **S1 Build** | t₁ | A dam is committed across the narrows (construction funnel). Hydro at the toe is **optional** in the product; the example may include it. Reservoir formation is the required outcome once outlets close. | Dam massing spans both banks; river passes diversion/outlet while building. |
| **S2 Impound** | t₁ → full (days) | Outlets close. The reservoir fills behind the dam, and downstream flow drops to the minimum release. | Water rises over the floodplain. **Submerged trees are hidden**, not deleted. The downstream river visibly narrows and slows. |
| **S3 Drown** | +2–4 weeks submerged | Trees under water past their species tolerance die (Drowned phase). They keep their wood as fuel, saturated. | Nothing new while submerged (hidden). Ledger counts move Live → Drowned. |
| **S4 Operate** | full | Reservoir at full stage. Hooked industrial intakes are submerged → coolers at rating. If hydro is present, it generates from head × flow. Operator may open spillway / bottom outlet in steps. | Spillway jet scales with gate opening when used. Downstream flow tracks releases; reservoir stage responds. |
| **S5 Strike** | t₅ | A missile from the launch site hits the dam with one of the warhead yields (variant). Where it hits matters: crest, mid-face, base or abutment. | Impact VFX, a crater carved in the dam voxels at the hit point, and a damage stage change. |
| **S6 Leak** | t₅ → collapse | Water flows through the breach (orifice or notch). The flow erodes the voxels around the hole (piping / overtopping), so the leak grows. Integrity falls Scarred → Compromised. | A jet from the hole that grows over time, and the dam visual state changes. |
| **S7 Collapse** | t₇ | Integrity crosses the collapse threshold. The breach widens to the failure geometry, and a flood wave goes downstream. | Wave front inundates the floodplain and cuts the road. Reservoir stage falls → hooked coolers derate/trip; optional hydro trips. |
| **S8 Recede** | t₇ → new equilibrium | The reservoir drains to a river through the breach. The flood recedes downstream. | The old lake bed is exposed. **Drowned trees reappear as dead standing trunks** with their own sprite and voxel model, not the burnt snag. |
| **S9 Dry + burn** | +days → weeks | Exposed dead wood dries according to the weather (timelag fuel moisture). A fire is ignited on the exposed bed. | The fire spreads faster through the dried dead stand than through the live forest next to it, and leaves Burnt snags. |

**Variants** (benchmark matrix): `yield ∈ {small, medium, large}` × `hit ∈ {crest, mid_face, base, abutment}` × `seed ∈ {1,2,3}`, plus `no_strike`, which keeps the dam intact and exercises operations only. Each variant has its own expected envelope; for example, a crest hit with the small yield must **not** collapse.

---

## 2. Operator decisions (SIGNED 2026-11-16)

| # | Decision | Locked | Why |
|:---|:---|:---|:---|
| D1 | **Water solver runtime** | **CPU sim in a region of interest (ROI), GPU for visuals.** A measured gate decides whether to move the solver to the GPU. | Flood depth decides tree death, power, routes and damage, so it is seeded sim truth. Async GPU readback would make seeded runs depend on frame timing (the same rule as dense heat, `plan_dense_heat_wgsl_v1.md` Review). The estimate fits easily on CPU: a 256×256-tile ROI = 65k cells, ~20 flops per cell per sub-step, ~4 CFL sub-steps per 1 s sim step at 70 m depth ≈ 5 Mflop per step. **Gate:** if `water.solver_ms_p95` exceeds the budget in §7 on the benchmark machine, open a GPU-solver slice (deterministic, fixed order, readback stamped like CQ-50). |
| D2 | **Dam type** | **Earth-fill embankment with a concrete spillway and bottom outlet** first; a concrete gravity dam later. | Earth dams leak, pipe and erode progressively before collapsing, which is the leak → deterioration → collapse you described. Concrete gravity dams usually fail suddenly by sliding or overturning blocks. Voxel material decides behaviour, so both can share one body model. |
| D3 | **Scene world** | **Authored valley fixture** (procedural, parameter RON), not a random seed. | It gives repeatable gauges, dam site, plant intake and launch-site geometry. Seeds vary only noise (forest, rain), not topology. |
| D4 | **Time compression** | **Coarse-step "time skip" for slow domains** (level-pool reservoir, ecology, fuel drying, weather means) during S2/S3/S8/S9. Fast domains (2D water, fire, ballistics) stay at 1 s steps during S4–S7. | At the 30-steps/frame cap (≈1800× at 60 fps) a game day costs ~48 s real, so S2–S9 would take hours. A benchmark must finish in minutes (see §7). |
| D5 | **Warhead set** | Three yields in `assets/configs/combat/warheads_v1.ron`: **small** (~250 kg HE class), **medium** (~500 kg class), **large** (~1000 kg penetrator class), each with blast radius, peak damage and penetration depth in voxels. | Real damage depends on yield, standoff and penetration. Numbers are tuning, with order-of-magnitude realism. |
| D6 | **Cooling consumer in the example** | Benchmark hooks the existing `nuclear_plant_pwr.json` cooling path to the **reservoir** (most demanding). Architecture must still allow **any** industrial cooler to attach the same way (coal, chemical, …). | Nuclear is the stress case; cooling intake is a general hook, not nuclear-only. |

---

## 3. Gap matrix — what fails the benchmark today (the upgrade list)

This is from a read-only survey of the code on 2026-11-16. **Every row is an upgrade slice in §5.**

| Domain | Today | Fails stage | Upgrade slice |
|:---|:---|:---|:---|
| **Water state** | `WaterClass` enum per tile, written once at world-gen (`water_class.rs:91`, `p4_hydrology.rs:73`). No depth, no surface height. Substrate `HydrologyState.water_depth` is a moisture placeholder (`substrate/hydrate.rs:34-50`). | S0–S8 | DF-2 |
| **Water dynamics** | No solver. `hydrology_background_tick_system` is a stub. `DamBreach` / `FloodSolve` hooks only mark chunks dirty (`substrate/hydrology/drain.rs`, `active_runtime.rs:86`), and `deep_solve_wired` falsely reports true (`substrate/hydrology/mod.rs:82`). | S0–S8 | DF-2 |
| **Water → consumers** | Changing extent needs full-map rebuilds: the stamp → passes 3/4, the dense-cache carve is one-way `min()`, the catalog is rebuilt, and pass 4 re-applies erosion (`p4_hydrology.rs:92-94`). Gameplay checks use the **render catalog**, not sim water (`build_interaction.rs:53`, `town.rs:333`, `site/systems.rs:73`). | S2, S7, S8 | DF-2, DF-3 |
| **Water render** | Rivers are fixed world-gen polylines; lake surface = one height per body from its rim (`pixel_world/field.rs:114-268`). No per-cell flow, so flow amount and speed can't be seen. | S0, S2, S4, S6–S8 | DF-3 |
| **Water save** | Not saved; regenerated from the seed (`io/save/dto.rs:19-30`, `manifest.rs:11`). | all (save mid-scene) | DF-2 |
| **Dam structure** | No archetype, building, gates, outlets or penstock. Buildings are blocked on water except port/oil-rig tokens (`preview_commit_guard.rs:36-57`). Linear placement is a fixed 4×1 wall. | S1, S4 | DF-4 |
| **Structure damage** | No per-voxel or per-region health. `VoxelOccupationBook` is one column per tile with a z-count. Blast is all-or-nothing: ≥50 damage on any footprint tile demolishes the whole site (`blast.rs:144-154`). `DestructibleDamageStage` is unused (CQ-57d). | S5–S7 | DF-8 |
| **Missiles** | No yield (damage hardcoded per kind), fixed radius 2.5. The launch "site" is a formation tile, targeting = `AirOpZone` centroid only (`missile_supply.rs:118-185`). Metres and tiles are mixed (`ballistics.rs`, `blast.rs:81`). Wind (CQ-56) and terrain hits (CQ-55) are unwired. | S5 | DF-7 |
| **Scenario timeline** | Steps are only fire/effect; one step per frame; `SimAdvance` doesn't wait; `when_tick_min` is never read (`scenario_steps.rs:36-65`, `script_host.rs:118-146`). | S1, S4, S5, S9 | DF-1 |
| **Trees + water** | Runtime water would reset a cell to `Bare`, `full_cap = 0`, which **deletes** the trees (`cell_stand.rs:610,624-633`). No drowned or dead phase. Burnt snags carry **zero** fuel. | S2, S3, S8, S9 | DF-5 |
| **Fuel moisture** | Per-cell `moisture` is static world-gen; fire reads it plus chunk rain. No per-cell dead-fuel moisture or drying clock. | S9 | DF-5 |
| **Dead-tree visuals** | Only the burnt snag exists (`world_map_vegetation_mesh.rs:591-598`, `pixel_world/layers.rs:133-139`). Wet cells are skipped by the site sampler (`footprint_wet`). | S8 | DF-5 |
| **Power** | Output = load × efficiency × status; no head, flow, fuel or cooling input (`power/systems.rs:206-247`). Nothing reads `current_output`, and grid supply comes from static capacity. Nuclear `cooling_available` ignores water (`failure_modes.rs:306-357,435-446`). The hydro definition has head/flow fields that nothing reads; there is no hydro or dam building. | S4, S7 | DF-6 |
| **Logistics** | Pathfinding ignores water; flooded roads stay usable. | S7 | DF-9 |
| **Time scale** | Max ≈1800× at 60 fps; no coarse-step path for slow domains. | S2, S3, S8, S9 runtime | DF-1 (time skip) |
| **Review path** | No water, dam, power-output or drowned-tree counters in the dump; no scene. | all | DF-1, DF-10 |

---

## 4. Architecture (authority map — sole writers)

```text
⊚ValleyFixture (RON) ═▶ world-gen terrain + base WaterClass (as today)            [world-gen / example scene only]
⊚SurfaceWaterField  ═▶ depth[m], surface[m], qx/qy[m³/s], wet_since[s]  (ROI)      [`systems/water` — SOLE WRITER]
   ◂ SurfaceWaterRoiRequest (player build, scenario, or DamFlood seeder — same API)
   ◂ BarrierHydraulics (crest / outlet / spillway) + BarrierControlIntent           [read-only inputs]
⊚ReservoirLevelPool ═▶ stage[m], storage[m³] — intact barrier + slow fill          [same water owner]
⊚WaterClassRuntime  ═▶ derived class per tile = f(base class, depth, hysteresis)  [water projection]
   ▸ substrate HydrologyState.water_depth ◂ projection (retire moisture placeholder)
⊚DamBody (voxel)    ═▶ per-voxel material + integrity; crest profile per tile        [construction damage funnel — SOLE WRITER]
⊚BarrierControlIntent ═▶ gate / outlet / (optional) turbine setpoint               [ops → water]
⊚CoolingIntake     ═▶ site → reservoir/river sample (min stage, flow demand)       [industrial hook — any cooler]
   ▸ reads ReservoirLevelPool / SurfaceWaterField (not DamFlood narrative)
⊚ChunkCellStand     ═▶ + Submerged / Drowned / Dead phases, wood fuel kept          [ecology — SOLE WRITER]
⊚CellFuelMoisture   ═▶ dead-fuel moisture per cell (timelag)                        [ecology — SOLE WRITER]
⊚Power plant state  ═▶ optional hydro P=ηρgQH; thermal/nuclear from CoolingIntake   [power — SOLE WRITER]
Render / UI ◂ snapshots only (water surface/flow textures on GPU, dam visual stage, dead-tree codes)
⛔ GPU readback feeding sim truth · ⛔ render catalog as gameplay water truth · ⛔ deleting trees on flood
⛔ cooling that bypasses reservoir/river water truth · ⛔ “power dam” as the only legal dam use
```

**Water model (DF-2).** A hybrid that is standard practice in hydraulic modelling:
- **Reservoir while the dam is intact and in slow phases:** *level-pool routing*. The surface is flat; `dS/dt = Q_in − Q_out(stage)`, where `Q_out` = spillway weir + outlets + turbine + leak. Inundation = cells whose ground is below the stage (cheap, exact for slow fill).
- **Everywhere else and in fast phases:** a 2D *virtual-pipes* shallow-water model on the tile grid inside the ROI (river corridor + valley + downstream). It runs at a CFL-limited sub-step `dt ≤ 0.5·Δx/√(g·h_max)` (Δx = 16 m; ≈ 0.31 s at 70 m depth, so ~4 sub-steps per sim step). The level pool hands over to 2D when the breach opens.
- **Mass conservation is a hard invariant**: the dump tracks `water.mass_error_frac`.
- **Hydraulic formulas** (SI):
  - Spillway or overtopping weir: `Q = C·L·H^1.5`, C ≈ 1.7.
  - Orifice / leak / bottom outlet: `Q = C_d·A·√(2gh)`, C_d ≈ 0.6.
  - Turbine: `Q_t ≤ Q_design`.
- **Breach growth** (DF-8): voxels next to the breach lose integrity at a rate ∝ (flow velocity − critical erosion velocity) for earth fill. Concrete voxels resist until stage thresholds.

---

## 5. Phases (each ends with behavioural tests + matrix rows; ADOPTED only with a production caller `file:line`)

### DF-0 — Sign charter
- **Do:** operator signs §2 D1–D6.
- **Accept:** this file's Status → SIGNED.

### DF-1 — Scene, fixture, timeline, gauges, time skip (benchmark harness first)
- **Do:**
  1. **Valley fixture.** `assets/configs/scenes/dam_valley_v1.ron` holds valley profile, narrows location, floodplain, side gully, river inflow hydrograph (base flow ≈ 150 m³/s, configurable), forest / town / plant / launch-site placements and gauge positions. The world-gen terrain override goes through the existing pipeline.
  2. **Scene.** `TestScene::DamFlood` (`--test dam`, `launch_args.rs`), with a seeder in the style of `seed_playquad_ne_build_order_once` (`test_harness.rs:1512`) committing via `CommitConstructionSiteEvent`.
  3. **Timeline steps** keyed by **sim tick or game time**:
     - New steps `CommitSite`, `SetDamControl`, `StrikeOrder`, `IgniteAt` (exists), `SetTimeSkip`.
     - `script_host` waits on `when_tick_min` instead of popping one step per frame.
     - `SimAdvance` blocks until done.
  4. **Gauges:** one `GaugeBook` kernel. Each gauge holds a cross-section line and reports discharge, stage and velocity. **One kernel with typed gauge sites, not a clone per gauge** (anti-clone rule).
  5. **Time skip:** slow domains (level pool, ecology phase clocks, fuel moisture, weather means) integrate with a coarse `dt_slow` (e.g. 600 s) when `SetTimeSkip` is on. Fast domains must be quiescent (assert).
  6. **Dump:** a `dam_flood` block (counts only) with stage, gauges, water mass, inundated tiles, dam integrity, outlet flows, stand phase counts, fuel moisture means, plant outputs, solver ms, and each stage's entry tick.
- **Accept:**
  - Lib tests: `dam_scene_timeline_*` (steps fire at the right tick), `gauge_kernel_*`, `time_skip_quiescent_*`.
  - Matrix rows `dam.*` exist with `when_scene: ["DamFlood"]`.
  - **Baseline v0 run recorded with every DF row failing**: this proves the benchmark detects missing systems.
- **Ledger (2026-11-16):** DF-1 shipped — `TestScene::DamFlood` / `--test dam` · `assets/configs/scenes/dam_valley_v1.ron` · `seed_dam_flood_valley_once` · scenario steps + `WaitUntilTick` / `SimAdvance` block · `GaugeBook` · `SlowDomainTimeSkip` · `sim_run_dump_dam_flood` · `playcheck.py --scene dam` · matrix `dam.*`. Callers: `src/engine/test_harness.rs` (`seed_dam_flood_valley_once`), **`src/scenario/dam_flood/`** (harness — not a sim domain), `src/dev/sim_run_dump_dam_flood.rs`. Baseline v0 score artifact: `tools/sim_review/expected/baseline_dam_v0_note.md` + scored dump under `debug_runs/sim_review/`.
- **Effort:** M

### DF-2 — Surface water solver (sim truth) + save
- **Do (general systems — §0 applies):**
  1. Own `SurfaceWaterField` + level-pool + virtual-pipes under a **general** water/hydrology module (not behind `if DamFlood`). ROI + inflow are **parameters** any scene/scenario can set.
  2. DamFlood / `dam_valley_v1.ron` only **seeds** that API (bounds, baseflow, gauge sites). No narrative stage → paint depth. Barriers come from dam/construction hydraulics inputs when those exist — not from fixture story flags.
  3. Inflow/outflow boundaries; rain-runoff from `WeatherField` (mass-balance counted).
  4. Projection: `WaterClassRuntime` (base class + depth hysteresis) → substrate `water_depth`. Retire moisture placeholder; `@cleanup-intelligence` on dead stubs.
  5. **Gameplay water checks read `WaterClassRuntime`**, not the render catalog (`build_interaction`, `town`, `site/systems`, `faction_step`).
  6. **Dirty regions** only; no erosion re-stamp on water update (`p4_hydrology`).
  7. **Save** overlay (depth + flux + level-pool) + round-trip test — works for any wired ROI.
  8. Honest `deep_solve_wired` (= surface field wired), or remove the flag.
- **Accept:**
  - Lib tests on the **general** solver (no `DamFlood` required): `water_mass_conserved_*`, `level_pool_fill_matches_volume_*`, `dam_break_wave_*` (Ritter), `water_save_roundtrip_*`.
  - DamFlood matrix rows pass because the **same** solver + gauges were configured by the example scene — not because of a dam-only branch.
  - Matrix: `dam.s0.gauge_balance_err < 0.02`, `dam.s2.downstream_ratio` ≈ min release / inflow (±10%) after real impound control.
- **Runtime:** CPU (D1).
- **Effort:** L
- **Correction (2026-11-16):** early DF-2 draft that gated solve/seed on `TestScene::DamFlood` / fixture narrative is **rejected** under §0. Relocate to general water ownership; dam scene = config only.
- **Ledger (2026-11-16):** DF-2 sim core shipped on general path — `systems/water/{surface_water,solver_tick,barrier_ops,hydro,cooling_intake}.rs` · ROI request · barrier install/breach · Ritter ±10% · level-pool fill · `io/save/surface_water_overlay.rs` · gameplay water via `WaterClassRuntime` (`build_interaction`, `town`, `faction_step`). Callers: `WaterSimPlugin`, `seed_surface_water_from_request_system`, `apply_barrier_*_request_system`. Matrix Accept: `dam.water_solver_wired` / `s0_gauge_balance` / `s2_downstream_ratio` (latched).

### DF-3 — Water visuals (GPU, render-only)
- **Do:**
  1. Per-cell surface height + velocity → a GPU texture uploaded per sim step (extract through the projection graph; no parallel hydro query in render).
  2. Water shader: surface at the field height, flow-map scrolling and foam from speed/turbulence, shoreline wetting. This replaces the per-body rim heights in `pixel_world/field.rs` and the static river ribbons inside the ROI.
  3. Emitters on the existing `gpu_water_particles` at spillway, outlet, breach jet and wave front, with intensity from flow.
  4. 2D tactical raster: depth tint + flow arrows on the existing overlay.
  5. Dirty-region repaint only.
- **Accept:**
  - Captures at S0/S2/S4 (gates 0/50/100%)/S6/S7/S8 in both views.
  - Matrix: `render.water_upload_ms_p95` within budget.
  - Visual review against the §1 "must be visible" column.
- **Runtime:** GPU
- **Effort:** M–L

### DF-4 — Dam structure, placement, control
- **Stance (not optional):** a dam is a **hydraulic structure** whose form follows **geology, topography, and hydrology**. It is **not** a power building. Hydro (penstock + turbine plant) is an **optional consumer** of the same reservoir head — same rule as cooling intakes (§0). Catalog / picker must not live under “Power plants.”
- **Dam kinds (catalog + `DamKind` on `DamBody`):**
  | Kind | Role | Site suitability (placement gates) |
  |:---|:---|:---|
  | **Embankment** | Earth/rock fill; sloping faces; length ≫ height | Wide valleys, fill borrow available; soft foundations OK with core |
  | **Gravity** | Concrete/masonry/earth weight resists overturning; **crib** = timber chambers filled with stone | Competent foundation; moderate canyons |
  | **Arch** | Masonry/concrete arched upstream; load into abutments | Narrow rock canyon with sound abutments both ends |
  | **Buttress** | Watertight upstream face + downstream buttresses | Intermediate valleys; less mass than gravity |
  | **Weir** | Low rise across channel for intake / fish pond / diversion | Existing channel; small head; still a barrier to the water solve |
  | **Coffer** | Temporary pumped enclosure for construction | Staging only; removed or flooded when permanent works finish |
- **Do:**
  1. `SiteArchetype::Dam` + per-kind catalogs (start: `dam_embankment_v1.json`, then gravity/weir; arch/buttress/coffer as follow-ons). Crest elevation, drag-span length, spillway/outlet params. **Optional** `penstock_to` / linked hydro site — never required for a valid dam.
  2. **Placement validation:** span water + bank tie-in (kind-specific: arch requires abutment rock class; embankment checks borrow/foundation softness later). `building_may_occupy_water` for dam/weir/coffer tokens.
  3. **Construction staging:** coffer/diversion (outlet open) → permanent works → fill → close. Coffer is a temporary site, not the reservoir dam.
  4. **`DamBody`:** `DamKind` + crest profile per tile → water barrier; voxel integrity/material by kind (clay core vs concrete arch vs weir sill).
  5. **`DamControl`:** gates + outlet always; **turbine setpoint only if a hydro plant is linked**. Play UI: reservoir stage, inflow/outflow, integrity — MW only when hydro present.
- **Accept:**
  - `dam_spans_banks_validation_*`, `dam_kind_site_suitability_*` (arch rejects soft wide valley; weir allows low span).
  - `dam_barrier_profile_*`, `spillway_weir_flow_*` / `orifice_flow_*` (±2%).
  - Matrix: `dam.s4.release_follows_gates`; cooling-only dam (no hydro) still fills and trips intakes on breach.
- **Effort:** L

### DF-5 — Drowned trees, dead fuel, fuel moisture, visuals
- **Do:**
  1. `ChunkCellStand`:
     - Runtime flooding sets **`submerged_since`** instead of resetting to Bare.
     - **`StandPhase::Drowned`** after the species flood tolerance (`veg_species_v1.ron`: e.g. upland conifer ~14 d, broadleaf ~21–28 d, willow/alder ≫ 60 d). These are tuning values; cite sources in the RON comment.
     - Then **`Dead` standing** once exposed.
     - Dead keeps `fuel_cap` = woody fuel (crown fines drop over time). Burnt stays 0.
     - **World-gen water keeps today's behaviour** (base class).
  2. **`CellFuelMoisture`:** dead-fuel moisture per cell with timelag classes (fine 1–10 h, branches 100 h, trunks 1000 h). It equilibrates to the weather's humidity and temperature, goes to saturation while submerged, and rain wets it. **Fire ignition and spread read it** (`cell_spread.rs`, `chunk_fire_overlay.rs`) next to the existing gates.
  3. **Visuals:**
     - Submerged cells hidden when depth > crown height (the sampler skips them); tops show in shallow water.
     - New render code `CELL_DEAD` + a bleached dead-trunk sprite/voxel (no crown, grey), distinct from the charred snag.
     - Assets through the MCP pipeline (`@designer-mcp`: seeded, procedural, no AI art).
  4. **Save:** the stand overlay carries phase, `submerged_since` and fuel moisture (coordinate with CQ-01).
- **Accept:**
  - `flood_does_not_delete_stand_*`, `drowned_after_tolerance_*`.
  - `dead_wood_keeps_fuel_*`, `fuel_moisture_dries_with_timelag_*`.
  - `fire_spreads_faster_in_dry_dead_than_live_*`.
  - Matrix: `dam.s3.drowned_cells > 0`, `dam.s8.dead_exposed_cells ≈ drowned_cells (±5%)`, `dam.s9.burn_rate_dead / burn_rate_live > 1.5`.
- **Effort:** L

### DF-6 — Reservoir consumers: cooling hooks + optional hydro
- **Do (§0 reservoir-first):**
  1. **`CoolingIntake` (general):** any industrial site that needs cooling declares intake tile / linked reservoir, min stage (or depth), and demand. Reads `ReservoirLevelPool` / `SurfaceWaterField` — **not** DamFlood stage names. Nuclear in the example is one consumer; coal/chemical/etc. must be able to use the same component.
  2. **Hydro (optional):** `P = η·ρ·g·Q_t·H_net` with η from the definition and `H_net = stage − tailwater − losses`. Turbine flow limited by setpoint, design flow, and available water. A dam with **no** hydro is still valid (reservoir + coolers only).
  3. **`current_output` feeds grid supply** where hydro exists (today nothing reads it).
  4. **Cooling from reservoir:** intake elevation + minimum submergence against reservoir stage.
     - Required withdrawal: `Q_cool = P_reject / (ρ·c_p·ΔT)`, so ~48 m³/s for ~2 GW rejected heat at ΔT = 10 K.
     - Below the minimum level, or with less available flow → derate. Below the trip level → trip.
     - Nuclear (example consumer) joins `cooling_available` (`failure_modes.rs:435-446`) next to offsite power and diesels — **same intake API** as any other cooler.
  5. **Dump block:** per-site cooling state + optional hydro output/head/flow.
  6. **Destruction path:** when breach drains the reservoir, hooked intakes see low stage and derate/trip without a dam-special case.
- **Accept:**
  - `cooling_intake_reads_reservoir_*`, `cooling_derates_on_low_stage_*`, `nuclear_cooling_needs_water_*` (example), `second_industry_can_share_reservoir_*` (fixture or unit).
  - Optional hydro: `hydro_power_formula_*` (±3%) when a turbine exists.
  - Matrix: cooling ok at S4 when stage ≥ intake min; after S7 drain, thermal/cooler `∈ {derated, tripped}`; hydro MW → 0 if present.
- **Effort:** M

### DF-7 — Launch site, strike orders, warheads
- **Do:**
  1. **Units first:** metres in all of `ballistics.rs` / `blast.rs`; fix the tile/metre mix and the 16-vs-32 chunk constant (`blast.rs:179`).
  2. A `missile_launch_site` building with a magazine (`MissilePackage` stock).
  3. A `StrikeOrder { target: Point | Structure{site, local_offset}, warhead }` through the existing order funnel.
  4. `warheads_v1.ron` (D5).
  5. Use `launch_with_wind` (CQ-56) and swept terrain/structure hits (CQ-55).
  6. Impact produces a `BlastIntent { center_m, yield → radius, peak, penetration }`.
- **Accept:**
  - `strike_hits_structure_offset_*` (CEP within tuning), `warhead_yield_scales_radius_*`, `blast_units_metres_*`.
  - Matrix: `dam.s5.impact_err_m < cep`.
- **Effort:** M

### DF-8 — Voxel damage → leak → deterioration → collapse
- **Do:**
  1. **Blast execute** carves `DamBody` voxels: a sphere or cone by yield and penetration, with integrity falloff. This is the construction funnel, the single writer.
  2. **The aggregate drives `DestructibleDamageStage`** (CQ-57d): Intact → Scarred → Compromised → Collapsed.
  3. **Leak:** an opening below the stage gives orifice/notch flow into DF-2.
  4. **Erosion:** voxels next to the flow lose integrity by velocity (earth fill erodes; concrete only past thresholds). This makes piping and overtopping grow.
  5. **Collapse:** when the stage reaches Collapsed, or the breach exceeds the failure geometry, the barrier drops to the breach invert. `HydrologyDirtyReason::DamBreach` is emitted (finally used). Collapse uses `execute_demolish_at_tile` only for the terminal state of whole-site removal; partial collapse is voxel-level.
  6. **Hydro and outlet ops** derate with the stage.
- **Accept:**
  - `blast_carves_voxels_by_yield_*`, `leak_grows_by_erosion_*`, `collapse_emits_dambreach_*`, `crest_small_yield_no_collapse_*`.
  - Matrix plausibility: peak breach discharge within **×0.5–2** of Froehlich (1995), `Q_p = 0.607·V_w^0.295·H_w^1.24` (m³/s, V_w in m³, H_w in m). Variant envelopes: time-to-collapse ordering large/base < medium/mid_face < small/crest (no collapse).
- **Effort:** L

### DF-9 — Downstream consequences + recession
- **Do:**
  1. Roads and rail are impassable at depth > 0.5 m or velocity > 2 m/s, via `LogisticsRouteDirtyTiles` and the single route cost (CQ-53).
  2. Sites in the flood get `operational_readiness` reduced by flood depth (through the existing writer, not a new one).
  3. Short floods wet the downstream stands but don't drown them.
  4. Recession: the field drains to the post-breach equilibrium, and exposed cells move to Dead with saturated fuel moisture.
- **Accept:**
  - `flooded_road_blocks_route_*`, `short_flood_does_not_drown_*`.
  - Matrix: `dam.s7.wave_arrival_g3_s` within the variant envelope; `dam.s8.inundated_tiles` falls to < 10% of peak.
- **Effort:** M

### DF-10 — Lock the benchmark (baseline v1)
- **Do:**
  1. Fill every `dam.*` matrix row with thresholds from §1/§5. Add perf rows (§7).
  2. `playcheck.py --scene dam --variants all --seeds 1,2,3` runs the variant matrix and captures.
  3. Commit the first fully green scored run as `tools/sim_review/expected/baseline_dam_v1.json`. Later runs score **"no worse than baseline"** on perf and on plausibility error.
- **Accept:** a green matrix on all variants; the dashboard shows each stage with captures. This file's Status → BASELINE v1.
- **Effort:** S–M

---

## 6. Dependency graph

```text
cleanup prerequisites (§9) ─▶ DF-0 ─▶ DF-1 ─▶ DF-2 ─┬─▶ DF-3 (visuals)
                                              ├─▶ DF-4 ─▶ DF-6 (power)
                                              │     └──▶ DF-8 ◀── DF-7 (strike)
                                              ├─▶ DF-5 (trees/fuel)
                                              └─▶ DF-9 ◀── DF-8
                                   all ─▶ DF-10
```

**Parallel after DF-2:** DF-3 (render), DF-5 (ecology), DF-7 (combat) and DF-4 (construction) touch disjoint files. DF-8 needs DF-4 + DF-7; DF-6 needs DF-4.

---

## 7. Benchmark budgets (release, benchmark machine)

| Row | Budget |
|:---|:---|
| `water.solver_ms_p95` (256² ROI, fast phase) | ≤ 2.0 ms per sim step. Above this → open the GPU-solver slice (D1). |
| `render.water_upload_ms_p95` | ≤ 0.5 ms per frame |
| `frame_ms_p95` during S7 flood | ≤ 16.7 ms at 1× speed |
| Whole scene (one variant, time skip on) | ≤ 5 min real |
| `water.mass_error_frac` (whole run) | ≤ 0.5% |
| Determinism | Same seed + variant → identical dump `dam.*` counts (bit-exact on CPU sim) |

---

## 8. Rejects

- Deleting or resetting stands to Bare on runtime flooding.
- The render catalog as gameplay water truth; a second water writer; GPU readback feeding the water, tree, power or route sim.
- Whole-site demolish as the "damage model" for the dam; a per-gauge clone of the gauge system.
- Witness JSON, `*_green` predicates, or Accept tests that only grep source.
- Marking a DF slice ADOPTED without a production caller `file:line` (honest exits, `plan_ref_books_upgrade_v3.md` §0.1).

---

## 9. Prerequisites from the cleanup plan

| Need | Cleanup item |
|:---|:---|
| Lib tests green, so the benchmark tests mean something | CQ-20, CQ-21 |
| Fire / stand save correct (stand overlay will grow) | CQ-01 |
| Terrain erosion conserves mass (the valley fixture depends on it) | CQ-04 |
| Voxel occupation saved + deterministic | CQ-07 |
| Projectile terrain/structure hits; missile wind | CQ-55, CQ-56 |
| Damage ladder wired | CQ-57d |
| Post-load rebuild real (water overlay joins it) | CQ-57h |
| Single route cost (flooded roads plug in) | CQ-02, CQ-53 |
| Fire hot-path copies fixed (S9 fire + flood frame budget) | CQ-60 |

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| 0.1.0 | 2026-11-16 | Charter draft from operator scene brief + code survey (water, structures/damage/missiles, ecology/power/review). Supersedes WSS HY-005/006. |
