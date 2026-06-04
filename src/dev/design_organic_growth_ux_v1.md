# DESIGN-ORGANIC-GROWTH-UX-001 — District organic growth UX `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **DESIGN-ORGANIC-GROWTH-UX-001** |
| **Parent** | [`construction_procedural_buildings_plan_v1.md`](construction_procedural_buildings_plan_v1.md) |
| **Coder exec** | [`plan_organic_growth_exec_001_v1.md`](plan_organic_growth_exec_001_v1.md) |
| **Version** | `1.1.0` |
| **Date** | 2026-06-02 |
| **Owner** | `@designer` |
| **Verdict** | **PASS** |
| **Unblocks** | **PROC-OG-3-001** |
| **No Rust** | UX / overlay / copy contract |

---

## Hard rule

**Zone paint never instant-builds.** Growth shows **proposals** (dashed ghosts) → player **Approve** / **Reject** / **Auto policy** → same `ConstructionPlanQueue` → staged pipeline → **Operational**.

**Forbidden UI copy:** “Zone built”, “Construction complete” on zone paint alone.

---

## Player mental model

```text
I zone land → I build roads/power/water
District pressure rises → proposals appear (dashed)
I approve (or auto policy approves) → normal construction stages run
```

---

## 1 — District map overlay

| Layer | Toggle key | Visual | Default |
|:---|:---|:---|:---:|
| Zoning | `Z` | existing zone paint | on |
| **Residential pressure** | `Pr` | warm gradient 0–100% | off |
| **Commercial pressure** | `Pc` | amber gradient | off |
| **Industrial pressure** | `Pi` | cool grey gradient | off |
| **Desirability** | `D` | green → red | off |
| **Transport access** | `T` | graph reach highlight (teal edge glow) | off |
| **Growth proposals** | `G` | dashed footprint pins | on when pending |

**Proposal ghost:** 2px **dashed** outline, fill α **15%**, hue = district type (res `#c8a070`, com `#e8b040`, ind `#8090a8`). **Never** solid like player tool ghost.

---

## 2 — District inspector (wireframe)

```text
┌─ District: North Industrial ──────────────┐
│ Population 1.2k · Jobs 840 · Housing 62%   │
│ Desirability 0.71 · Transport 0.85        │
│ Pollution 0.22 · Crime 0.08               │
├───────────────────────────────────────────┤
│ Auto-build:  ( ) Off  (•) Approve each    │
│              ( ) Auto residential         │
│              ( ) Auto all commercial      │
├───────────────────────────────────────────┤
│ Pending (3)                               │
│  · Warehouse 4×2  Block 7    [Approve][×] │
│  · Workshop 2×1   Block 7    [Approve][×] │
│  · Corner shop 2×1 Block 12  [Approve][×] │
├───────────────────────────────────────────┤
│ [Approve all] [Reject all] [Pause 30d]    │
└───────────────────────────────────────────┘
```

| Field | Source |
|:---|:---|
| Metrics | `DistrictMetrics` |
| Policy | district book `auto_build_policy` |
| List | `GrowthProposal` queue |

---

## 3 — Growth proposal card

```text
[Commercial] Corner shop · Block 12
Reason: high transport + employment
Footprint: 2×1 · Style: Victorian
[Approve] [Reject] [Inspect location]
```

| Action | Effect |
|:---|:---|
| **Approve** | Enqueue construction plan (Planned phase — not Operational) |
| **Reject** | Remove proposal; cooldown tile optional |
| **Inspect** | Pan camera; flash dashed ghost 2s |

---

## 4 — Auto-build policy states

| Policy | Player label | Behavior |
|:---|:---|:---|
| `Off` | **Manual only** | Proposals accumulate; no auto enqueue |
| `ApproveEach` | **Ask before each build** | Toast + inspector queue |
| `AutoResidential` | **Auto: housing** | Auto-approve `usage=Residential` under cap |
| `AutoCommercial` | **Auto: commercial** | Same for commercial |
| `AutoIndustrial` | **Auto: industry** | Same for industrial |
| `AutoAll` | **Auto: all types** | Cap: **3 proposals / sim day** per district |

**Pause growth 30d:** sets `growth_frozen_until_tick` — proposals visible but not approvable.

---

## 5 — Ghost language (legend required)

| Ghost type | Outline | Fill | Meaning |
|:---|:---|:---|:---|
| Player build ghost | **solid** 2px | tool color 25% | user placement |
| Parametric staged ghost | solid + checklist icon | staging UX v2 | queue preview |
| **Growth proposal** | **dashed** 2px | district hue 15% | sim suggestion — **not built** |

Tray help line: `Dashed outline = city proposal (not your build). Approve in District panel.`

---

## 6 — District identity (zone → usage weights)

| Zone paint | Default proposal mix |
|:---|:---|
| Residential low | detached, duplex, corner shop |
| Residential med | apartments, row, small retail |
| Mixed use | retail ground + residential upper |
| Commercial | shop, office low-rise |
| Industrial | warehouse, workshop, depot |
| Civic | school, office, government |

Weights are **probabilities**, not spawn lists — still one proposal at a time through queue.

---

## 7 — Notification toast (sim)

`City planning: 3 new proposals in North Industrial` — click opens inspector.

**Not:** `Buildings placed` on zone paint.

---

## Acceptance

| # | Criterion | Status |
|:---:|:---|:---:|
| G1 | Overlay layers + proposal dashed style | ☑ |
| G2 | Inspector wireframe + policy table | ☑ |
| G3 | Ghost distinction vs player/parametric | ☑ |
| G4 | Zone → usage weights | ☑ |
| G5 | Approve / Reject / Auto / Pause copy | ☑ |
| G6 | No instant zone→built implication | ☑ |

---

## Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer` | **PASS** | 2026-06-02 |
