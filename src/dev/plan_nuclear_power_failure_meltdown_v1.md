# PLAN-NUCLEAR-POWER-FAILURE-001 — offsite power · SCRAM · meltdown `v1`

```text
⟦SYMLANG⟧⟐v1  ◈PLAN
⟨ID⟩ PLAN-NUCLEAR-POWER-FAILURE-001
Date: 2026-06-18
Status: **SIGNED** (@planner)
Owner: @designer (profiles + UX) · @coder (sim phases)
Charter: $ref:src/dev/design_nuclear_containment_failure_v1.md
Parent: $ref:src/dev/plan_power_grid_construction_ux_v1.md · power parity matrix §5
```

**Headline:** Nuclear generators that **lose offsite power** must follow a **type-specific failure ladder** — SCRAM, diesel-backed cooling, decay heat, and (for PWR-class plants) **meltdown** if cooling is not restored. Other plant types trip safely without core melt.

---

## 0. Why this plan exists

- `ContainmentBuilding` + `nuclear_containment_placeholder` exist but are **empty**
- `pwr_4loop_1100mw_v1` has aux loads but **no offsite-power dependency**
- Grid **islanding** (electrical) is the primary gameplay trigger for **LOOP**
- Designer questions ([`rulebook_backlog_designer_brief_v1.md`](../../docs/archive/2026-06-prompts-guides/runbooks/guides/rulebook_backlog_designer_brief_v1.md)) ask for decay heat / SCRAM behavior — **unanswered in sim**

---

## 1. Phased delivery

| Phase | Scope | Ship gate |
|:---|:---|:---|
| **P1 — LOOP & SCRAM** | Offsite power loss → auto SCRAM · diesel start · restore = stable | No meltdown yet |
| **P2 — Decay heat crisis** | Diesel depletion · core heat rise · critical alerts | Meltdown threshold |
| **P3 — Meltdown & breach** | Core damage · containment breach · fallout hook (WSS) | Operator sign-off |
| **P4 — Type expansion** | SMR passive profile · BWR variant | Per-profile JSON |

**Do not ship P3** without P1 grid-link + plant card UX green.

---

## 2. Track A — Data & profiles (@designer → JSON)

| ID | Deliverable |
|:---|:---|
| **DES-NUCLEAR-FAILURE-PROFILE-001** | `nuclear_failure_profile` schema on `PlantDefinition` |
| **DES-NUCLEAR-FAILURE-PROFILE-002** | PWR 4-loop profile values (time constants, MW aux) |
| **DES-NUCLEAR-FAILURE-PROFILE-003** | Coal/gas **negative** profile (meltdown disabled) |

**Authority:** `assets/config/power/plant_definitions.json` — extend schema, don't hardcode in Rust.

---

## 3. Track B — Simulation (@coder)

| ID | Deliverable | Phase |
|:---|:---|:---|
| **COD-NUCLEAR-GRID-LINK-001** | Plant offsite feed from `UtilityGraph` / grid membership | P1 |
| **COD-NUCLEAR-LOOP-SCRAM-001** | LOOP detect → SCRAM state + message | P1 |
| **COD-NUCLEAR-DIESEL-001** | Diesel backup resource + fuel countdown | P1 |
| **COD-NUCLEAR-COOLING-001** | Decay heat → `ThermalComponent` | P2 |
| **COD-NUCLEAR-MELTDOWN-001** | Replace `nuclear_containment_placeholder` | P2–P3 |
| **COD-NUCLEAR-EVENTS-001** | `NuclearScramEvent`, `MeltdownEvent` messages | P1+ |

**Capability rule:** all logic on `With<ContainmentBuilding>` + profile JSON — per [`failure_modes.rs`](../../src/entities/production/power/failure_modes.rs).

---

## 4. Track C — UX (@designer → @coder)

| ID | Deliverable |
|:---|:---|
| **DES-NUCLEAR-UX-ALERTS-001** | Tiered alert copy |
| **DES-NUCLEAR-PLANT-CARD-001** | Offsite / diesel / core heat panel |
| **DES-NUCLEAR-MAP-READ-001** | Reactor icon states |
| **DES-ART-VFX-NUCLEAR-001** | SCRAM / meltdown VFX charter — [`plan_power_grid_art_assets_v1.md`](plan_power_grid_art_assets_v1.md) Lane E |
| **DES-ART-NUCLEAR-PLANT-001** | PWR containment massing — Lane A2 |
| **COD-NUCLEAR-UX-WIRE-001** | Ops strip + focus card |

---

## 5. Dependencies

| Program | Link |
|:---|:---|
| [`plan_power_grid_construction_ux_v1.md`](plan_power_grid_construction_ux_v1.md) | Grid islanding triggers LOOP |
| [`plan_industrial_facility_grammar_suite_v1.md`](plan_industrial_facility_grammar_suite_v1.md) | Substation/transformer as targets |
| [`plan_power_grid_art_assets_v1.md`](plan_power_grid_art_assets_v1.md) | Modules, VFX, HUD, nuclear visual states |
| [`power_damage_ui_persistence_v1.md`](../docs/reference/designer_questions/production_economy/power_damage_ui_persistence_v1.md) | Repair / damage model |
| WSS radiation | Containment breach → fallout (P3) |

---

## 6. Priority order

```text
P1  DES-NUCLEAR-FAILURE-PROFILE-001 + DES-NUCLEAR-UX-ALERTS-001
P1  COD-NUCLEAR-GRID-LINK-001 + LOOP-SCRAM + DIESEL
P2  COD-NUCLEAR-COOLING-001 + plant card wire
P3  COD-NUCLEAR-MELTDOWN-001 + map read + fallout hook
```

---

## 7. Success metrics

| Metric | Target |
|:---|:---|
| PWR loses offsite → SCRAM | **100%** auto when `scram_on_loop` |
| Coal loses offsite → meltdown | **0%** |
| Player sees diesel countdown | before heat rise |
| Restore power averts melt | when within window |
| Terminology | grid islanding ≠ nuclear island (glossary) |

---

## Changelog

| Ver | Date | Note |
|:---|:---|:---|
| v1.0.0 | 2026-06-18 | Initial — LOOP/SCRAM/meltdown phased plan |

```text
⟦/PLAN-NUCLEAR-POWER-FAILURE-001⟧  ΔWF→ P1 LOOP/SCRAM before meltdown
```
