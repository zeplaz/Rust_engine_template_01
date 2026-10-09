# PLAN-FRONT-COMBAT-v1 — fronts, theaters, pressure points, deployment, fortification, symbology, combat

| Field | Value |
|:---|:---|
| **Id** | **PLAN-FRONT-COMBAT-v1** |
| **Status** | **OPEN** — slices **FC-1 → FC-10** |
| **Date** | 2026-10-08 |
| **Owner** | fleet `front` (see `src/dev/fleet_charter_20261008_v1.md`) |
| **Code home** | new `src/combat/` (pure, headless, deterministic) + thin systems in `src/strategic/frontline.rs`, `src/strategic/zones.rs` |
| **Obeys** | `AGENTS.md` (attach to authoritative contracts), `src/dev/construction_invariants.md` (single execute funnel), `plan_attrition_systems_fit_v1.md` / `plan_vehicular_threat_fit_v1.md` (no second influence grid, no third artillery-danger writer) |
| **Units** | Metric. One map tile = `TILE_SIZE_M` (balance table, default **25 m**). Time = sim seconds from `SimTimeMicros`. |

## Summary

The game already has the raw fields a front needs — `ChunkStrategicOverlay.faction_control`,
`threat`, `artillery_danger`, `logistics_strength`, `mobility_cost` — and two writers that stamp
them (`apply_zones_to_strategic_overlays_system`, `apply_site_zone_emitters_to_overlays_system`).
It already has fortification **sites** (`DefenseKind` → `SiteArchetype::{TrenchLine, BunkerComplex,
DefensiveWall, DragonTeeth, Minefield}`) that go through the construction commit funnel, and a pair
stance book (`FactionStanceBook`, `(u8,u8) → Ally/Enemy/Neutral`).

What is missing:

1. The **front** is a chunk list (`FrontlineState.contested_chunks`) computed for slots 0/1 only.
   There is no cell-level front, no per-hostile-pair segment, no polyline to draw.
2. Overlay control **latches**: every writer adds and clamps, nothing relaxes, so a front can never
   move once stamped.
3. No **theater** partition, no **pressure points**, no **deployment orders**.
4. No **combat**: `src/systems/damage` is a TODO stub, `src/systems/collision` is empty, there are no
   military formations with strength / morale / supply.
5. No **faction insignia** and no map unit symbology.
6. No balance data.

This plan fills those gaps **without** a second control grid: formations project presence through the
existing `Zone` component (the zone pass is the control writer), the front is derived from the
overlay, and fortification protection is read from operational `ConstructionSite` rows.

```text
⊚Formation (+ Zone{Control}/Zone{Fire}) ─▶ apply_zones_to_strategic_overlays_system ═▶ ⊨ faction_control, threat, artillery_danger
⊚ControlRelaxation (per sim-second, in zones pass) ═▶ ⊨ faction_control decays toward 0 where no stamp
⊚derive_frontline_from_control_system ═▶ ⊨ FrontlineState{contested_chunks (unchanged), front cells, segments, polylines}
⊚theater pass ═▶ ⊨ TheaterBook (stable ids)
⊚pressure pass ═▶ ⊨ PressurePointBook (explained scores, top-K per theater per side)
⊚DeploymentOrderQueue ─▶ execute_deployment_orders (single funnel) ═▶ ⊨ Formation.posture/target
⊚FortificationIndex ⊰ ConstructionSite(Operational, defense archetype) + SiteFootprint
⊚combat_resolve ═▶ ⊨ Formation.strength / morale / supply ; CombatEventLog
⊚FrontDrawDemandBook ═▶ (read by logistics fleet)
⊚FactionInsigniaBook + UnitSymbol ─▶ map overlay painter (presentation only)
```

## Authority map (who writes what)

| Resource / field | Sole writer | Readers |
|:---|:---|:---|
| `ChunkStrategicOverlay.faction_control` | zone pass (`zones.rs`) incl. relaxation; site civil stamp (unchanged) | front derivation, settlement, network flow |
| `FrontlineState` | `derive_frontline_from_control_system` (`frontline.rs`) | theaters, settlement `faction_step`, `WorldReadSnapshot`, `site_sample`, map overlay |
| `TheaterBook` | `derive_theaters_system` (`src/combat/theater.rs`) | pressure pass, AI fleet, map overlay |
| `PressurePointBook` | `score_pressure_points_system` (`src/combat/pressure.rs`) | AI fleet, deployment UX, map overlay |
| `DeploymentOrderQueue` | UI / AI **enqueue only** | `execute_deployment_orders_system` |
| `Formation` (posture, target, tile) | `execute_deployment_orders_system`, `advance_formations_system` | combat, map overlay |
| `Formation` (strength, morale, supply) | `combat_resolve_system`, `formation_supply_system` | map overlay, witness |
| `FortificationIndex` | `rebuild_fortification_index_system` (derived from sites) | combat, pressure |
| `FrontDrawDemandBook` | `publish_front_draw_demands_system` | logistics fleet |
| `FrontBalance` | loader (RON) | everything in `src/combat/` |
| `FactionInsigniaBook` | loader (RON) / defaults | map overlay, HUD |

