# DESIGN-NUCLEAR-CONTAINMENT-FAILURE-001 — loss of power · SCRAM · meltdown `v1`

| Field | Value |
|:---|:---|
| **Program** | **PLAN-NUCLEAR-POWER-FAILURE-001** |
| **ID** | **DESIGN-NUCLEAR-CONTAINMENT-FAILURE-001** |
| **Date** | 2026-06-18 |
| **Owner** | `@designer` (concepts + UX) · `@planner` (phasing) · `@coder` (sim) |
| **Verdict** | **PASS (charter)** |
| **Code hooks** | `ContainmentBuilding` · `nuclear_containment_placeholder` · `ThermalComponent` |
| **Data** | `assets/config/power/plant_definitions.json` — `pwr_4loop_1100mw_v1` |
| **Parity matrix** | [`power_legacy_functional_parity_v1.md`](../../docs/archive/2026-06-prompts-guides/matrix/matrix/production/power_legacy_functional_parity_v1.md) §5 Nuclear column |
| **Grid coupling** | [`plan_power_grid_construction_ux_v1.md`](plan_power_grid_construction_ux_v1.md) — **grid islanding** = loss of offsite power |

**Headline:** Nuclear plants are **not** “just another generator.” If they **lose offsite power** and **backup cooling** fails, decay heat can escalate through **SCRAM → core heat → meltdown → containment breach**. Coal/gas lose grid and trip; **PWR-class nuclear** can fail catastrophically. Plan must be **type-specific**, **data-driven**, and **readable on map + alerts**.

**North star:** Player sees **why** a plant SCRAMmed, **how long** diesels buy, and **what happens** if cooling is not restored — before meltdown, not after opaque collapse.

**Rejected:** one meltdown timer for all plant types · meltdown without prior SCRAM/cooling failure · conflating **grid islanding** with **nuclear containment island** (see §0.1).

---

## 0. Terminology (lock)

| Term | Meaning | Not |
|:---|:---|:---|
| **Grid islanding** | Electrical subgraph disconnected from grid — no offsite feed | Terrain `island_mode` |
| **Offsite power** | Station service from external grid (MV feeds, aux transformers) | On-site turbine generator export |
| **Station black** | Plant internal buses lost — pumps/comms dead | Same as SCRAM |
| **SCRAM** | Control rods in — chain reaction stopped | Shutdown complete (decay heat remains) |
| **Decay heat** | Residual core heat after SCRAM — requires cooling **days** | Instant cool-down |
| **LOOP** | Loss Of Offsite Power | Random brownout |
| **Meltdown** | Core damage from inadequate cooling post-SCRAM | Reactor still critical |
| **Containment breach** | Pressure/vessel failure — radiological release | Meltdown synonym (breach is **later** phase) |
| **Nuclear island** (engineering) | Reactor + containment building physical boundary | Electrical grid island |

---

## 1. What exists today (honest)

| Piece | State |
|:---|:---|
| `PowerPlantType::Nuclear` + `pwr_4loop_1100mw_v1` | JSON on disk — aux loads, SCRAM-ish shutdown states |
| `ContainmentBuilding` marker | Attached to nuclear defs |
| `nuclear_containment_placeholder` | **Empty** system in `failure_modes.rs` |
| `ThermalComponent` | On activated plants — temperature fields unused for meltdown |
| Grid overload / island UX | Partial — overload toast; island **planned** |
| Fallout / radiation sim | Mentioned in WSS plan — **not** wired to nuclear failure |

**Gap:** No **offsite-power dependency**, **diesel backup**, **SCRAM trigger**, or **meltdown state machine**.

---

## 2. Plant-type failure taxonomy

Not every generator fails the same when power is lost.

| Class | Examples | Lose offsite power | Meltdown path? |
|:---|:---|:---|:---:|
| **A — Nuclear containment** | PWR 4-loop (`pwr_4loop_1100mw_v1`) | SCRAM + diesels required · cooling clock | **Yes** |
| **B — Nuclear (future SMR)** | Small modular | Shorter decay curve · passive cooling option | **Reduced** / passive bypass |
| **C — Steam thermal** | Coal, gas, biomass | Boiler trip · aux trip · no core melt | No (fire/explosion separate) |
| **D — Hydro / renewable** | Hydro, wind, solar | Inverter/gate trip · safe stop | No |
| **E — Black-start capable** | Some gas peakers, hydro | Can restart island | No |

**Rule:** meltdown logic queries **`ContainmentBuilding`** + **`PlantDefinition.nuclear_failure_profile`** — never bare `PowerPlantType::Nuclear` match in scattered code.

---

## 3. Nuclear failure state machine (PWR baseline)

```text
Operational
    ↓ (grid island / line cut / transformer loss)
OffsitePowerLost (LOOP)
    ↓ (auto within N seconds)
Scrammed + DieselsStarting
    ↓
├─ Offsite restored OR diesels stable → ScrammedCoolingStable (decay heat managed)
├─ Diesels fail / fuel out → ScrammedCoolingDegraded → CoreHeatRising
│       ↓ (timer + cooling margin)
│   CoreDamage → Meltdown → ContainmentBreach (optional release phase)
└─ Instant attack on containment → may skip to breach (military)
```

### 3.1 Triggers (design)

