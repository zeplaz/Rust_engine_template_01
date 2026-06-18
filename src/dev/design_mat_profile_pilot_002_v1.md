# Material profile pilot pack 002 `v1` — DMCP-MAT-PROFILE-PILOT-002

| Field | Value |
|:---|:---|
| **Gate** | **DMCP-MAT-PROFILE-PILOT-002** |
| **Program** | PLAN-DESIGNER-WORK-202606 · Track B3 |
| **Owner** | `@designer-mcp` |
| **Date** | 2026-06-02 |
| **Pack** | [`mat_profile_pilot_002_pack.json`](../assets/staging/specs/mat_profile_pilot_002_pack.json) |
| **IA authority** | [`design_aps_materials_tab_ia_v1.md`](../docs/archive/2026-06-src-dev/plans/design_aps_materials_tab_ia_v1.md) |
| **Verdict** | **PASS** (spec only — no registry merge) |

```yaml
order_critique:
  request_summary: "24-profile pilot stubs across APS category tree leaves"
  rules_audit:
    deterministic_output: pass
    category_tree_coverage: pass
    no_inline_texture_generation: pass
  proceed: yes
```

---

## 1. Pack summary

| Field | Value |
|:---|:---|
| `pack_id` | `mat_profile_pilot_002` |
| `profile_count` | **24** |
| `development_tier` | `pilot` |
| `generator` | `procedural_tile_v1` |
| `seed_base` | 42000 |

---

## 2. Category leaf coverage (24)

| Parent | Leaves in pack |
|:---|:---|
| **industrial** | steel ×2 · concrete ×2 · corrugated · gravel |
| **residential** | brick ×2 · wood ×2 · stucco · plaster |
| **commercial** | glass · stone · metal_panel |
| **civic** | limestone · brick_band |
| **roof** | tile · metal · slate |
| **glass** | clear · frosted |
| **utility** | warning_paint · galvanized |
| **landscape** | grass · soil · ash · char |

---

## 3. Rules

- Each `profile_id` is unique · seed-driven · no runtime randomness
- **Do not** merge into `material_profiles_v1.json` until `@coder-mcp` `validate-report material_profiles` green
- Landscape burn profiles deferred to **pilot 003** (LG-5 coupling)

---

## 4. Handoff

| Owner | Task |
|:---|:---|
| **@coder-mcp** | Validate pack JSON · procedural gen smoke |
| **@designer** | `DES-APS-MAT-BROWSE-001` tree labels for new leaves |

DMCP-MAT-PROFILE-PILOT-002 Q✓
