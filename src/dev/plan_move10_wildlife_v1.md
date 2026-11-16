# PLAN-MOVE10-WILDLIFE-v1 — minimal sim-affecting wildlife flock

| Field | Value |
|:---|:---|
| **Id** | **PLAN-MOVE10-WILDLIFE-v1** |
| **Status** | **PROGRAM CLOSE** — **MOVE-10 ADOPTED** |
| **Date** | 2026-11-16 |
| **Parents** | [`plan_ref_books_defer_unlock_v1.md`](plan_ref_books_defer_unlock_v1.md) **MOVE-10-FLOCK** |
| **Unlocks** | **MOVE-10** |
| **Dual-cite** | aig3 15.2 / 15.2.2 / 15.2.3 flocking · steering stability |

## Product stance (operator 2026-11-16)

| Decision | Choice |
|:---|:---|
| Agents | **Yes** — minimal wildlife/crowd exist (ecology domain) |
| Scope | Seed-driven · few agents · simple flock (sep + cohesion + align) |
| Sim effect | Soft path cost near herd (nav RO) — not decoration-only |
| Combat | ⛔ Never write `Formation.tile` / posture / combat members |

## Authority (frozen)

```text
⊚WildlifeHerdBook (ecology) ═▶ ⊨ wildlife positions / velocities / soft costs
⊚NavFieldFrame.wildlife_soft_samples ◂⊳[RO] WildlifeHerdBook
⊚AgentFrame.agent_count ◂⊳[RO] wildlife count (ecology sync)
⛔ Formation.tile / combat FormationMember
⛔ Full crowd AI / decorative-only particles
```

## Slice M10-01 — flock + soft cost

| Field | Value |
|:---|:---|
| **Files** | `systems/ecology/wildlife_flock.rs` · `frame_snapshots` wildlife samples |
| **Do** | Seed spawn · stable flock step · soft cost samples · Accept `--lib move10` |
| **Status** | **DONE** 2026-11-16 |

## Rejects

- Wiring wildlife into combat formations
- Unseeded randomness
- Decorative-only agents with zero sim effect
