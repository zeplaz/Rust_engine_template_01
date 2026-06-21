# Aluminum chain site templates `v2`

| Field | Value |
|:---|:---|
| **ID** | **DES-INDUSTRIAL-RESEARCH-002** |
| **Program** | PLAN-INDUSTRIAL-FACILITY-GRAMMAR-001 · Track E1-A |
| **Date** | 2026-06-20 |
| **Owner** | `@designer` |
| **Extends** | [`design_industrial_process_research_v1.md`](design_industrial_process_research_v1.md) §2 |
| **Verdict** | **PASS** |

```text
DES-INDUSTRIAL-RESEARCH-002 Q✓
Aluminum site JSON templates · 22→85→200→48 power ladder
```

---

## Site grid templates (handoff DMCP-PILOT-ALUMINUM-SITE-001)

| Step | W×D | primary% | utility% | loading% | buffer% |
|:---|:---:|:---:|:---:|:---:|:---:|
| bauxite_mine | 8×7 | 20% | 10% | 0% | **35%** |
| alumina_refinery | 10×8 | 18% | **25%** | 8% | 20% |
| aluminum_smelter | 12×10 | 15% | **30%** | 5% | 25% |
| aluminum_fabrication | 8×6 | 14% | 12% | **15%** | 18% |

**Required adjacency:** smelter ↔ substation ≤4 tiles · refinery → smelter ≤2 tiles feed.

---

## Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer` | **PASS** | 2026-06-20 |
