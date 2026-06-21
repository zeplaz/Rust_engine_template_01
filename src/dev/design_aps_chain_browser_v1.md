# APS chain browser `v1` — read-only step picker

| Field | Value |
|:---|:---|
| **ID** | **DES-APS-CHAIN-BROWSER-001** |
| **Program** | PLAN-INDUSTRIAL-FACILITY-GRAMMAR-001 · Track E3-A |
| **Date** | 2026-06-20 |
| **Owner** | `@designer` |
| **Depends** | [`design_aps_facility_needs_v1.md`](design_aps_facility_needs_v1.md) · [`design_industrial_process_research_v1.md`](design_industrial_process_research_v1.md) |
| **Authority** | `assets/configs/industrial_supply_chains.json` |
| **Verdict** | **PASS** |

```text
DES-APS-CHAIN-BROWSER-001 Q✓
Read-only chain diagram · pick step → pre-fill archetype + district
```

---

## 1. Placement

**Tab:** Assembly · **Advanced** collapsible (G2+ grammar tier).

```text
┌─ Chain browser ─────────────────────────────────────────────┐
│ Concrete (Portland)  ▼                                       │
│ [mine]──►[kiln]──►[mixer]   ← selected step highlighted      │
│ ⚡ medium · catalog: concrete_cement_kiln                     │
│ [Use step in Assembly]                                       │
└──────────────────────────────────────────────────────────────┘
```

---

## 2. Interaction

| Action | Result |
|:---|:---|
| Select chain | Load steps from JSON — no invented steps |
| Click step node | Highlight + Facility Needs strip refresh |
| **Use step** | Pre-fill archetype row + district from catalog binding |
| Edit fields | **Forbidden** — read-only; edit in grammar RON |

---

## 3. Empty / error

| Case | Copy |
|:---|:---|
| No binding | `○ Step has no catalog binding — pick another` |
| Chain missing | `○ Chain not in registry` |

---

## Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer` | **PASS** | 2026-06-20 |
