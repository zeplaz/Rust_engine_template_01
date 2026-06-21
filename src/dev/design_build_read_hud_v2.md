# Build readability HUD `v2` — ghost + commit state

| Field | Value |
|:---|:---|
| **ID** | **DES-BUILD-READ-HUD-002** |
| **Program** | PLAN-DESIGNER-WORK-202606-001 · Track D |
| **Date** | 2026-06-20 |
| **Owner** | `@designer` |
| **Depends** | [`construction_parametric_staging_ux_v2.md`](construction_parametric_staging_ux_v2.md) |
| **Verdict** | **PASS** |

```text
DES-BUILD-READ-HUD-002 Q✓
Ghost validity · scale · corridor phase — one strip
```

---

## 1. Strip (bottom-left build mode)

```text
BUILD  ·  Valid ✓  ·  2×1  ·  Corridor · phase 2
```

| State | Copy |
|:---|:---|
| Valid ghost | `Valid ✓` |
| Invalid | `Blocked ✗ · {reason}` |
| Scale | `{w}×{h}` or parametric length |
| Corridor | `Corridor · phase {n}` when active |
| Demolish | `DEMOLISH · hover` |

---

## 2. Rules

| Rule | Spec |
|:---|:---|
| Single line | max 48 chars — truncate reason |
| Invalid | red word `Blocked` — not fill-only |
| No duplicate | staged panel owns detail — strip is summary |

---

## Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer` | **PASS** | 2026-06-20 |
