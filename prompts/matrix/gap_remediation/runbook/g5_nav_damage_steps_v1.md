# G5 — Navigation / damage / manufacturing remediation `v1`

> **STATUS:** **Placeholder pack** — there is **no single “shared sim control” owner** for all findings below. **Navigation, damage, and production/manufacturing** can expose **different** debug/player UI, **different** tables (damage, loss, throughput), and **different** world-space overlays (pathfinding, flow debug for developers). **Data sources** are **assets**, **simulation logic**, and the **ECS** components/resources each feature already uses. **Matrices:** add or extend rows when a domain needs a traceability anchor; otherwise drive **from code** and backfill docs when stable.

**Pair:** orchestrator [`../../../guides/gap_remediation_runbook_v1.md`](../../../guides/gap_remediation_runbook_v1.md) · hunt guide [`../../../guides/implementation_gap_hunt_runbook_v1.md`](../../../guides/implementation_gap_hunt_runbook_v1.md) · navigation questions [`../../../designer_questions/navigation/implementation_questions_v1.md`](../../../designer_questions/navigation/implementation_questions_v1.md) · production specs [`../../../designer_questions/production_economy/spec/README.md`](../../../designer_questions/production_economy/spec/README.md).

**Promotion / owner queue:** [`../../../guides/rulebook_backlog_designer_brief_v1.md`](../../../guides/rulebook_backlog_designer_brief_v1.md) §4 **BQ-111** — per subsystem before `G5-SNN` atomic steps.

---

## Scope

G5 owns **subsystem-shaped** gameplay placeholders surfaced by gap hunt, for example:

- [`../../../../src/systems/navigation/potental_feild_nav.rs`](../../../../src/systems/navigation/potental_feild_nav.rs) — owner defaults until `AgentOwnable` is on vehicle entities.
- [`../../../../src/systems/damage/damage_system.rs`](../../../../src/systems/damage/damage_system.rs) — road damage accumulation and related tables.
- [`../../../../src/entities/production/core/manufacturing_plugin.rs`](../../../../src/entities/production/core/manufacturing_plugin.rs) — throughput / decay / alert placeholders.

**Other gaps** in these areas land here when gap-hunt routes them — each may need its **own** mini-spec or matrix row.

---

## Recorded answers (replaces single-owner blocker table)

| Topic | Decision |
|:---|:---|
| **Owner** | **Per finding / per subsystem** — navigation debug (e.g. **path display in world** for developers), damage (**damage / loss tables**, road accumulation), manufacturing (**flow, throughput, alerts**) are **not** one bucket. Split atomic steps by **file + behavior**. |
| **Player-visible vs dev-only** | **Navigation:** path/flow debug in-world for tests. **Damage:** tables and state that affect or explain loss. **Production:** alerts and throughput views tied to ECS **assets + logic**. Document per `G5-SNN` when promoted. |
| **Data source** | **Assets** and **ECS** (components/resources) for that subsystem — same evolution rule as G3: extend as sim features land. |
| **Matrix row** | **If** a navigation/damage/production matrix exists for the change, **flip that row**. **If none exists,** author the **implementation** first or add a **minimal matrix row** when the behavior stabilizes — **no blocking** on a pre-existing row name. |

---

## Living work queue (add rows from gap-hunt)

| Finding | Subsystem | ECS / assets | UI / debug surface (if any) |
|:---|:---|:---|:---|
| Potential-field owner | Navigation | *vehicles / AgentOwnable* | optional nav overlay |
| Road damage | Damage | *damage inputs* | damage/loss inspection |
| Manufacturing throughput | Production | *blueprints / ticks* | alerts / diagnostics hooks |

---

## Placeholder sequencing

When promoted, **split by subsystem** (not “one G5 stream”):

1. Potential-field owner derivation.
2. Road damage accumulation (+ tables as designed).
3. Manufacturing throughput / efficiency / alerts.
4. In-world debug overlays where developers need **flow** visibility.
5. Matrix/spec backfill for rows touched.

No `G5-SNN` atomic steps exist yet by design.
