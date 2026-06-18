# Rowhouse production v2 operator QC `v1` — DMCP-TILE-ROWHOUSE-V2-001

| Field | Value |
|:---|:---|
| **Gate** | **DMCP-TILE-ROWHOUSE-V2-001** |
| **Program** | MCP production spine · Victorian rowhouse |
| **Owner** | `@designer-mcp` |
| **Date** | 2026-06-02 |
| **Batch** | [`tile_batch_rowhouse_victorian_production_v1.json`](../tools/mcp/schemas/examples/tile_batch_rowhouse_victorian_production_v1.json) |
| **Variant set** | [`variant_set_rowhouse_victorian_production_v1.json`](../tools/mcp/schemas/examples/variant_set_rowhouse_victorian_production_v1.json) |
| **Verdict** | **PASS** — damage + 8-frame burn operator-visible |

```yaml
order_critique:
  request_summary: "Operator can see damage + burning progression in APS/atlas"
  rules_audit:
    variant_completeness: pass
    atlas_registration: pass
    fire_frame_sequence: pass
  proceed: yes
```

---

## 1. Operator-visible variant matrix

| Group | Keys | Operator read |
|:---|:---|:---|
| **Clean** | `clean_day`, `clean_night_off`, `clean_night_on` | Baseline operational |
| **Damage** | `damaged_day`, `damaged_night_on` | Mid damage · half fill |
| **Construction** | `under_construction_01` | Early build scaffold read |
| **Fire** | `burning_00` … `burning_07` | **8-frame** progression · `sim_fire_frame_*` tags |

**Required for v2:** damage group + full burn ladder — **present** in production variant set (14 tiles).

---

## 2. Atlas / staging authority

| Field | Value |
|:---|:---|
| `atlas_id` | `rowhouse_victorian_production_v1` |
| `tile_px` | 128 |
| `variant_count` | **14** |
| Staging folder | `assets/staging/tiles/tile_rowhouse_victorian_production_v1/` |
| Keyframe stills | `assets/staging/tiles/keyframe_stills/rowhouse_victorian/` (canonical G4) |

---

## 3. Operator rubric (APS Catalog / RT lookup)

| # | Question | Pass |
|:---:|:---|:---:|
| 1 | Can operator pick `damaged_day` without debug JSON? | yes — variant_key in atlas |
| 2 | Burn frames labeled `burning_00`–`07` not `burning_8`? | yes |
| 3 | Night burn uses `night_on` lighting lane? | yes |
| 4 | `sim_fire` tags on all burn variants? | yes |
| 5 | Damage monotonic across burn frames? | yes — 0.55→0.69 |

---

## 4. v2 delta vs pilot

| Pilot | Production v2 |
|:---|:---|
| Greybox ortho seeds | Keyframe pack + production atlas |
| Partial burn set | **8** fire frames |
| `ship: false` frozen | Production tier · RT registry |

---

## 5. Handoff

- **@coder-mcp:** maintain `tile_rowhouse_victorian_production_v1` batch on grammar edits
- **@designer:** `DES-STYLE-VICTORIAN-ROW-001` for bay rhythm polish (parallel)

DMCP-TILE-ROWHOUSE-V2-001 Q✓
