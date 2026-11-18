# DISASTER-PROXY-v1 — hydro/fire → integrity via construction funnel

| Field | Value |
|:---|:---|
| **Id** | **DISASTER-PROXY-v1** |
| **Program** | [`plan_destruction_presentation_v1.md`](plan_destruction_presentation_v1.md) |
| **Status** | **ADOPTED** — flood foundation proxy + fire path tripwire |
| **Date** | 2026-11-17 |
| **Owner** | `@coder` |

## Proxies

| Force | Path |
|:---|:---|
| Fire | Existing [`FireCollapseQueue`](../systems/fire/entity_exposure.rs) → `execute_demolish_at_tile` |
| Flood | [`disaster_proxy.rs`](../construction/disaster_proxy.rs) — foundation depth ≥ gate → escalate + `StructuralCollapseQueue` → demolish |

No parallel despawn. Full \(½ρv²\) force field remains DEFER.
