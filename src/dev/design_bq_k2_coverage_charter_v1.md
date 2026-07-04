# BQ-K2 slot coverage charter `v1` — BQ-K2-COVERAGE-001

| Field | Value |
|:---|:---|
| **ID** | **BQ-K2-COVERAGE-001** |
| **Issue** | BQ-K2 |
| **Parent** | [`plan_building_quality_v1.md`](plan_building_quality_v1.md) § Phase K |
| **Date** | 2026-07-03 |
| **Owner** | `@designer-mcp` (audit charter) → `@coder-mcp` / APSR-Q2 panel |
| **Status** | **SIGNED** |
| **Verdict** | **PASS** — slot resolution 100%; style purity gaps tracked pending K1 wire |
| **Audit module** | [`kit_coverage_audit.py`](../../tools/mcp/python/rust_engine_mcp/kit_coverage_audit.py) |
| **Witness** | [`debug_runs/bq_k2_coverage_001_live.json`](../../debug_runs/bq_k2_coverage_001_live.json) |

```yaml
order_critique:
  request_summary: "100% standard slot coverage audit per style pack + K1 purity gap ledger"
  rules_audit:
    pytest_auditable: pass
    apsr_q2_consumer: pass
    no_silent_hide_slot: pass
  blocked: false
  proceed: yes
```

---

## 0. Two coverage layers

| Layer | Meaning | Gate owner |
|:---|:---|:---|
| **Slot resolution** | Every declared slot key resolves to a GLB-ready module (cross-style fallback OK today) | This charter — **green now** |
| **Style purity** | Slot module id matches material family of the style pack (BQ-K1 targets) | Closes when `@coder-mcp` wires K1 batch |

Do not collapse these — APSR-Q2 Kit panel shows resolution %; BQ-A2 style purity % is separate.

---

## 1. Standard slot matrix

**Core (all packs):** `wall_1u`, `door_default`, `window_1u`, `roof_default`

**Extended (when pack declares them):** `wall_2u`, `window_2u`, `door_wide`, `roof_flat`, `corner_outer`, `corner_inner`, `prop_clutter`, `window_industrial`, `roof_industrial`

Audit verifies: (a) required keys present per pack profile; (b) each bound module resolves in module index with production GLB.

---

## 2. Style pack profiles

| Pack | Required beyond core |
|:---|:---|
| style_colonial | wall_2u, window_2u, door_wide, roof_flat, corner_outer |
| style_victorian | wall_2u, window_2u, roof_flat, corner_outer, prop_clutter |
| style_rural | wall_2u, window_2u, door_wide, roof_flat, prop_clutter |
| style_modern | wall_2u, window_2u, window_industrial, roof_flat, prop_clutter |
| style_military | wall_2u, roof_flat, corner_outer, prop_clutter |
| style_industrial_west | wall_2u, door_wide, window_industrial, roof_industrial, roof_flat, corner_outer, prop_clutter |
| style_industrial_soviet | wall_2u, door_wide, window_industrial, roof_flat, prop_clutter (`window_industrial` satisfies `window_1u`) |

---

## 3. Acceptance (designer-mcp gate)

| # | Criterion |
|:---:|:---|
| K2-A | Charter + audit module + pytest |
| K2-B | 7/7 style packs slot resolution 100% |
| K2-C | `style_purity_gaps` ledger populated from K1 `replaces_slots` |
| K2-D | APSR-Q2 witness hook (`write_apsr_q2_witness`) unchanged consumer contract |

**Exit for full BQ-K2 product gate:** `style_purity_gaps == 0` after K1 bake wire — tracked in witness, not blocking charter sign-off.

---

## 4. APSR-Q2 integration

[`catalog_kit_coverage_strip.py`](../../tools/mcp/art_pipeline_suite/catalog_kit_coverage_strip.py) consumes `format_kit_coverage_summary()` — no panel changes required for this charter.
