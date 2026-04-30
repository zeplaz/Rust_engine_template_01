# Diplomacy / bargaining reference outline `v1`

**Status:** **Non-authoritative.** Brainstorm / ingested external prompt fragment about a Spaniel-style game-theory / bounded-rationality agent (Bayesian beliefs, signaling, equilibrium reasoning, deception/trust, cooperation/conflict). The **binding** specs do not exist yet; this outline is a **pre-authoring scratchpad** for whoever later writes the `prompts/matrix/factions/` matrices and the `factions/diplomacy/` designer specs.

Use this file to spawn **new implementation questions** and **matrix rows**, not as runtime truth. Pair with [`../terrain_world/llm_world_evolution_reference_outline_v1.md`](../terrain_world/llm_world_evolution_reference_outline_v1.md) (sister outline for the **worldgen** side of agent reasoning).

---

## Context (why this is a separate outline)

- **Where it plugs in:** factions / NPCs / scenario AI. Existing anchors: [`faction_editor/01_data_model.md`](faction_editor/01_data_model.md) (DTOs), [`implementation_questions_v1.md`](implementation_questions_v1.md), repo-wide bargaining mentions in [`terrain_world/spec/03_political_territory.md`](../terrain_world/spec/03_political_territory.md).
- **Why non-authoritative:** there is no diplomacy matrix yet, no Rust types, no save format. Per the meta-runbook ([`../../guides/system_runbook_authoring_meta_v1.md`](../../guides/system_runbook_authoring_meta_v1.md) §10) "empty-system policy" applies until the matrix lands.
- **What this file is for:** answer "where would this idea live in our repo if it became real?" without committing to any of it.

---

## Causes of bargaining failure (lifted as design pressure)

The outline opens with seven causes of war / bargaining failure (cognitive factors, cost-benefit calculations, domestic politics, constructivism, multi-player bargaining, divergent interpretations of identical information, usefulness in individual cases). Treat these as **design pressures** the future diplomacy system must accommodate, not features:

| Cause | Implication for any future diplomacy design |
|:---|:---|
| **Cognitive factors** | Belief updates must be **bounded** — not full Bayes. Allow stickiness (`belief_inertia` parameter, **`ASK:`**). |
| **Cost-benefit calculation differences** | Every agent has its own utility weights; no shared utility function. |
| **Domestic politics** | Faction leaders may act against the faction's "objective" interest — model leader entity distinct from faction. |
| **Constructivism (identity through conflict)** | Reputation / identity drift over time, not fixed at scenario start. |
| **Multi-player bargaining** | Equilibrium computation must support `n > 2`. Any 2-player-only API is a dead end. |
| **Divergent interpretation** | Same observation produces different belief deltas per agent. No global "truth feed." |
| **Single-case unpredictability** | Outcomes are **distributional, not deterministic in gameplay** — but the **simulator** itself remains deterministic for the same seed + scenario (per repo invariant). |

> **Hard constraint inherited from the repo:** the **simulator** is deterministic for `seed + scenario + config`. The **stories** that emerge can be unpredictable; the **code path** producing them must reproduce.

---

## Naming alignment (outline phrase ↔ canonical repo, where it would map)

| Outline phrase | Likely canonical (proposed) | Notes |
|:---|:---|:---|
| Agent / faction | `FactionId` (existing in faction_editor docs) + new `AgentId` for sub-agents (`ASK:`) | |
| Belief state | New `BeliefSet` component (`ASK:` representation) | |
| Type / type space | Discrete enum per scenario (`ASK:` whether scenario-defined or registry-defined) | |
| Action space | `ActionId` registry (mirrors `MaterialRegistry` pattern) | |
| Signal | `SignalKind { CheapTalk, Costly, Index }` enum | |
| Game / interaction | `InteractionTemplate` asset (`ASK:` JSON or RON) | |
| Payoff matrix | `PayoffMatrix2D` (and `PayoffTensorN` for `n > 2`) | |
| Reputation | `ReputationLedger` component on faction | |
| Equilibrium mode | `EquilibriumKind { Nash, BayesianNash, PerfectBayesian, Sequential }` | |

> All of these are **proposed**, none authoritative. They become real only when a `prompts/matrix/factions/diplomacy_*` matrix is authored and approved.

---

## Core data structures (proposed shapes; not implementations)

### Agent (extends faction DTO)

```jsonc
{
  "id": "faction_42",
  "type": "faction | npc | system",
  "state": { "resources": 120, "position": [x, y], "capabilities": ["move", "harvest", "attack"] },
  "preferences": { "utility_weights": { "survival": 1.0, "expansion": 0.7, "conflict": -0.3 } }
}
```

> Constraint: **`utility_weights`** must live in scenario / faction config (designer-edited), not Rust constants. Hot-reload via `Assets<T>` if added.

### BeliefSet (about other agents)

```jsonc
{
  "about_agent_17": {
    "type_distribution": { "aggressive": 0.6, "defensive": 0.4 },
    "resource_estimate": { "mean": 80, "variance": 20 },
    "belief_inertia": 0.3
  }
}
```

> Constraint: **probability distributions stay distributions**. No collapsing to MAP estimate inside the engine.

### InteractionTemplate

```jsonc
{
  "players": ["A", "B"],
  "type_space": ["aggressive", "passive"],
  "actions": ["cooperate", "defect"],
  "payoffs": { "A": [[3,0],[5,1]], "B": [[3,5],[0,1]] }
}
```

### ReputationLedger

