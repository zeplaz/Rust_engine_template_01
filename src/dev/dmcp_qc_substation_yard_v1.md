# Substation yard promoted GLB — artist QC `v1`

| Field | Value |
|:---|:---|
| **ID** | **DMCP-QC-SUBSTATION-001** |
| **Program** | PLAN-POWER-GRID-ART-ASSETS-001 |
| **Date** | 2026-06-18 |
| **Owner** | `@designer-mcp` |
| **Depends** | **MCP-PWR-PROMOTE-SUBSTATION-001** green |
| **Authority** | [`dmcp_spec_substation_yard_v1.md`](dmcp_spec_substation_yard_v1.md) · [`design_utility_industrial_style_v1.md`](design_utility_industrial_style_v1.md) |
| **GLB** | `assets/models/modules/kit_substation_yard_production_run001/model.glb` |
| **Witness** | `debug_runs/art_pipeline/dmcp_qc_substation_live.json` |
| **Verdict** | **PASS WITH NOTES** — promoted teach composite · G4 manual stills open |

```yaml
order_critique:
  request_summary: "Artist QC promoted substation vs spec + utility style bible"
  rules_audit:
    catalog_wired: pass
    footprint_4x3: pass
    asset_glb_validate: pass
    silhouette_64px: partial — teach-tier composite
  proceed: yes_with_notes
```

---

## 1. Scope

Post-promote **human + machine** QC for `grid_substation` production GLB — not a re-spec.

---

## 2. Must-read checks (operator)

| # | Question | Pass if |
|:---:|:---|:---|
| Q1 | **Footprint** | Reads as **4×3** yard in iso — matches catalog |
| Q2 | **Bus / breaker** | Low horizontal mass + **vertical bus or breaker** silhouette @ 64px |
| Q3 | **Fence rhythm** | Perimeter **chainlink rhythm** — not solid wall |
| Q4 | **Materials** | Galvanized + gravel + sparse yellow warning — no residential brick |
| Q5 | **Zone read** | Open yard center · service shack secondary |
| Q6 | **Map glyph** | Pairs `node_substation` — wider than transformer |

---

## 3. Machine checks (automated witness)

| Check | Rule |
|:---|:---|
| Promote witness | `mcp_pwr_substation_promote_live.json` → `green: true` |
| GLB on disk | `model.glb` under `kit_substation_yard_production_run001/` |
| `validate-report asset_glb` | `status: passed` |
| Catalog | `procedural_module_id` = `kit_substation_yard_production_001` |
| Grid | `building_size_x/y` = **4 / 3** |

**Teach-tier flag:** `vertex_count < 200` → composite stub — not art-ship final.

---

## 4. Keyframe stills (G4 manual — not blocking promote)

| Still | Zoom | Must show |
|:---|:---|:---|
| `substation_yard_iso_64_clean.png` | 64px iso | Bus + fence perimeter |
| `substation_yard_iso_64_night.png` | 64px iso | Yard flood optional |
| `substation_yard_iso_32_glyph.png` | 32px | `node_substation` parity |

Folder: `assets/staging/keyframe_stills/utility_power/substation_yard/`

---

## 5. Pass / fail

| Verdict | When |
|:---|:---|
| **PASS** | Q1–Q6 green · machine checks green · verts ≥200 · manual stills on disk |
| **PASS WITH NOTES** | Machine + Q1/Q4 green · teach composite · G4 stills deferred |
| **FAIL** | Missing GLB · catalog drift · validate fail · reads as factory hall |

**Current:** **PASS WITH NOTES** (teach composite, 48 verts).

---

## 6. Handoff

| Owner | Next |
|:---|:---|
| **Operator** | G4 stills §4 |
| **@coder-mcp** | Constituent module bpy refresh when art-ship |
| **@designer** | Overlay damaged stroke on yard (DES-ART-POWER-OVERLAY) |

---

## 7. Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer-mcp` | **PASS WITH NOTES** | 2026-06-18 |
