# S7B intel minimap legend `v1` — threat + logistics pins

| Field | Value |
|:---|:---|
| **ID** | **DES-S7B-INTEL-MINIMAP-001** |
| **Program** | PLAN-DESIGNER-WORK-202606-001 · Track D |
| **Date** | 2026-06-20 |
| **Owner** | `@designer` |
| **Depends** | [`minimap_ux_v1.md`](minimap_ux_v1.md) · stage7 behavioral lane |
| **Verdict** | **PASS** |

```text
DES-S7B-INTEL-MINIMAP-001 Q✓
Intel pin taxonomy · legend strip · toggle defaults
```

---

## 1. Pin taxonomy

| Pin | Shape | Label |
|:---|:---:|:---|
| Threat | `▲` red | `Threat` |
| Supply | `■` blue | `Supply` |
| Objective | `◇` gold | `Objective` |
| Friendly | `●` green | `Unit` |

---

## 2. Legend strip (minimap corner)

```text
▲ Threat   ■ Supply   ◇ Obj
```

| Rule | Spec |
|:---|:---|
| Max 4 rows | collapse overflow to `+N` |
| Toggle | per-category on/off in minimap menu |
| Default sim | Threat + Objective on |

---

## Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer` | **PASS** | 2026-06-20 |
