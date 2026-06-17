# G4 list-staging sign workflow — landscape atlases `v1` (DMCP-G4-STAGING-SIGN-001)

| Field | Value |
|:---|:---|
| **Owner** | `@designer-mcp` |
| **Date** | 2026-06-17 |

---

## Workflow

1. **List** — `python -m rust_engine_mcp.cli list-staging --domain landscape`
2. **Validate** — `validate-report tile_batch <spec.json>`
3. **Honesty** — `validate-report witness_honesty debug_runs/art_pipeline/tile_<batch>_live.json`
4. **QC brief** — `atlas-meta-brief <meta_json>` (plain copy from [`design_aps_atlas_qc_copy_v1.md`](design_aps_atlas_qc_copy_v1.md))
5. **Sign** — designer-mcp sets `proceed_ship: yes|no` on domain signoff YAML
6. **Register** — coder-mcp upserts `_landscape_atlas_index.ron` only after step 5 yes

---

## Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer-mcp` | **PASS** | 2026-06-17 |
