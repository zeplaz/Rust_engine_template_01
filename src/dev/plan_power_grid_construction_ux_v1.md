# PLAN-POWER-GRID-CONSTRUCTION-UX-001 — draw · read · attack · repair `v1`

```text
⟦SYMLANG⟧⟐v1  ◈PLAN
⟨ID⟩ PLAN-POWER-GRID-CONSTRUCTION-UX-001
Date: 2026-06-18
Status: **SIGNED** (@planner)
Owner: @designer (UX) · @coder (construction + graph) · @coder B (overlay/damage read)
Parent: $ref:src/dev/plan_infrastructure_world_layers_exec_001_v1.md INFRA-E4
Charter: $ref:src/dev/design_power_line_construction_ux_v1.md
```

**Headline:** Power lines are **strategy** — building them should feel as good as roads, with **curved vs 90°** routing, **clear voltage types**, and **map-readable** damage, islanding, transformer knockouts, and repair.

---

## 0. Gap

| Have | Need |
|:---|:---|
| `UtilityGraph`, `PowerLine`, `VoltageClass` | Player **draw** tool |
| Road/rail path UX | Power **parallel** tool |
| Grid overload toast | **Island** + **cut** + **repair** read |
| Gold overlay stroke | **State** variants (preview, damage, dead) |
| `UtilityAuthoringTool` stub | Full authoring loop |

---

## 1. Tracks

```text
Track A — Line construction UX     draw modes · tool sheet · snap · commit  ★ P0
Track B — Map read & overlay       strokes · nodes · hover · load/island    P1
Track C — Combat & repair UX       cut · transformer KO · repair queue     P2
Track D — Sim integration          graph → activation → overload/island    P1–P2
```

---

## 2. Track A — Construction (P0)

| ID | Agent | Deliverable |
|:---|:---|:---|
| **DES-POWER-LINE-TOOL-SHEET-001** | @designer | Tool sheet wire + copy (extends road popup pattern) |
| **DES-POWER-ROUTING-MODE-001** | @designer | Curved vs 90° rules + keybind |
| **DES-POWER-VOLTAGE-PICKER-001** | @designer | Low / MV / HV picker + mismatch copy |
| **DES-POWER-SNAP-RULES-001** | @designer | Transformer, junction, grid snap |
| **COD-POWER-LINE-DRAW-001** | @coder | `BuildTool::PowerLine`, placement resource, input system |
| **COD-POWER-LINE-COMMIT-001** | @coder | Commit → `UtilityNetworkSnapshot` / graph edge |
| **COD-POWER-ORTHOGONAL-ROUTER-001** | @coder | 90° Manhattan path generator |
| **COD-POWER-SPLINE-ROUTER-001** | @coder | Curved — reuse infra spline |

**Pattern authority:** `src/construction/roads/` (popup, input, ghost, commit)

---

## 3. Track B — Map read (P1)

| ID | Agent | Deliverable |
|:---|:---|:---|
| **DES-POWER-MAP-OVERLAY-002** | @designer | Line state visuals (live, preview, damage, island) |
| **DES-POWER-NODE-HOVER-001** | @designer | Transformer/substation hover cards |
| **COD-POWER-OVERLAY-RENDER-001** | @coder B | Compositor strokes by `VoltageClass` + state |
| **COD-POWER-ISLAND-HIGHLIGHT-001** | @coder | Island boundary + dim unpowered |

**Color authority:** [`design_infra_network_overlay_v1.md`](design_infra_network_overlay_v1.md) gold family

---

## 4. Track C — Combat & repair (P2)

| ID | Agent | Deliverable |
|:---|:---|:---|
| **DES-POWER-TARGETING-001** | @designer | Cut line / KO transformer preview copy |
| **DES-POWER-REPAIR-PANEL-001** | @designer | Repair queue UX (parts, priority) |
| **COD-POWER-DAMAGE-SEGMENT-001** | @coder | Segment HP + cut → graph split |
| **COD-POWER-REPAIR-QUEUE-001** | @coder | Repair jobs UI + sim hook |

**Authority:** [`power_damage_ui_persistence_v1.md`](../docs/reference/designer_questions/production_economy/power_damage_ui_persistence_v1.md)

---

## 5. Track D — Integration

| ID | Agent | Deliverable |
|:---|:---|:---|
| **COD-UTILITY-ACTIVATION-LINK-001** | @coder | Activation reads `UtilityConnection` not radius hack |
| **COD-POWER-ISLAND-TOAST-001** | @coder | Island toast + ops strip (extend IND-E03 pattern) |
| **COD-POWER-TOOL-RAIL-001** | @coder | Utilities rail → Lines entry |

**Epic milestone (INFRA-E4):** Place line → graph → factory powers on.

**Nuclear coupling:** **Grid islanding** (electrical subgraph cut) can cause **loss of offsite power (LOOP)** at nuclear plants → [`plan_nuclear_power_failure_meltdown_v1.md`](plan_nuclear_power_failure_meltdown_v1.md). Islanding alone is not meltdown — SCRAM + diesel window first.

---

## 6. Priority order

```text
P0  DES-POWER-LINE-TOOL-SHEET-001 + ROUTING-MODE + VOLTAGE-PICKER
P0  COD-POWER-LINE-DRAW-001 + SPLINE + ORTHOGONAL routers
P1  COD-POWER-LINE-COMMIT-001 + OVERLAY-RENDER + ISLAND-HIGHLIGHT
P1  COD-UTILITY-ACTIVATION-LINK-001
P2  DES-POWER-TARGETING-001 + REPAIR-PANEL → coder damage/repair
```

---

## 7. Success metrics

| Metric | Target |
|:---|:---|
| Operator “fun to wire” score | **8/10+** |
| Draw + commit without docs | **yes** (T1–T3) |
| Curved ↔ 90° switch | **≤1 click** |
| Island/cut readable on map | **yes** (T4–T5) |
| Voltage mismatch blocked | **100%** with reason |

---

## 8. Links

| Program | Link |
|:---|:---|
| Industrial facility grammar | Transformers/substations as nodes — [`plan_industrial_facility_grammar_suite_v1.md`](plan_industrial_facility_grammar_suite_v1.md) |
| Sim HUD Phase 2 | Tool sheet chrome — [`plan_sim_hud_professional_polish_v1.md`](plan_sim_hud_professional_polish_v1.md) |
| INFRA-E4 | [`plan_infrastructure_world_layers_exec_001_v1.md`](plan_infrastructure_world_layers_exec_001_v1.md) §8 |

**Prompt:** [`designer_power_grid_prompt_v1.md`](designer_power_grid_prompt_v1.md)

**Art & assets:** [`plan_power_grid_art_assets_v1.md`](plan_power_grid_art_assets_v1.md) · [`designer_mcp_power_grid_art_prompt_v1.md`](designer_mcp_power_grid_art_prompt_v1.md)

---

## Changelog

| Ver | Date | Note |
|:---|:---|:---|
| v1.0.0 | 2026-06-18 | Initial — power line construction + strategic read |

```text
⟦/PLAN-POWER-GRID-CONSTRUCTION-UX-001⟧  ΔWF→@designer TOOL-SHEET · @coder DRAW
```
