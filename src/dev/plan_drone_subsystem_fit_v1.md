# PLAN-DRONE-SUBSYSTEM-FIT-v1 — drones as base-case fit (not a parallel stack)

| Field | Value |
|:---|:---|
| **Id** | **PLAN-DRONE-SUBSYSTEM-FIT-v1** |
| **Status** | **OPEN** — **D0–D8 SHIPPED** (2026-11-13) |
| **Date** | 2026-11-13 |
| **Owner** | `@coder` (sim/weather/combat/logistics), `@designer` (D7/D8 UX+LOD art) |
| **Cursor plan** | `~/.cursor/plans/drone_subsystem_fit_v1_e8f9a0b1.plan.md` |
| **Weather** | [`plan_weather_wind_v1.md`](plan_weather_wind_v1.md) §4e |
| **Obeys** | `AGENTS.md`, front combat authority, construction funnel, no second weather extract |

## Principle

Expand **weather / C2 / sensors / logistics / production / AD**, then fit drones onto those contracts. No `DroneSim` world.

Player never direct-controls airframes: draws optional **AirOpZone** / **DroneAO**, sets priorities and **supply distribution**; **MissionBook** automates sorties under envelopes.

## Authority map

| Resource / field | Sole writer | Readers |
|:---|:---|:---|
| `WeatherField` | weather step | envelopes, MissionBook, dump |
| Link / Sensor / Flight envelopes | pure fns from `sample` (+ smoke01) | MissionBook, resolve, C-UAS, dump |
| `AirOpsWeatherStatus` | `sync_air_ops_weather_status` | dump / AI / HUD |
| `MissionBook` / AO / AirOpZone (D1+) | one executor from enqueue queue | resolve, pressure, presentation |
| Formation packages (D1+) | combat / supply systems | resolve, front demand |
| Manufacturing / CargoKind (D4) | existing plant + order sim | front restock |
| Air presentation (D8) | render extract (read-only snap) | GPU / overlay — **must not write sim** |

## Role taxonomy (RON data)

| Role | Range | Job |
|:---|:---|:---|
| FpvStrike | Contact | Anti-infantry / armor / logistics hunt |
| ReconLoiter | Short–med | ISR into resolve/pressure |
| AttritCruise | Long | Deep strike / saturation |
| MedAttack | Medium | Stand-off |
| Interceptor | Short–med | Anti-drone / soft anti-missile |
| SwarmMember | Contact | Saturation (counts, not N entities) |

**C2:** `Radio` → `Fiber` → `SatRelay` → `Autonomy{L0..L3}`.  
**Sensors:** `EO`, `IR`, `Radar`, `PassiveRF`.

## Weather × C2 matrix (acceptance spine)

| Factor | Radio FPV | Fiber | Sat | Autonomy L2+ |
|:---|:---|:---|:---|:---|
| Wind | Flight abort | Flight abort | Flight abort | Flight abort |
| Vis / fog | Sensor + link | Sensor | Sensor | Sensor |
| Smoke | EO kill; IR partial | EO kill | EO kill | per kit |
| Lightning / EW proxy | Radio hard fail | Fiber resists RF | Sat partial | Autonomy resists C2 loss |

## Production ladder (one page)

```text
Raw (Al / composites / electronics / energetics)
  → Tier-1 plants (airframe, battery, seeker, warhead, fiber_draw)
    → uas_final_assembly
      → Depot / theater stock
        → FrontSupplyPoint demand (package TOE + attrition)
          → Formation packages
            → combat / weather abort / C-UAS loss → demand again
```

| Hop | Owner slice |
|:---|:---|
| CargoKind BOM tags | D4a |
| ManufacturingNode blueprints + catalog map | D4b |
| Haul (inherits W4 weather mul) | existing LO + D4a |
| Front demand publish | D4a + combat supply |
| Recipe gates / yield | D5 research data |

## Player interaction

| Does | Does not |
|:---|:---|
| Draw/edit AirOpZone & DroneAO | Direct-control airframes |
| Priority targets & mission weights | Path every sortie |
| Supply distribution across fronts/AOs | Micro mid-flight |
| Pause/cancel mission classes | Bypass weather envelopes |

Air ops = optional zones + pool allocate (base / off-map → on-map).  
Drones = AO + mission type/role mix + MissionBook automation.

## Presentation LOD (D8)

| Band | Display | Motion |
|:---|:---|:---|
| Far | AO tint + symbology/counts | Pulse only |
| Mid | Batched sprites/dots (hard cap) | Local orbits; wind lean |
| Near | Few animated sprites | Canned paths |
| Hero | Rare mesh/voxel (large air only) | Crossfade; budgeted |

