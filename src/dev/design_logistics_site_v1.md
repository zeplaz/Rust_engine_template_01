# Rail logistics site read `v1`

| Field | Value |
|:---|:---|
| **ID** | **DES-LOGISTICS-SITE-001** |
| **Program** | PLAN-INDUSTRIAL-FACILITY-GRAMMAR-001 · Track E1-C |
| **Date** | 2026-06-20 |
| **Owner** | `@designer` |
| **Pilot** | `logistics_rail_warehouse_pilot_v1.json` |
| **Verdict** | **PASS** |

```text
DES-LOGISTICS-SITE-001 Q✓
Rail-edge warehouse — spur + loading wing + buffer
```

---

## Site zones

| Zone | % typical | Read |
|:---|:---:|:---|
| primary | 25% | warehouse mass |
| loading | 20% | dock doors on rail edge |
| rail | 15% | spur + platform |
| buffer | 25% | truck queue |
| parking | 10% | service |

**Grammar:** `RailEdge` archetype · district `industrial_west` default.

---

## Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer` | **PASS** | 2026-06-20 |
