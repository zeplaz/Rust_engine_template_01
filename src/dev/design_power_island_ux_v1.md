# Power island player UX `v1`

| Field | Value |
|:---|:---|
| **ID** | **DES-POWER-ISLAND-UX-001** |
| **Program** | PLAN-POWER-GRID-CONSTRUCTION-UX-001 · Track D |
| **Date** | 2026-06-20 |
| **Owner** | `@designer` |
| **Depends** | [`design_power_map_overlay_v1.md`](design_power_map_overlay_v1.md) · IND-E03 grid overload pattern |
| **Coder** | `COD-POWER-ISLAND-TOAST-001` (wired) |
| **Verdict** | **PASS** |

```text
DES-POWER-ISLAND-UX-001 Q✓
Map dim + ops strip + toast — island never color-only
```

---

## 1. Map read

| Element | Spec |
|:---|:---|
| Island boundary | gold dashed ring on overlay |
| Unpowered buildings | dim 40% while island active |
| Live lines outside island | unchanged stroke |

---

## 2. Ops strip (PWR zone)

```text
PWR  ○ Island — N offline
```

| Rule | Spec |
|:---|:---|
| Duration | while island highlight active + toast window |
| Word | always `Island` — not icon alone |
| Count | offline building count from graph |

---

## 3. Toast (optional banner)

```text
Power island — N buildings offline
```

360 sim ticks · stacks with grid overload toast — island wins PWR strip priority.

---

## Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer` | **PASS** | 2026-06-20 |