**Default:** smaller + more numerous → simpler. Mass FPV/swarm stay marks/sprites; flight paths are **cosmetic** (wind-lean templates). Presentation never writes sim.

### Fidelity upgrade path (locked — do not paint into a corner)

Code: [`AirPresentationPolicy`](../systems/weather/air_presentation_policy.rs) + [`AirPresentationSortieHint`].

| Knob | Default | Future raise |
|:---|:---|:---|
| `mass_flight_fidelity` | `TemplateCosmetic` | `EnvelopeDriven` → `SimSynced` when MissionBook publishes path samples / sortie ticks |
| `scarce_flight_fidelity` | `EnvelopeDriven` | `SimSynced` |
| `mass_geometry_tier` | `MarksOrSprites` | `VoxelOrMesh` only if `allow_mass_voxel_upgrade` + `voxel_hero_budget` |
| `sprite_cap_per_view` / `near_animated_cap` / `voxel_hero_budget` | Conservative caps | `scale_mass_quality` / `enable_mass_voxel_upgrade` |

**D1 MissionBook must** emit additive presentation hints (`mission_id`, wind, heading, optional `path_samples_m`, `sortie_tick`) so D8 can climb fidelity without new sim writers or a rewrite. Empty path samples ⇒ templates still work.

## Phases

| Phase | Exit |
|:---|:---|
| **D0** | Link/Sensor/Flight APIs + Drone* RON + dump/tests green |
| **D1** | Packages + MissionBook/AO enqueue; auto-sortie stub; stop for review |
| **D2** | C2 modality + autonomy + resist muls |
| **D3** | AirDefense C-UAS vs cruise/swarm |
| **D4a/b** | BOM + plants + front coverage E2E |
| **D5** | Research data gates |
| **D6** | DroneDoctrine live; swarm as counts |
| **D7** | Zone/AO/mission/supply play UI |
| **D8** | LOD bands + caps |

## Rejected (defaults — not forever bans)

- Parallel `DroneSim` / second weather extract / presentation writing sim / RTS micro / drone-only factory ledger — **hard reject**.
- **Default-off:** per-drone voxels for mass FPV/swarm, and sim-synced flight for every sortie. These may be **scaled on later** via `AirPresentationPolicy` when budgets allow — never as the D8 launch path.

## D0 status

**D0–D8 SHIPPED 2026-11-13.**

| Phase | Landed |
|:---|:---|
| D0 | Link/sensor/flight envelopes + Drone* RON |
| D1 | MissionBook / AO / packages / auto-sortie → resolve mods |
| D2 | Autonomy level + research resist + modality gates |
| D3 | Interceptor / CUAS cuts `cruise_threat` |
| D4a | `DroneCargoDemandBook` → front `PublishDemand` |
| D4b | UAS manufacturing blueprints + catalog map ids (buildings deferred — see §Catalog) |
| D5 | `DroneResearchState` recipe/resist gates |
| D6 | `FactionDroneDoctrine` scales strike effects |
| D7 | Draw tool `U` + Shift+U quick AO + pause/remove + map outlines |
| D8 | `AirPresentationFrame` LOD caps from policy + hints |
| Smoke | `ChunkSmokeField`/L0 → `smoke01` into link/sensor (status + sorties) |

## Catalog buildings (UAS plants) — decision

**Do not** slap `uas_*` rows into the build picker until the industrial path is complete.

How plants work today:

```text
catalog JSON (assets/configs/buildings/*.json)
  → BuildingDefinitionRegistry + construction funnel
  → ConstructionSite → Operational
  → activate_industrial_facilities_system
       ├─ IndustrialSupplyChainRole / resource-flow membership
       └─ manufacturing_node_for_catalog(id) → ManufacturingNode
  → ManufacturingCorePlugin ticks throughput → output buffers → haul / front demand
```

What D4b already shipped: `mfg_uas_*` blueprints + string map (`uas_final_assembly`, `battery_cell_line`, `fiber_draw`, …). **No** matching catalog JSON, **no** `IndustrialSupplyChainRole` variants, **no** `industrial_supply_chains.json` hops, **no** art/footprint.

Correct next program (separate slice, not a picker hack):

1. Author a real UAS chain (tier-1 battery/fiber/seeker → final assembly) in `industrial_supply_chains.json`.
2. Add catalog buildings with footprint, power, consumes/produces — same shape as aluminum/concrete.
3. Extend `IndustrialSupplyChainRole` (or a parallel UAS role enum) + activation/runtime if the plant needs more than `ManufacturingNode`.
4. Only then expose in the play build picker / rail; until then blueprints remain for scenario spawn / tests / future activation.

Transitional OK: attach UAS `ManufacturingNode` to an existing fabrication hall via catalog alias — not a fake empty building.
