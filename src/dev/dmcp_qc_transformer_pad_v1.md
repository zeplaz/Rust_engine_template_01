# Transformer pad promoted GLB — artist QC `v1`

| Field | Value |
|:---|:---|
| **ID** | **DMCP-QC-TRANSFORMER-001** |
| **Program** | PLAN-POWER-GRID-ART-ASSETS-001 |
| **Date** | 2026-06-18 |
| **Owner** | `@designer-mcp` |
| **Depends** | **MCP-PWR-PROMOTE-TRANSFORMER-001** green |
| **Authority** | [`dmcp_spec_transformer_pad_v1.md`](dmcp_spec_transformer_pad_v1.md) · [`design_utility_industrial_style_v1.md`](design_utility_industrial_style_v1.md) |
| **GLB** | `assets/models/modules/prop_transformer_production_run001/model.glb` |
| **Witness** | `debug_runs/art_pipeline/dmcp_qc_transformer_live.json` |
| **Verdict** | **PASS** — production mesh · 3-bushing read @ 32px |

```yaml
order_critique:
  request_summary: "Artist QC transformer — cylinder + 3 bushings @ 32px"
  rules_audit:
    supersedes_lod0: pass
    asset_glb_validate: pass
  proceed: yes
```

---

## 1. Scope

Post-promote QC for `grid_distribution_transformer` — supersedes `prop_transformer_lod0_run001`.

---

## 2. Must-read checks (operator)

| # | Question | Pass if |
|:---:|:---|:---|
| Q1 | **Footprint** | **2×2** pad · tank centered |
| Q2 | **Tank** | Horizontal **cylinder** — galvanized read |
| Q3 | **Bushings** | **3** ceramic caps on top — **white dots @ 32px** |
| Q4 | **Pad** | Concrete slab + gravel margin |
| Q5 | **Glyph parity** | Matches `node_transformer` — cylinder + 3 dots |
| Q6 | **LOD0 superseded** | Production path in catalog · lod0 not default |

---

## 3. Machine checks (automated witness)

| Check | Rule |
|:---|:---|
| Promote witness | `mcp_pwr_transformer_promote_live.json` → `green: true` |
| Supersedes lod0 | both production + lod0 GLB exist |
| `validate-report asset_glb` | `status: passed` |
| Catalog | `procedural_module_id` = `prop_transformer_production_run001` |
| Grid | `building_size_x/y` = **2 / 2** |
| Geometry proxy | `vertex_count >= 200` (cylinder mesh, not box stub) |

---

## 4. Keyframe still (operator sign)

| Still | Zoom | Must show |
|:---|:---|:---|
| `transformer_pad_iso_32.png` | 32px iso | 3 bushings as distinct white caps |

Folder: `assets/staging/keyframe_stills/utility_power/transformer_pad/`

---

## 5. Pass / fail

| Verdict | When |
|:---|:---|
| **PASS** | Q1–Q6 green · machine checks green |
| **PASS WITH NOTES** | Machine green · Q3 bushing read marginal @ 32px — still ship |
| **FAIL** | Box stub only · missing bushings · catalog still points lod0 |

**Current:** **PASS** (792 verts · validate passed).

---

## 6. Handoff

| Owner | Next |
|:---|:---|
| **@coder** | Map overlay snap to transformer pad |
| **@designer** | HUD `icon_transformer_place` atlas tile |

---

## 7. Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer-mcp` | **PASS** | 2026-06-18 |
