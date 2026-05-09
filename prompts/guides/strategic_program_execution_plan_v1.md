# Strategic program execution plan `v1`

> **STATUS:** Draft **v1** — **recommended start order** across orchestrators and new runbooks.

Version: `v1.0.0`  
Parent index: [`simulation_expansion_orchestrator_v1.md`](simulation_expansion_orchestrator_v1.md)

**Orchestrators:**

- [`strategic_fields_and_ai_orchestrator_v1.md`](strategic_fields_and_ai_orchestrator_v1.md)
- [`infrastructure_and_research_orchestrator_v1.md`](infrastructure_and_research_orchestrator_v1.md)
- [`experience_layer_orchestrator_v1.md`](experience_layer_orchestrator_v1.md)

---

## 1. Principles

1. **Experience and boundaries early** — cheap to align; prevents rework when overlays land.
2. **Doctrine as ongoing gate** — not a late-only doc; check anti-patterns whenever adding war/network features.
3. **Construction before heavy AI** — AI *consumes* graphs and fields; stub AI on empty graphs wastes time.
4. **Overlay contract before city/war AI** — city and operational AI assume readable congestion, risk, and recon fields.

---

## 2. Phased order (what to start first)

| Phase | Focus | Runbooks / docs | Rationale |
|:---:|:---|:---|:---|
| **P0** | Shell + input + doctrine checklist | [`experience_layer_orchestrator_v1.md`](experience_layer_orchestrator_v1.md), [`doctrine_simulation_alignment_runbook_v1.md`](doctrine_simulation_alignment_runbook_v1.md) §2–3 | Stable UX + catch design drift early |
| **P1** | Strategic overlay MVP | [`strategic_overlay_runbook_v1.md`](strategic_overlay_runbook_v1.md) | Fields other domains will read |
| **P2** | Infrastructure construction core | [`infrastructure_construction_runbook_v1.md`](infrastructure_construction_runbook_v1.md), [`infrastructure_corridor_runbook_v1.md`](infrastructure_corridor_runbook_v1.md) | Corridors need nodes/edges from construction |
| **P3** | Resilience + reroute | [`infrastructure_resilience_and_failure_runbook_v1.md`](infrastructure_resilience_and_failure_runbook_v1.md) | Makes logistics and AI meaningful |
| **P4** | Logistics AI | [`logistics_ai_runbook_v1.md`](logistics_ai_runbook_v1.md) | Uses graphs + stress + failures |
| **P5** | Settlement growth | [`settlement_growth_runbook_v1.md`](settlement_growth_runbook_v1.md) | Demographics before full city planner |
| **P6** | City planning AI | [`ai_city_planning_runbook_v1.md`](ai_city_planning_runbook_v1.md) | Heavy overlay + congestion dependence |
| **P7** | Operational warfare AI | [`ai_operational_warfare_runbook_v1.md`](ai_operational_warfare_runbook_v1.md) | Needs logistics + threat/recon fields |
| **P8** | Research ecosystem | [`research_capability_ecosystem_runbook_v1.md`](research_capability_ecosystem_runbook_v1.md) | Parallelizable from **P2** onward as **design/data**; tighten loops after industry anchors exist |

---

## 3. Parallel tracks

- **Track A (sim):** P1 → P2 → P3 → P4 following [`strategic_fields_and_ai_orchestrator_v1.md`](strategic_fields_and_ai_orchestrator_v1.md).
- **Track B (infra/research):** P2 ↔ P8 design in lockstep; implementation weight shifts to P8 after [`concrete_industry_sim_runbook_v1.md`](concrete_industry_sim_runbook_v1.md) / [`petroleum_industry_simulation_runbook_v1.md`](petroleum_industry_simulation_runbook_v1.md) mature enough to feed **institutions** and **production maturity**.
- **Track C (UX):** P0 continuous; revisit when each overlay milestone ships (inspectors, toggles, visual language).

---

## 4. “Start next” one-liner

If nothing else is blocked: **finish P0, then P1 (`strategic_overlay`), then P2 (`infrastructure_construction` + corridor)** — that trio unblocks almost all downstream AI and UI visualization.
