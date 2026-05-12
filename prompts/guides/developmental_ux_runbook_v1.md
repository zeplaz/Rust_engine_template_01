# Developmental UX runbook v1

> **PURPOSE:** The engine is **systems-heavy, layered, emergent, operational, pressure-driven**. Player-facing UX must teach **causality**, not only expose controls. Menus are insufficient; this runbook defines **interpreted operational meaning** across layers.

**Version:** v1.0.0  
**Boundary:** Player-facing teaching UI → **Bevy UI** (`ui_boundary_guide_v1.md`). Designer / debugging depth → **egui** where noted. Do not mix stacks in one plugin.

**Canonical principles:** `base_ui_direction_principls.md` (command table, map-primary).

---

## Core teaching chain

Teach in order:

1. **Cause** — what state / input produced this?
2. **System interaction** — which sim systems are coupled?
3. **Strategic consequences** — what changes in the world if I commit?

Never stop at “invalid” or a raw scalar.

---

## UX layers (L0–L7)

| Layer | Goal | Primary surface | Phase |
|-------|------|-----------------|--------|
| **L0** Immediate context | Mode, selection, affordances, keys — **no window** | Bevy ops + slim context strip | **UX-1** |
| **L1** Failure explanation | Every failure answers **why** (structured diagnostics) | Same strip + future detail tray | **UX-1** |
| **L2** Dependencies | Causal chains (power → pump → water) | Hover / inspector graph | **UX-2** |
| **L3** Strategic consequences | Pre-commit **projected** effects | Ghost / preview copy | **UX-3** |
| **L4** Emergent patterns | Readable faction / doctrine abstractions | Tooltips, codex-light | **UX-4** |
| **L5** Simulation storytelling | Semantic event feed, not `-5 wood` | Narrative observation bus | **UX-4** |
| **L6** Mission authoring | Pressures, biases, constraints — not hardcoded scripts | Authoring tooling | **UX-5** |
| **L7** AI explainability | “Why attack?” contributors for dev/mod/balance | Debug / egui inspector | **UX-6** |

---

## Delivery phases

### UX-1 — Context + feedback (now)

- Always-visible **developmental context strip** under ops row (L0).
- **Validation diagnostics** from placement / actions (L1) — `ValidationDiagnostic` + `ValidationSeverity`.
- Keybinding hints from `InputBindings`; **operational language** from `operational_feedback_language_v1.md`.
- **Files:** `src/gui/hud/*.rs` (see codebase).

### UX-2 — Dependency + causality

- ECS: `DependencyLink` (or graph resource), upstream tracing systems.
- UI: “CAUSE CHAIN:” formatted tree (Bevy or egui for dense graphs).

### UX-3 — Strategic analysis

- Pre-commit **projected** deltas (throughput, risk, ecology) from field/graph snapshots.
- Overlay **district summaries** (supply fragility, flood exposure, etc.).

### UX-4 — Simulation storyteller

- `NarrativeObservation` bus: category, severity, **generated_text** (template + ECS facts).
- Event feed UI; faction / doctrine tooltips (L4).

### UX-5 — Mission authoring

- See `mission_authoring_framework_v1.md`.
- Mission pressure knobs wired to sim, not quest scripts.

### UX-6 — AI explainability

- See `simulation_explainability_runbook_v1.md`.
- Contributor breakdown for major AI decisions (dev + optional “trust” mode).

---

## Design rule (non-negotiable)

**Do not** surface raw tuners to players by default (`utility_weight=0.3244`).

**Do** surface **interpreted operational meaning** (“prioritizing territorial consolidation”).

---

## Related documents

- `operational_feedback_language_v1.md` — tone, BAD/GOOD examples, token glossary.
- `mission_authoring_framework_v1.md` — L6 pressures & authoring table.
- `simulation_explainability_runbook_v1.md` — L7 patterns, modder hooks.
- `ui_boundary_guide_v1.md` — Bevy vs egui split.

---

## Test rounds

Mirror other runbooks: **implement → full `cargo test` → add tests** for new copy contracts and diagnostic mapping tables where deterministic.
