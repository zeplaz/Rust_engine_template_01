# Style pack registry map `v1`

| Field | Value |
|:---|:---|
| **ID** | **DES-STYLE-PACK-REGISTRY-001** |
| **Program** | PLAN-DESIGNER-WORK-202606-001 · Track C2 |
| **Date** | 2026-06-20 |
| **Owner** | `@designer` |
| **Authority** | `_module_index.ron` · style pack RON rows |
| **Verdict** | **PASS** |

```text
DES-STYLE-PACK-REGISTRY-001 Q✓
style_pack_id → bible → module whitelist → tile batch
```

---

## 1. Registry row

| Column | Source |
|:---|:---|
| `style_pack_id` | RON / grammar district |
| `visual_bible` | `design_style_*_v1.md` path |
| `module_whitelist` | `_module_index` filter |
| `tile_batch` | atlas batch id |
| `tier_min` | G0–G4 grammar gate |

---

## 2. Example rows

| pack_id | bible | batch |
|:---|:---|:---|
| `industrial_west` | `design_style_industrial_west_v1.md` | `kit_production_002` |
| `victorian_row` | `design_style_victorian_row_v1.md` | `tile_rowhouse_victorian` |

---

## Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer` | **PASS** | 2026-06-20 |
