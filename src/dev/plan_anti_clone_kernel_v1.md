# PLAN-ANTI-CLONE-KERNEL-v1 — no parallel protocol farms

| Field | Value |
|:---|:---|
| **Id** | **PLAN-ANTI-CLONE-KERNEL-v1** |
| **Status** | **SIGNED** |
| **Date** | 2026-11-15 |
| **Parents** | [`plan_combat_sustainment_v1.md`](plan_combat_sustainment_v1.md) · cleanup-intelligence scan 2026-11-15 |

## Rule (standing)

1. **Protocol ≠ product.** N systems with the same control flow and only type/tag/sink swapped = **CLONE_FARM** → one kernel + typed adapters. Do **not** mash domains into a god-system.
2. **One pool per bus per frame.** Shared stock scans run once; sinks allocate from that pool.
3. **A shared bus is not a license to clone.** New consumers add a sink adapter or book section — not a sibling `publish_*` / `refill_*` file.
4. **False-positive bar.** Different metering, writer resource, or layer (sim vs view vs UI) → KEEP_DISTINCT; document in the owning plan.
5. **Gate.** Before shipping a 2nd parallel system in a domain, cite this rule or open a CS-/ACK- slice.

## Known farms

| Id | Status | Owner |
|:---|:---|:---|
| **CF-1** combat drone/missile/personnel publish+refill | **CLOSED** — CS-1…5 (shared pool + one publisher) | `plan_combat_sustainment_v1.md` |
| **CF-2** settlement dual one-shot enqueue | mild → TWN-AI | `plan_town_growth_v1.md` |

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| 1.0.0 | 2026-11-15 | SIGNED — standing rule + CF table. |
