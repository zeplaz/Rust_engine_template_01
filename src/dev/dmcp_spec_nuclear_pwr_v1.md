# DMCP spec — nuclear PWR plant kit `v1`

| Field | Value |
|:---|:---|
| **ID** | **DMCP-SPEC-NUCLEAR-PWR-001** |
| **Program** | PLAN-POWER-GRID-ART-ASSETS-001 · Lane A2 |
| **Date** | 2026-06-02 |
| **Owner** | `@designer-mcp` |
| **Depends** | **DES-ART-NUCLEAR-PLANT-001** PASS · **DES-ART-UTILITY-STYLE-001** PASS |
| **Plant registry** | `pwr_4loop_1100mw_v1` in [`plant_definitions.json`](../../assets/config/power/plant_definitions.json) |
| **Catalog (planned)** | `nuclear_plant_pwr` — building JSON not on disk yet |
| **Staging spec** | [`kit_nuclear_pwr_production_001.json`](../../assets/staging/specs/kit_nuclear_pwr_production_001.json) |
| **Verdict** | **PASS** (spec only — **no bpy** until witness green) |

```yaml
order_critique:
  request_summary: "6×6 PWR site AssetSpec from designer massing handoff"
  rules_audit:
    no_ai_generated_images: pass
    deterministic_output: pass
    catalog_authority: pass_with_note  # plant_definitions + planned catalog id
    no_bpy_in_spec_wave: pass
  proceed: yes
  blocker_for_bpy: none — spec witness authorizes MCP-PWR-NUCLEAR-BATCH-001 pick
```

```text
DMCP-SPEC-NUCLEAR-PWR-001 Q✓
6×6 PWR kit — containment hero + aux modules — spec sign-off only
```

---

## 1. AssetSpec summary

| Field | Value |
|:---|:---|
| `asset_id` | `kit_nuclear_pwr_production_001` |
| `archetype` | `module_kit` |
| `style_pack` | `style_industrial_west` + utility bible §1 |
| `development_tier` | `production` |
| `grid_units` | **[6, 6]** |
| `utility_role` | `nuclear` |
| `power_tier` | `grid` |
| `plant_definition_id` | `pwr_4loop_1100mw_v1` |
| `batch_id` | `kit_nuclear_pwr_production_001` |

**Height budget** (from massing): dome apex ≤3u · hall ≤1.5u · cooling ≤2.5u.

---

## 2. Silhouette (@ 64px iso)

**Must read:** dome (containment) + turbine hall (low box) + cooling tower/bank.

Authority: [`design_nuclear_plant_massing_v1.md`](design_nuclear_plant_massing_v1.md) §1.

---

## 3. Site zones

| Zone | Target % | Kit content |
|:---|:---:|:---|
| **primary** | 40% | `containment_dome_pwr` 3×3 + `turbine_hall_1u` ×2 |
| **utility** | 25% | `switchyard_edge_1u` ×4 · `cooling_tower_1u` ×2 · `warning_sign_nuclear_1u` ×2 |
| **service** | 15% | `diesel_gen_pad_2x2` ×1 |
| **buffer** | 20% | `fence_chainlink_1u` perimeter (reuse utility batch) |

Aligns [`design_facility_site_zone_taxonomy_v1.md`](design_facility_site_zone_taxonomy_v1.md).

---

## 4. Module whitelist (Phase 1)

| Module id | Grid | Zone | Notes |
|:---|:---|:---|:---|
| **`containment_dome_pwr`** | **3×3** | primary | **Hero** — cylindrical dome, concrete |
| `turbine_hall_1u` | 2×1 | primary | Repeat for hall length |
| `cooling_tower_1u` | 1×1 | utility | `cooling_bank_2u` variant deferred |
| `diesel_gen_pad_2x2` | 2×2 | service | Backup — `icon_diesel` adjunct |
| `switchyard_edge_1u` | 1×1 | utility | Bus + fence segment |
| `fence_chainlink_1u` | 1×1 | buffer | Reuse `kit_utility_power_production_001` |
| `warning_sign_nuclear_1u` | 1×1 | utility | Yellow + simplified trefoil |

---

## 5. Materials

Per utility bible + nuclear massing:

| Profile | Use |
|:---|:---|
| `concrete_grey_01` | containment dome, turbine hall base |
| `galvanized_steel_01` | switchyard, hall cladding accents |
| `gravel_yard_01` | setback buffer |
| `warning_paint_yellow_01` | nuclear signs |

**Ban:** residential brick · decorative façade windows on containment · photoreal trefoil texture.

---

## 6. State read (gameplay pairing)

| State | Visual |
|:---|:---|
| Normal | Dome + steady stack |
| SCRAM | Amber dome wash + `icon_scram` |
| Meltdown | Column VFX + danger adjunct |
| Diesel run | Aux pad exhaust |

Pairs [`power_glyphs_spec_v1.md`](../../assets/ui/infrastructure/power_glyphs_spec_v1.md) `node_plant_nuclear_*`.

---

## 7. MCP pipeline (blocked until this spec Q✓)

| Step | Owner | Tool |
|:---|:---|:---|
| Spec witness | @designer-mcp | `dmcp-nuclear-pwr-spec-witness` |
| Per-module specs | @designer-mcp | `*_production_run001.json` stubs (batch wave) |
| Validate | @coder-mcp | `validate_asset_spec` |
| Generate | @coder-mcp | `blender_batch_modules` — **MCP-PWR-NUCLEAR-BATCH-001** |
| Promote | @coder-mcp | registry + planned `nuclear_plant_pwr.json` |

**This wave:** spec JSON + dmcp doc only — `bpy_blocked: true` on AssetSpec.

---

## 8. Handoff checklist (from massing §5)

- [x] AssetSpec `kit_nuclear_pwr_production_001` — 6×6 site assembly
- [x] `containment_dome_pwr` hero module listed (3×3)
- [x] Module whitelist from massing §3
- [x] Staging witness under `debug_runs/art_pipeline/`
- [ ] Per-module bpy jobs — **@coder-mcp** after batch pick
- [ ] `nuclear_plant_pwr.json` catalog — **@coder** with activation bridge

---

## 9. Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer` | PASS (massing) | 2026-06-18 |
| `@designer-mcp` | **PASS** (spec only) | 2026-06-02 |

**Unblocks:** `MCP-PWR-NUCLEAR-BATCH-001` · `PWR-ART-DOWNSTREAM-CLOSE-001` (partial)
