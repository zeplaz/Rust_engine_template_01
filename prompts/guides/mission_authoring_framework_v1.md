# Mission authoring framework v1

> **PURPOSE:** **L6** — campaigns manipulate **pressures, biases, constraints, relationships** — not hardcoded cinematic scripts. Sim produces consequences; missions **inject** operational conditions.

**Version:** v1.0.0  
**See also:** `developmental_ux_runbook_v1.md` (UX-5), `strategic_program_execution_plan_v1.md`, behavior / pressure docs.

---

## Design intent

- **MissionPressure**-style knobs (unrest, paranoia, logistics stress, ideology, weather bias, doctrine nudges) feed the **same** systems players experience.
- **Authoring UX** (future): visual operational tooling — tables, curves, region pickers — still behind egui / editor boundary per `ui_boundary_guide_v1.md`.

---

## Category table (authoring surface)

| Category | Example knob | Sim consumer (directional) |
|----------|--------------|----------------------------|
| Pressure injection | unrest +0.3 | `PressureField`, missions, hybrid brain |
| Faction mood | paranoia | Fracture / behavior pipelines |
| Logistics disruption | fuel shortage | Transport / overlay fields |
| Ideology pressure | nationalism | Decision composition |
| Weather modifiers | drought | Chunk weather / ecology |
| AI doctrine bias | defensive | Behavior model weights |
| Infrastructure targets | rail vulnerability | Graph / corridor hooks |
| Narrative hints | evacuation pressure | Mission readouts, optional observations |

---

## Authoring rules

1. **Declare pressures explicitly** — schema-version mission RON/JSON; no hidden globals.
2. **Prefer ranges and curves** over one-shot spikes unless the scenario demands it.
3. **Validate against sim** — mission load should fail soft with **diagnostics** (L1) for designers.
4. **No bypass** — authored effects must route through ECS / scheduled systems (same rule as AI construction).

---

## UX-5 deliverables (staged)

1. Schema: `MissionPressure` + binding to existing `PressureField` / mission components.
2. Editor panel: inject, preview, diff against baseline world snapshot.
3. Player: optional **briefing strip** summarizing active mission pressures (interpreted, not raw).

---

## Related

- `simulation_explainability_runbook_v1.md` — trace pressure → decision for L7.
- Scenario scripting (Wave 1) — orchestration **steps** vs **pressure injection**; keep boundaries clear in code comments.
