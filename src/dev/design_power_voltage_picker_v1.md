# Power voltage picker `v1`

| Field | Value |
|:---|:---|
| **ID** | **DES-POWER-VOLTAGE-PICKER-001** |
| **Program** | PLAN-POWER-GRID-CONSTRUCTION-UX-001 · Track A |
| **Date** | 2026-06-18 |
| **Owner** | `@designer` |
| **Authority** | `VoltageClass` in [`src/infrastructure/utility/mod.rs`](../../src/infrastructure/utility/mod.rs) |
| **Overlay** | [`design_power_map_overlay_v1.md`](design_power_map_overlay_v1.md) §2 |
| **Verdict** | **PASS** |

```text
DES-POWER-VOLTAGE-PICKER-001 Q✓
Three player labels map 1:1 to VoltageClass — mismatch always names reason
```

---

## 1. Mapping (canonical)

| `VoltageClass` | Player label | Sheet radio | Strip short |
|:---|:---|:---|:---|
| `Low` | **Distribution** | `( ) Distribution` | `distribution` |
| `Medium` | **Medium voltage** | `(•) Medium` | `MV` |
| `High` | **Transmission** | `( ) Transmission` | `HV` |

**Sheet title:** `Power line — {player label}`

**Ban:** engineer strings `Low` / `Medium` / `High` in primary UI (tooltip OK).

---

## 2. Picker placement

| Rule | Spec |
|:---|:---|
| Location | Tool sheet row 2 — **before first LMB point** |
| Default | Last used `VoltageClass` per session |
| Change mid-draw | Allowed — **revalidate** all segments; invalid → red hatch |
| Locked after commit | N/A — new draw picks fresh |

---

## 3. Stroke & preview (live map)

| Class | Color | Weight | Preview |
|:---|:---|:---:|:---|
| Distribution | `#e8c040` | 2px | dashed @ 60% α |
| Medium | `#f0d050` | 3px | dashed @ 60% α |
| Transmission | `#ffd878` | 4px | dashed + subtle outer glow |

Committed **live** strokes: solid — see overlay spec.

---

## 4. Compatibility rules

| Rule | Valid | Blocked copy |
|:---|:---:|:---|
| HV → residential stub only | ✗ | `blocked: transmission requires substation step-down` |
| MV → factory bus | ✓ | — |
| Distribution → smelter main feed | ✗ | `blocked: smelter needs medium voltage or higher` |
| Class drop at transformer | ✓ | Auto if transformer supports step-down |
| Mismatch at tee | ✗ | `blocked: voltage mismatch at junction` |

**Authority:** node `max_voltage` from catalog/building — coder validates; strip shows designer copy only.

---

## 5. Endpoint hints (hover)

| Snap target | Allowed classes | Hint |
|:---|:---|:---|
| Distribution transformer pad | Low, Medium | `MV or distribution` |
| Grid substation | Medium, High | `MV or HV` |
| Factory smelter bus | Medium | `MV recommended` |
| Mine / light load | Low | `distribution` |

---

## 6. Copy registry

| Key | String |
|:---|:---|
| `power.voltage.distribution` | `Distribution` |
| `power.voltage.medium` | `Medium voltage` |
| `power.voltage.transmission` | `Transmission` |
| `power.blocked.mismatch` | `blocked: voltage mismatch at {node}` |
| `power.blocked.smelter` | `blocked: smelter needs medium voltage or higher` |
| `power.blocked.hv_residential` | `blocked: transmission requires substation step-down` |

---

## 7. Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer` | **PASS** | 2026-06-18 |

---

## 8. Exit predicate

| Field | Value |
|:---|:---|
| **Deliverable** | This spec on disk with **PASS** verdict |
| **Marker** | `DES-POWER-VOLTAGE-PICKER-001 Q✓` in header fence |
| **Registry** | `designer_signoff_registry.json` → **SIGNED** |
| **Unblocks** | `COD-POWER-LINE-DRAW-001` |