```jsonc
{
  "agent_17": { "trustworthiness": 0.7, "aggression": 0.6, "consistency": 0.8 }
}
```

---

## Decision pipeline (proposed phases — would map to a future runbook)

```
Observe ECS / world tags  →  Update beliefs (bounded Bayesian)
                                  ↓
                 Infer opponent strategy (per equilibrium mode)
                                  ↓
                 Evaluate payoff matrix under current beliefs
                                  ↓
                 Compute best response (or sample mixed strategy)
                                  ↓
                 Select action / signal
                                  ↓
                 Log decision (mandatory debug emission)
```

Each phase is a **future runbook step**. The meta-runbook (§7) prescribes the schema. Until the matrix exists, each step is `Pending — ASK:`.

---

## Signaling system (proposed taxonomy)

| Signal kind | Cost | Credibility seed |
|:---|:---|:---|
| `CheapTalk` | 0 | low — discount on receive |
| `Costly` | resource debit | proportional to cost |
| `Index` | inherent (state-tied) | high — hard to fake |

**Deception model:** misreport allowed; cost is reduced future credibility tracked in `ReputationLedger.consistency`. Requires a per-interaction `ObservedSignal` log so cross-checks can detect inconsistency over time.

---

## Equilibrium reasoning modes

The outline asks for Nash / Bayesian Nash / Perfect Bayesian / Sequential. **Repo position:** treat each as a **separate solver** behind a feature flag:

| Mode | Default available? | Feature flag |
|:---|:---:|:---|
| Nash | Yes (simplest) | always on |
| Bayesian Nash | optional | `diplomacy_bayes` |
| Perfect Bayesian | optional | `diplomacy_pbe` |
| Sequential | optional | `diplomacy_seq` |

> Solvers must be **deterministic** for fixed inputs. Any randomization (mixed strategies) seeded from the scenario seed.

---

## Cooperation / conflict strategies

| Strategy | Note |
|:---|:---|
| `tit_for_tat` | baseline cooperative |
| `grim_trigger` | one-shot defection ⇒ permanent defection |
| `forgiving_tit_for_tat` | probabilistic forgiveness, seed-deterministic |
| `randomized_retaliation` | mixed strategy with seeded RNG |

These live in scenario / faction config, **not** code constants.

---

## Integration with terrain / worldgen (where this outline meets the world)

| World signal | Diplomacy effect |
|:---|:---|
| `MaterialDef.properties.traversable` | Movement cost → payoff modifier in spatial games |
| `TagSet` containing resource tags | Resource estimate prior in `BeliefSet` |
| `MaterialFamily` (`TerrainClass`) | Capability constraints (e.g. amphibious vehicle in Wetland) |
| `ChunkCellMatrix` boundaries | Multi-player bargaining over shared chunk borders |

> These integration points are **read-only** for diplomacy: world influences strategy, strategy never edits world via this path. World edits flow through the worldgen side ([`../terrain_world/llm_world_evolution_reference_outline_v1.md`](../terrain_world/llm_world_evolution_reference_outline_v1.md)) only.

---

## Constraints summary (must / must-not)

**MUST**

- maintain probabilistic beliefs (no premature collapse)
- log reasoning steps (mandatory debug emission per decision)
- avoid perfect-information assumptions
- support `n > 2` players from day one (don't bake 2-player APIs)
- be deterministic given seed + observations
- stay modular (pluggable into ECS + worldgen outputs)

**MUST NOT**

- assume opponent honesty
- collapse belief distributions to point estimates inside the engine
- ignore signaling cost structures
- mutate world state directly (world edits flow only through worldgen)
- hardcode utility weights, thresholds, or strategies (designer-config only)

---

## Mandatory debug emission (proposed shape)

```jsonc
{
  "decision": "cooperate",
  "beliefs": { "opponent_aggressive": 0.4 },
  "expected_payoff": 3.2,
  "reasoning": "cooperation yields higher expected value under current belief"
}
```

Behind the same `dev_tools` feature gate already used for `RuleTrace` (matrix §17 / impl Q §75 in terrain). One log path, two producers.

---

## What to feed back into authoritative docs (later, when human is ready)

These become real **only after a diplomacy matrix is authored**. Until then, leave as `📎 ASK:`:

- New designer doc: `prompts/designer_questions/factions/diplomacy_v1.md` — narrative + DTOs.
- New matrix: `prompts/matrix/factions/diplomacy_migration_matrix_v1.md` — phase plan, type↔file map.
- New implementation questions in [`implementation_questions_v1.md`](implementation_questions_v1.md) covering belief-set representation, equilibrium-solver determinism, multi-player payoff tensor storage, signaling cost ledger, reputation persistence in saves.
- New per-system runbook authored via [`../../guides/system_runbook_authoring_meta_v1.md`](../../guides/system_runbook_authoring_meta_v1.md) using **phase letter `D`**.

---

## Cross-links

- Sister outline (worldgen LLM evolution): [`../terrain_world/llm_world_evolution_reference_outline_v1.md`](../terrain_world/llm_world_evolution_reference_outline_v1.md)
- Faction data model: [`faction_editor/01_data_model.md`](faction_editor/01_data_model.md)
- Faction implementation checklist: [`implementation_questions_v1.md`](implementation_questions_v1.md)
- Repo-wide bargaining mentions: [`../terrain_world/spec/03_political_territory.md`](../terrain_world/spec/03_political_territory.md)
- Meta-runbook (how to author when matrix exists): [`../../guides/system_runbook_authoring_meta_v1.md`](../../guides/system_runbook_authoring_meta_v1.md) (especially §§10–12)
