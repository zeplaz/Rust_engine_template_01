# Global iso readability rules `v1`

| Field | Value |
|:---|:---|
| **ID** | **DES-STYLE-ISO-READ-001** |
| **Program** | PLAN-DESIGNER-WORK-202606-001 · Track C1 |
| **Date** | 2026-06-20 |
| **Owner** | `@designer` |
| **Verdict** | **PASS** |

```text
DES-STYLE-ISO-READ-001 Q✓
Silhouette · roof legibility @64px · fire read @ operational zoom
```

---

## 1. Zoom bands

| Band | px/tile | Read priority |
|:---|:---:|:---|
| L0 strategic | ≤8 | color mass only |
| L1 operational | 9–16 | roof + door |
| L2 tactical | 17–32 | bay rhythm |
| L3 inspect | 33+ | material hint |

---

## 2. Silhouette rules

| Rule | Spec |
|:---|:---|
| Roof dominant | ≥40% of tile silhouette |
| Wall secondary | vertical breaks every 2u |
| Prop max | 15% of tile — no noise fields |
| Fire | orange rim + dark core — not full fill |

---

## 3. Acceptance still @ 64px

Operator captures at 1280×720 · operational zoom · 3 building types + 1 burn tile.

---

## Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer` | **PASS** | 2026-06-20 |
