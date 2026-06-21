# Utility yard research `v2`

| Field | Value |
|:---|:---|
| **ID** | **DES-INDUSTRIAL-RESEARCH-003** |
| **Program** | PLAN-INDUSTRIAL-FACILITY-GRAMMAR-001 · Track E1-A |
| **Date** | 2026-06-20 |
| **Owner** | `@designer` |
| **Extends** | [`design_industrial_process_research_v1.md`](design_industrial_process_research_v1.md) §3 |
| **Verdict** | **PASS** |

```text
DES-INDUSTRIAL-RESEARCH-003 Q✓
utility_role yards — not supply chains
```

---

## Yard patterns

| Role | primary | utility | service | rail |
|:---|:---:|:---:|:---:|:---:|
| substation | transformer pads | **≥60%** | control shack | optional |
| transformer | pad + bus | **≥50%** | — | — |
| coal plant | turbine hall | coal + cooling | water intake | **required** |

**Rule:** `utility_role` buildings never appear in chain browser step list.

---

## Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer` | **PASS** | 2026-06-20 |
