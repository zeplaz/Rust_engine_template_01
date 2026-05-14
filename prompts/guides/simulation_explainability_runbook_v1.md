# Simulation explainability runbook v1

> **PURPOSE:** **L7** — make AI / pressure-driven behavior **legible** for balancing, missions, mods, and player trust. Raw ECS dumps are for developers only.

**Parent:** [`experience_layer_orchestrator_v1.md`](experience_layer_orchestrator_v1.md) · [`strategic_fields_and_ai_orchestrator_v1.md`](strategic_fields_and_ai_orchestrator_v1.md)

**Related:** [`developmental_ux_runbook_v1.md`](developmental_ux_runbook_v1.md), [`operational_feedback_language_v1.md`](operational_feedback_language_v1.md), [`legacy_cpp_repos_agent_communication_maps_v1.md`](legacy_cpp_repos_agent_communication_maps_v1.md) §8.5 (belief gap contributors)

**Version:** v1.0.0  
**Boundary:** Deep breakdowns → **egui** diagnostics / “explain” panel; **interpreted** one-liners may appear in Bevy HUD when a `trust_explain` flag exists.

---

## Problem statement

Without explainability:

- AI feels random.
- Mission tuning is blind.
- Modders cannot reason about doctrine + pressure.
- Balancing regresses silently.

---

## “Why did X happen?” pattern

Example: **Why did this faction attack?**

Show **contributors** as signed, **labeled** factors — not weights:

- + expansionist doctrine
- + rail advantage vs neighbor
- + mission pressure (offensive nudge)
- − fuel shortage (dampener)
- − internal unrest (dampener)

Implementation sketch (future):

- `DecisionScoreComponents` or parallel **explainability snapshot** resource written when a decision fires.
- Optional `ExplainabilityPlugin` (dev / advanced options).

---

## Rules

1. **Never** show raw internal weights to default players.
2. **Always** map to **operational_language** (`operational_feedback_language_v1.md`).
3. **Stable contributor ids** for telemetry and mod patches (include **belief error**, **dispatch delay**, **stale intel** when Stage-7 comms land).
4. **Deterministic in replay** when seeded — explainability record should replay with sim.

---

## Phasing

- **UX-6 (dev first):** egui panel “last decision breakdown” + faction inspector.
- **Optional player:** “Briefing” difficulty setting unlocks one-line strategic hints (still interpreted).

---

## Related

- [`developmental_ux_runbook_v1.md`](developmental_ux_runbook_v1.md) — layer L7.
- [`operational_feedback_language_v1.md`](operational_feedback_language_v1.md) — contributor phrasing.
- [`strategic_fields_and_ai_orchestrator_v1.md`](strategic_fields_and_ai_orchestrator_v1.md) — AI consumers.
- [`base_behav_a.md`](base_behav_a.md) — behavior pipeline composition boundaries.
- Hybrid / behavior pipeline modules — hook explainability at composition boundaries.