Construction: the front fleet never spawns a site. Fortify orders become `PendingConstructionQueue`
rows via the existing pending → validate → commit funnel (`construction_invariants.md` rules 1, 2, 7, 8).

## Slices

Each slice is one `[front]` checkpoint commit, `cargo check --lib` green plus the slice's filter.

### FC-1 — balance table + cell-level front derivation (sim)

- `src/combat/mod.rs`, `src/combat/balance.rs`: `FrontBalance` (serde, RON) with
  `tile_size_m`, front thresholds (`own_min`, `contest_band`), relaxation rate, theater params,
  pressure weights, combat coefficients, fortification profiles, unit-kind profiles.
  Default compiled in via `include_str!("../../assets/configs/combat/front_balance_v1.ron")`;
  file override at the same path when present. Validation fn returns an issue list
  (probabilities in 0..1, positive rates, every unit kind × kind has an effectiveness row).
- `src/combat/front.rs` (pure): `ControlSample` grid view over overlays → per-cell owner
  (`argmax` slot with `control ≥ own_min`), contested cells (two **hostile** slots ≥ `own_min` and
  `|Δ| < contest_band`), front cells (owned cell 4-adjacent to a hostile-owned or contested cell),
  boundary edges between hostile owners chained into **polylines** in tile space (sorted, deterministic).
  Hostility: `FactionStanceBook` `Enemy` pairs; with no book, slots 0 and 1 are hostile (today's behavior).
- `frontline.rs`: `FrontlineState` gains `front_cells`, `contested_cells`, `segments: Vec<FrontSegment>`
  (pair, cells, polylines, length_m), `revision`. `contested_chunks`, `stability`, `flux` keep
  their meaning so existing readers do not move.
- **Accept:** `cargo test -p proc_A_dine01 --lib combat::front` + `combat::balance`.

### FC-2 — control relaxation (front can move)

- `zones.rs`: before stamping, when `ControlRelaxation` resource is present and sim time advanced,
  multiply `faction_control` by `exp(-rate · dt_s)`. Absent resource ⇒ legacy latch (existing tests
  unchanged). Stamps then re-add, so held ground stays held and abandoned ground fades.
- **Accept:** `cargo test -p proc_A_dine01 --lib strategic::zones`.

### FC-3 — theaters (sim)

- `src/combat/theater.rs`: 8-connected components of front cells per hostile pair; components longer
  than `theater_max_len_m` split along their principal axis into sectors; area of responsibility =
  cells within `theater_depth_m` of the segment (both sides). `TheaterBook` keeps **stable ids** by
  best-overlap (Jaccard ≥ `theater_id_keep_overlap`) with the previous revision.
- **Accept:** `combat::theater` tests: two blobs → two theaters; long line splits; ids stable across a 1-cell shift.

### FC-4 — pressure points (sim, explainable)

- `src/combat/pressure.rs`: per theater, per side, score each front cell:
  `w_threat·enemy_threat + w_artillery·enemy_artillery + w_contest·(1−|margin|) + w_supply·(1−own_logistics)
   + w_mobility·(1−mobility_cost) − w_fort·own_fortification`.
  Two kinds: **Threatened** (defend here) and **Opportunity** (attack axis: enemy weak, low mobility cost).
  Non-max suppression with radius `pressure_nms_radius_m`, top-K per theater per side. Each point
  carries the driver breakdown (for `ai_explainability` and tooltips).
- **Accept:** `combat::pressure` tests: highest enemy threat wins Threatened; fortification lowers it; NMS keeps K.

### FC-5 — formations + combat resolution (sim)

- `src/combat/formation.rs`: `Formation { faction_slot, kind: UnitKind, echelon: Echelon, strength (0..1 of TOE),
  morale (0..1), supply (0..1), posture: Posture, tile, target }`. Kinds: Infantry, Mechanized, Armor,
  Artillery, Recon, Engineer, AirDefense. Each formation carries a `Zone{Control}` sized from the
  unit profile (radius m → tiles) × strength × morale; artillery also gets a child `Zone{Fire}`
  (no third threat writer — the zone pass stamps both).
- `src/combat/resolve.rs` (pure): engagements = hostile formation pairs within the attacker's
  range. Per combat step (`combat_step_s`), expected-value Lanchester losses:
  `loss_B = k_attrition · strength_A · fire_A · eff[A.kind][B.kind] · posture_mult · supply_factor(A)
            · morale_factor(A) · (1 − cover_B) · (1 − obstacle_slow?) · dt`.
  `cover_B` from `FortificationIndex` at B's tile + terrain; morale drops with losses
  (`morale_loss_per_strength`), recovers out of contact; `morale < break_threshold` ⇒ posture `Retreat`.
  No RNG: same inputs, same outcome. `CombatEventLog` ring buffer (bounded).
- **Accept:** `combat::resolve` tests: symmetric fight is symmetric; armor beats infantry in the open;
  infantry in a bunker beats the same armor; broken unit retreats; deterministic replay.

### FC-6 — deployment orders + draw demands (sim)

- `src/combat/orders.rs`: `DeploymentOrder { formation, kind: Move|Hold|Reinforce(pressure point)|Attack(tile)|Fortify(theater) , issued_by }`.
  `DeploymentOrderQueue` (enqueue only), `execute_deployment_orders_system` validates (formation alive,
  owner, target in bounds, not into own retreat) and is the **only** writer of posture/target.
  `advance_formations_system` moves at `speed_mps / (1 + mobility_cost·k)`.
- `FrontDrawDemandBook`: per formation, per resource class (`Ammunition`, `Fuel`, `Rations`, `Engineering`),
  demand in **kg/s** from unit profile × posture (combat posture draws more). Published for the
  `logistics` fleet; `supply` recovers when logistics reports delivery (`FrontSupplyDeliveries`), else
  decays at `supply_burn_per_s`. Until logistics wires deliveries, overlay `logistics_strength` at the
  formation tile is the fallback feed.
- **Accept:** `combat::orders` tests.

### FC-7 — fortification building + protection

- `src/combat/fortification.rs`: `FortificationProfile` per defense archetype in balance
  (`cover`, `obstacle_slow`, `mobility_cost_add`, `build_effort_person_hours`, `anti_armor`).
  `FortificationIndex` (tile → best profile, owner slot) rebuilt from Operational defense sites.
- **Fortify order:** for a theater/side, plan trench lines along the front polyline offset
  `fortify_setback_m` behind it, bunkers at Threatened pressure points, wire/dragon teeth on
  Opportunity axes of the enemy. Emit **pending construction rows** only (the construction funnel
  validates stock — `defense_staging_stock.rs`, `defense_concrete_stock.rs` — and commits).
- Barbed wire needs a `DefenseKind::Wire` + `SiteArchetype::WireObstacle`; those types live outside
  `front` ownership → board ask; until then wire profile is data-only.
- **Accept:** `combat::fortification` tests + planner emits rows that pass `allows_commit` in a headless app.

### FC-8 — faction insignia + APP-6-style unit symbology (data)

- `src/combat/symbology.rs`: `FactionInsignia { slot, name, short, primary_rgb, secondary_rgb, emblem: Emblem }`
  (emblems: Star, Roundel, Cross, Chevron, Eagle, Shield, Bar) with RON defaults for 16 slots.
- `UnitSymbol` from (viewer slot, formation): **frame** by affiliation (friend = rectangle,
  hostile = diamond, neutral = square, unknown = quatrefoil), **icon** by kind (infantry ✕, armor ellipse,
  mechanized ✕+ellipse, artillery dot, recon diagonal, engineer E-bar, air-defense dome), **echelon**
  marks above frame (squad •, section ••, platoon •••, company I, battalion II, regiment III, brigade X,
  division XX), **status** (strength bar, morale tint, retreat arrow). Output is a list of vector
  primitives in a unit box so egui, 2.5D sprites, or 3D billboards draw the same symbol.
- **Accept:** `combat::symbology` tests (friend/hostile frames differ; all kinds have icons; echelon counts).

### FC-9 — map presentation

- `src/gui/front_overlay.rs` (new): egui painter through the existing map projection — front polylines
  (FLOT style: faction-coloured line with teeth toward the enemy), contested hatching, theater outlines +
  labels, pressure markers (▲ threatened / ◆ opportunity, size by score), formation symbols from FC-8.
  Toggle key + legend. Presentation only: reads books, writes nothing in sim.
- Board asks: `graphics` (2.5D/3D billboard/decal path for the same primitives), `interaction`
  (placement UX for deploy / fortify orders, hit-testing formations).

### FC-10 — witness + headless scenario

- `src/combat/scenario.rs`: seeded two-faction scenario (headless app, no window): fronts derive,
  theaters + pressure points exist, orders move formations, a fortified defender holds, attacker breaks.
  Writes `debug_runs/front_combat_live.json` (`_agent_meta` envelope) with green/issue list.
- **Accept:** `cargo test -p proc_A_dine01 --lib combat::scenario`.

## Must not change

- No second control / influence / threat grid; overlay stays the field store.
- No new writer of `threat` / `artillery_danger` other than formations' `Zone` components through the
  existing zone pass.
- `contested_chunks` semantics stay (settlement + snapshot readers).
- No site spawn outside the construction funnel; fortify = pending rows.
- No `Transform` writes from sim; presentation reads only.
- RON for balance; metric units.

## Cross-fleet

| To | What |
|:---|:---|
| `logistics` | consume `FrontDrawDemandBook` (kg/s per class per formation); publish `FrontSupplyDeliveries` |
| `ai` | consume `TheaterBook`, `PressurePointBook`; enqueue `DeploymentOrder`s |
| `graphics` | render `UnitSymbol` primitives + front polylines on 2.5D/3D presentation |
| `interaction` | placement UX for deployment / fortify orders; `DefenseKind::Wire` |
| `legacy` / site owners | `SiteArchetype::WireObstacle` |