| Trigger | Auto SCRAM? | Notes |
|:---|:---:|:---|
| **Grid island** — plant loses offsite feed | Yes | Primary gameplay path |
| **Transformer destroyed** upstream | Yes | Ties to power targeting |
| **Overload cascade** | Maybe | Designer: overload alone ≠ SCRAM unless aux trip |
| **Direct military hit** on reactor | Yes + damage skip | Separate damage track |
| **Operator manual SCRAM** | Yes | Future control panel |

### 3.2 Time constants (data-driven per plant def)

| Phase | PWR 4-loop (draft) | Player read |
|:---|:---|:---|
| SCRAM delay after LOOP | 10–30 s | “Auto SCRAM — offsite power lost” |
| Diesel start | 30–60 s | “Emergency diesels starting” |
| Diesel sustain window | 72 h (game-compressed) | Fuel bar on plant card |
| Core heat rise if cooling lost | 30–120 min sim | Rising gauge + amber alert |
| Meltdown | after heat threshold | Critical alert + map icon |
| Breach | after meltdown + pressure | Fallout hook |

**Compress time for gameplay** — but **order** must stay physically plausible (SCRAM before melt).

### 3.3 Aux power accounting

From existing JSON: `aux_load_fraction` per status — extend with:

```json
"nuclear_failure_profile": {
  "requires_offsite_power_when": ["Standby", "Operational", "ScrammedCooling"],
  "offsite_power_mw": 12.0,
  "diesel_backup_mw": 10.0,
  "diesel_fuel_hours": 72.0,
  "passive_cooling": false,
  "scram_on_loop": true,
  "meltdown_enabled": true
}
```

**Coal plant:** omit block or `meltdown_enabled: false`.

---

## 4. Coupling to power grid program

| Grid event | Nuclear consequence |
|:---|:---|
| **Line cut** islanding plant | LOOP if no alternate feed |
| **Transformer KO** | LOOP for downstream plant bus |
| **Restore line** before diesel depletion | Cooling stable — crisis averted toast |
| **Enemy cuts redundant paths** | Strategic KO — player must keep **two feeds** or diesels |

**Design:** nuclear sites should **encourage redundant MV feeds** — teaches grid building from [`plan_power_grid_construction_ux_v1.md`](plan_power_grid_construction_ux_v1.md).

---

## 5. Player UX (design)

### 5.1 Alerts (tiered)

| Tier | Example |
|:---|:---|
| Info | `Nuclear · offsite power lost — SCRAM initiated` |
| Warn | `Nuclear · diesels running · 48 h fuel remaining` |
| Critical | `Nuclear · core cooling degraded · restore power` |
| Emergency | `Nuclear · meltdown in progress · evacuate district` |
| Catastrophic | `Containment breach · radiation release` |

Use ops strip **PWR** zone + plant focus card — not log-only.

### 5.2 Plant focus card (minimum)

| Field | Source |
|:---|:---|
| Status | Operational / SCRAM / Meltdown |
| Offsite feed | Connected / islanded |
| Diesels | Off / Starting / Running / Failed |
| Core heat | 0–100% gauge |
| Containment pressure | 0–100% (P1) |
| Time to next phase | “~18 min to core damage” |

### 5.3 Map read

| State | Map |
|:---|:---|
| SCRAM | Reactor icon amber |
| Cooling degraded | Pulsing amber ring |
| Meltdown | Red core icon + smoke VFX hook |
| Breach | Radiation overlay zone (WSS future) |

---

## 6. Designer / coder split

| ID | Agent | Deliverable |
|:---|:---|:---|
| **DES-NUCLEAR-FAILURE-PROFILE-001** | @designer | `design_nuclear_failure_profiles_v1.md` — A/B plant classes + JSON schema extension |
| **DES-NUCLEAR-UX-ALERTS-001** | @designer | Alert copy + plant card wire |
| **DES-NUCLEAR-MAP-READ-001** | @designer | Icon states + district evac read |
| **PLAN-NUCLEAR-SIM-PHASES-001** | @planner | Phase 1 LOOP/SCRAM → Phase 2 meltdown → Phase 3 breach/fallout |
| **COD-NUCLEAR-LOOP-SCRAM-001** | @coder | Offsite power check + auto SCRAM |
| **COD-NUCLEAR-COOLING-001** | @coder | Diesel + decay heat + `ThermalComponent` drive |
| **COD-NUCLEAR-MELTDOWN-001** | @coder | State machine + events |
| **COD-NUCLEAR-GRID-LINK-001** | @coder | Hook grid islanding → LOOP |

**Phase 1 only:** LOOP → SCRAM → diesel window → cool if power restored. **No meltdown** until Phase 2 signed.

---

## 7. Acceptance tests

| # | Test |
|:---:|:---|
| N1 | Cut last offsite line → SCRAM within spec delay |
| N2 | Restore MV feed during diesel window → stable SCRAM, no melt |
| N3 | Diesel depletion → core heat rises — alert fires |
| N4 | Coal plant grid loss → trip, **no** meltdown path |
| N5 | Player sees offsite/diesel/core heat on plant card |
| N6 | Meltdown only on defs with `meltdown_enabled: true` |

---

## Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer` | **PASS (charter)** | 2026-06-18 |

```text
DESIGN-NUCLEAR-CONTAINMENT-FAILURE-001 → Phase 1 COD-NUCLEAR-LOOP-SCRAM-001
```
