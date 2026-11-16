# PLAN-MOVE10-WILDLIFE-v1 — minimal sim-affecting wildlife flock

| Field | Value |
|:---|:---|
| **Id** | **PLAN-MOVE10-WILDLIFE-v1** |
| **Status** | **REOPENED 2026-11-16** — herd never seeded in play; soft costs unread. **MOVE-10 → UNWIRED** · cleanup **CQ-51/CQ-52** |
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

## Review 2026-11-16 — reopened

> Operator code review of the Cursor wave ([`plan_cleanup_cursor_wave_v1.md`](plan_cleanup_cursor_wave_v1.md)). The product stance above stands: wildlife must affect sim (soft path cost), not decoration.

| Finding | Evidence | Fix |
|:---|:---|:---|
| Book always empty in play | `WildlifeHerdBook` is `Default`; `from_seed` called only in tests (`wildlife_flock.rs:273,309`), so `tick_wildlife_herd_system` returns early every frame | Seed herds from the world seed in the ecology spawn (CQ-51) |
| Syncs never run | `sync_nav_wildlife_soft`, `sync_agent_frame_from_wildlife` have no production caller | Schedule both (CQ-51) |
| Soft cost unread | `NavFieldFrame.wildlife_soft_samples` has no reader | Input to the single route cost function (CQ-52 → CQ-53) |
| Not visible | no render path | If drawn: instanced sprites on the existing sprite path |

**Close again only when:** herds exist in a normal seeded run, and a route measurably avoids a herd in a test that uses the production route cost. Every ADOPTED claim cites a production caller `file:line` (a test alone does not count).

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
| **Status** | **REOPENED 2026-11-16** — see Review |

## Rejects

- Wiring wildlife into combat formations
- Unseeded randomness
- Decorative-only agents with zero sim effect
