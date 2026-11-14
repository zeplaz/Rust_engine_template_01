# PLAN-PRODUCTION-RESTOCK-SPINE-v1 — capability → cargo → plant → haul → consumer

| Field | Value |
|:---|:---|
| **Id** | **PLAN-PRODUCTION-RESTOCK-SPINE-v1** |
| **Status** | **OPEN** — P0–P6 SHIPPED; residual DEFERs below |
| **Date** | 2026-11-13 |
| **Cursor plan** | `production_spine_charter_48007c26.plan.md` |
| **Obeys** | `AGENTS.md`, construction funnel, one writer per domain, no orphan catalog maps |

## Universal contract

```text
Capability (UnitKind | DroneRole | MissileClass | Deployable | BuildingPlant)
  → Cargo lines (tag, class, buffer/reorder, TOE qty)
  → Plant outputs (catalog_id + chain role OR Manufacturing blueprint)
  → Haul bus (LogisticsOrderSim for freight; ledger only for proven deployables)
  → Consumer (Front stock | Formation.supply | DronePackage | MissilePackage | SiteStaging | VehicleFleet)
  → Closed loop (delivery credits consumer; attrition republishes demand)
```

### Hard rules

| Rule | Meaning |
|:---|:---|
| No orphan tags | Every `CargoKind` string is listed in [`cargo_kinds_v1.ron`](../../assets/configs/economy/cargo_kinds_v1.ron) |
| No orphan plants | Producible tags have catalog buildings (or `scenario_only`) — never map ids without JSON |
| No demand-only military | Publish without refill is incomplete |
| Buildings | Construction funnel → activate → flow and/or MFG; picker after catalog+chain green |
| Buildings-as-products (P7) | Catalog + chain only; never `ManufacturingNode` without `catalog_id` (activation already enforces) |
| Presentation | Never invents production (air D8 read-only) |

### Haul-bus rule

| Bus | Use for |
|:---|:---|
| **LogisticsOrderSim** (preferred) | Continuous civ materials, UAS discretes, missiles, front sustainment |
| **InTransitLedger → SiteStagingStock** | Deployable `*_unit` only until merged (documented debt) |

Power/`Electricity` is **`FacilityInputs`**, not a cargo loop.

## Data authorities

| Asset | Owner |
|:---|:---|
| [`cargo_kinds_v1.ron`](../../assets/configs/economy/cargo_kinds_v1.ron) | Cargo tag registry |
| [`capability_toe_v1.ron`](../../assets/configs/combat/capability_toe_v1.ron) | TOE lines per capability |
| [`industrial_supply_chains.json`](../../assets/configs/industrial_supply_chains.json) | Multi-step plant chains |
| `assets/configs/buildings/*.json` | Catalog plants |
| `ManufacturingBlueprintRegistry` | Discrete MFG recipes |

## Domain board

| Domain | Status | Notes |
|:---|:---|:---|
| Concrete / aluminum continuous flow | **SHIPPED** | ResourceFlow + order sim |
| Power / grid | **SHIPPED** | FacilityInputs; not cargo |
| Front ammo/Fuel/rations/engineering | **SHIPPED** | Aggregate UnitKind draw |
| Trucks (class-masked fleet) | **SHIPPED (P4)** | fleet_growth + registry class + TROOP recipe |
| Deployables MFG→staging | **SHIPPED** (dual-bus debt) | Ledger path |
| Petroleum well→refinery | **SHIPPED (P1)** | `petroleum` chain + linker order |
| Aluminum E2E | **NOTE** | Catalog+edges shipped; deepen seed when needed (Portland remains primary proof) |
| MFG → order-sim plant stock | **SHIPPED (P2)** | Activate buffers + `bridge_mfg_output_stock` + MFG plant sites |
| UAS plants + DronePackage refill | **SHIPPED (P3)** | Catalog JSON + aligned `drone_airframe` + refill system; industrial menu |
| Logistics formations (haul amplify) | **SHIPPED** | [`plan_haul_amplify_v1.md`](plan_haul_amplify_v1.md) — Formation mul on order-sim only |
| Personnel restock | **SHIPPED (P5)** | toe_personnel deficit → PublishDemand + credits → resolve strength |
| Missiles / air magazines | **SHIPPED (P6)** | MissilePackage + AirOpZone weather sorties; tags + demand/refill; MFG blueprint ready |
| Missile catalog buildings | **SHIPPED** | `missile_*.json` + `missile_assembly` chain → picker (MISSILE-CATALOG-1) |
| Nuclear / geopolymer depth | **DEFER** | Unless blocking |

## Later / residual

### Logistics formations (haul amplify) — SHIPPED (HAUL-AMPLIFY-1)
Formation kind mul scales effective trip capacity inside `LogisticsOrderSim` only — see [`plan_haul_amplify_v1.md`](plan_haul_amplify_v1.md).

### Missile plants — SHIPPED (MISSILE-CATALOG-1)
`missile_seeker_plant` / `missile_warhead_plant` / `missile_body_plant` + `missile_assembly` chain; discrete MFG blueprints; industrial picker lists them. Haul: MFG buffers → order-sim plant bridge → front → `MissilePackage` refill (P6).

### Deployable dual-bus (Order 7 note)
Deployables MFG→staging remain on the **ledger** path (SHIPPED with dual-bus debt). Narrowing: do not invent a second order-sim fleet for deployables; keep ledger staging until a single-bus migration is chartered. Missile plants are **not** on that bus — they use the order-sim plant stock bridge like UAS.

## Phase exit predicates

| Phase | Exit |
|:---|:---|
| **P0** | Charter + RON registries + loader/tests green |
| **P1** | `petroleum` in chains JSON + `chain_catalog_order`; oil→fuel edge can link |
| **P2** | Activate inserts `ManufacturingOutputBuffers` when blueprint has tag; bridge credits order-sim plant stock |
| **P3** | UAS catalog JSON + chain; demand tags aligned; front stock → `DronePackage` credit; industrial menu lists UAS |
| **P4** | Fleet recipes use registry Steel/Machinery; class via `CargoKindRegistry`; TROOP recipe; formations DEFER documented |
| **P5** | Four sustainment tags unchanged; `personnel` PublishDemand from toe_personnel deficit; resolve applies credits |
| **P6** | `missile_*` tags live; MissilePackage + demand/refill; AirOpZone sorties weather-gated consume; toe missiles usable |
| **P7** | Charter note: buildings-as-products = catalog+chain; no ManufacturingNode without catalog_id |

## Rejected

- Parallel DroneSim / missile entity micro
- AI final art for plants
- Replacing ResourceFlow with MFG for concrete/aluminum
- Picker entries for map-only catalog ids
