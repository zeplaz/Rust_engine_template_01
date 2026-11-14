# Ballistics / blast contract (Ranking C3)

| Field | Value |
|:---|:---|
| **Status** | **CONTRACT** — implement behind this; no greenfield physics world |
| **Parent** | [`plan_slice_c_sensory_v1.md`](plan_slice_c_sensory_v1.md) |
| **Cite** | gea_v2_e4 `physics_integration` p423 · `advanced_physics` p440 |

## Rules

1. **Sim owns trajectories and damage.** Presentation (FX, audio) reads snaps / dumps.
2. Prefer **deterministic closed-form / stepped integrate** (same spirit as `ember_spot_ignition` / ember physics) over middleware for missiles and blast radii.
3. **Destruction / env damage** goes through existing construction / ecology funnels — no overlay HP writers.
4. **Collision** = footprint / radius vs graphs and fields first; charter a middleware sole writer only if product requires continuous contact.
5. Emit audio via `AudioOneShotQueue::push_combat` / `push_warning` — never `ResMut` sim from audio.

## Extension points (code)

| Existing | Use for |
|:---|:---|
| `systems/fire/ember_spot_ignition.rs` ballistic loft | Missile / shell loft patterns |
| `MissionBook` / missile sortie dump | Launch events → audio + near-band ASM `Launch` |
| `construction/blast.rs` | **SHIPPED scaffold** — `BlastIntentQueue` → crater + demolish |
| `SimAttentionFrame` | Full-tier theaters get high-rate ballistic steps |

**Flow:** `step_ballistic_flights` (impact) → enqueue `BlastIntent` → `execute_blast_intents_system` (sole site mutator).

## Rejected

- Rapier/Avian for every truck
- Per-airframe ECS rigid bodies as default AI atoms
- UI or audio writing blast damage
