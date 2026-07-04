# BQ-K1 kit fill charter `v1` — BQ-K1-KITFILL-001

| Field | Value |
|:---|:---|
| **ID** | **BQ-K1-KITFILL-001** |
| **Issue** | BQ-K1 |
| **Parent** | [`plan_building_quality_v1.md`](plan_building_quality_v1.md) § Phase K |
| **Date** | 2026-07-03 |
| **Owner** | `@designer-mcp` (charter + job specs) → `@coder-mcp` (G0–G5 bakes + style-pack wire) |
| **Status** | **SIGNED** |
| **Verdict** | **PASS** — 11 geometry job specs on disk |
| **Batch** | [`bq_k1_kitfill_batch_v1.json`](../../tools/mcp/schemas/examples/bq_k1_kitfill_batch_v1.json) |
| **Witness** | [`debug_runs/bq_k1_kitfill_001_live.json`](../../debug_runs/bq_k1_kitfill_001_live.json) |

```yaml
order_critique:
  request_summary: "Material-family kit holes — brick/wood/concrete roof+door+window modules"
  rules_audit:
    data_not_code: pass
    deterministic_seed_chain: pass
    module_contract_v1: pass
    no_bpy_in_charter: pass
  blocked: false
  proceed: yes
  handoff: "@coder-mcp bakes GLBs + rewires style_pack slot targets from replaces_slots map"
```

---

## 0. Problem (B1 style purity)

Colonial/Victorian brick packs still bind `roof_default` → cross-family modules (`roof_pitched_gable`, `door_civic`). Rural wood windows fall back to shop fronts. Modern/Military concrete roofs use generic flats. **BQ-F2** logs fallbacks; **BQ-K1** supplies the replacement module ids.

---

## 1. Priority order (plan authority)

| # | Material | Categories | Style packs |
|:---:|:---|:---|:---|
| 1 | brick | roof, door, window 1u/2u | colonial, victorian |
| 2 | wood | roof, window 1u/2u | rural |
| 3 | concrete | roof, door, window 1u/2u | modern, military |

**Job count:** 11 (within plan ~10–14 band).

---

## 2. Deliverables

| Artifact | Path |
|:---|:---|
| Batch manifest | `tools/mcp/schemas/examples/bq_k1_kitfill_batch_v1.json` |
| AssetSpecs | `assets/staging/specs/kit_fill/*_production.json` |
| Geometry job examples | `tools/mcp/schemas/examples/geometry_job_*_production_run001.json` |
| Catalog authority | `tools/mcp/python/rust_engine_mcp/bq_k1_kitfill_catalog.py` |

Each job declares `replaces_slots` — the style-pack keys `@coder-mcp` must rewire after promote.

---

## 3. Acceptance (designer-mcp gate)

| # | Criterion |
|:---:|:---|
| K1-A | Charter + batch manifest present |
| K1-B | 11 specs + 11 geometry jobs on disk |
| K1-C | Every job references `module_contract_v1` + material_family |
| K1-D | Seeds deterministic (`550100 + job_index`) |
| K1-E | `replaces_slots` maps every targeted pack slot |

**Not in scope for this gate:** GLB bake, module_index promotion, style_pack RON edits — `@coder-mcp` after sign-off.

---

## 4. `@coder-mcp` handoff checklist

1. Run geometry batch `kit_fill_bq_k1_001` through G0–G5 lane with BQ-C2/C3 validators.
2. Promote rows to production tier in module index.
3. Patch style pack RON slot targets per `replaces_slots` in batch manifest.
4. Re-run `dmcp-bq-k2-coverage-witness` — `style_purity_gaps` should drop to 0.
