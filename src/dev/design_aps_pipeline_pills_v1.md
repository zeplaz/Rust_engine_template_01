# DES-APS-PIPELINE-PILLS-001 — Pipeline pill validity copy `v1`

| Field | Value |
|:---|:---|
| **ID** | **DES-APS-PIPELINE-PILLS-001** |
| **Pairs** | [`design_aps_chrome_mockup_spec_v1.md`](design_aps_chrome_mockup_spec_v1.md) §3 |
| **Date** | 2026-06-16 |
| **Owner** | `@designer` |
| **Verdict** | **PASS** |

---

## Canonical state words (all lanes)

| State key | Glyph | State word | Pill bg | fg token |
|:---|:---:|:---|:---|:---|
| `pending` | `○` | `pending` | `#ffffff` | `COLOR_MUTED` |
| `saved_qc_not_run` | `◐` | `saved (QC not run)` | `#fff8ee` | `COLOR_WARN` |
| `valid` | `✓` | `valid` | `#f0faf0` | `COLOR_PASS` |
| `fail` | `✗` | `FAIL` | `#fff0f0` | `COLOR_FAIL` |
| `stamp_pending` | `○` | `Stamp pending` | `#ffffff` | `COLOR_MUTED` |
| `stamp_done` | `✓` | `Stamp registered` | `#f0faf0` | `COLOR_PASS` |

**Display template:** `{glyph} {StepLabel} {state_word}`

Examples:
- `✓ Assembly valid`
- `◐ Assembly saved (QC not run)`
- `○ Atlas pending`
- `○ Stamp pending`

---

## Buildings lane pills

| Step key | Label | pending | saved_qc_not_run | valid | fail |
|:---|:---|:---|:---|:---|:---|
| `catalog` | Catalog | `○ Catalog pending` | — | `✓ Catalog valid` | — |
| `assembly` | Assembly | `○ Assembly pending` | `◐ Assembly saved (QC not run)` | `✓ Assembly valid` | `✗ Assembly FAIL` |
| `materials` | Materials | `○ Materials pending` | `◐ Materials partial` | `✓ Materials valid` | — |
| `variants` | Variants | `○ Variants pending` | `◐ Variants saved (QC not run)` | `✓ Variants valid` | — |
| `atlas` | Atlas | `○ Atlas pending` | `◐ Atlas packed (QC not run)` | `✓ Atlas valid` | `✗ Atlas FAIL` |

### Buildings validity rules (@coder-mcp)

| Step | `valid` when | `saved_qc_not_run` when |
|:---|:---|:---|
| catalog | module selected + GLB validate PASS | — |
| assembly | `assembly_p0_passed is True` | snapshot on disk, P0 not run / None |
| materials | all placements have `material_profile` | some missing |
| variants | `variant_set` schema valid | file saved, not validated |
| atlas | atlas QC PASS | folder/atlas present, QC not run |

**Assembly P0 fail:** use `fail` row — `✗ Assembly FAIL` (not green).

---

## Landscape lane pills

| Step key | Label | pending | saved_qc_not_run | valid | fail |
|:---|:---|:---|:---|:---|:---|
| `presets` | Presets | `○ Presets pending` | `◐ Presets loaded (QC not run)` | `✓ Presets valid` | `✗ Presets FAIL` |
| `grammar` | Grammar | `○ Grammar pending` | `◐ Grammar saved (QC not run)` | `✓ Grammar valid` | `✗ Grammar FAIL` |
| `states` | States | `○ States pending` | `◐ States saved (QC not run)` | `✓ States valid` | — |
| `atlas` | Atlas | `○ Atlas pending` | `◐ Atlas packed (QC not run)` | `✓ Atlas valid` | `✗ Atlas FAIL` |
| `stamp` | Stamp | `○ Stamp pending` | — | `✓ Stamp registered` | `✗ Stamp FAIL` |

**Stamp** = map-stamp / `tile-atlas-register` complete — **landscape only**, fifth pill after Atlas.

### Landscape G0–G5 (atlas pill detail — tooltip only)

Do not flatten to one green. Tooltip on Atlas pill when `saved_qc_not_run`:

```text
Atlas packed — art-ship G4/G5 not run. See States tab.
```

When `valid` on atlas pill, tooltip may list: `G0 schema ✓ · G3 stills ✓ · G5 ship ✓` (scope-explicit).

---

## Lane hint line (not a pill)

| Lane | Text |
|:---|:---|
| Buildings | `Keyframe bake is behind Atlas — Assembly/Materials/Preview work without ship proof.` |
| Landscape | `LG-5 atlas art-ship (G4/G5) is separate from schema/bake green.` |

Font: `FONT_HINT` · fg `COLOR_MUTED` · right of pill row or wrapped below @960px.

---

## Tooltip keys (`aps_tooltips.py`)

```python
"pipeline_catalog": "Catalog step — module selected and GLB healthy.",
"pipeline_assembly": "Assembly snapshot saved and P0-validated.",
"pipeline_materials": "Every placement has material_profile.",
"pipeline_variants": "variant_set ready for tile batch.",
"pipeline_atlas": "PNG folder packed; QC run.",
"pipeline_presets": "Landscape preset selected and schema-valid.",
"pipeline_grammar": "topology_graph saved on preset.",
"pipeline_states": "Succession/disturbance variant rows ready.",
"pipeline_stamp": "Atlas registered for map stamp — engine can resolve UVs.",
```

---

## Migration from flat labels

| Old (`pipeline_status_bar`) | New pill |
|:---|:---|
| `✓ Catalog complete` | `✓ Catalog valid` |
| `◐ Assembly saved (P0 not run)` | `◐ Assembly saved (QC not run)` |
| `○ Variants pending` | unchanged pattern |
| `✓ Grammar saved` | `✓ Grammar valid` or `◐ Grammar saved (QC not run)` |

**Rename P0 → QC** in user-visible copy for cross-lane consistency (P0 still internal gate name in logs).

---

## Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer` | **PASS** | 2026-06-16 |

```text
DES-APS-PIPELINE-PILLS-001 Q✓ — copy locked for APS-E1-PIPELINE-LANE-001
```
