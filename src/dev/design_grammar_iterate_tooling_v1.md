# Designer grammar iterate tooling `v1`

| Field | Value |
|:---|:---|
| **ID** | **DESIGNER-GRAMMAR-ITERATE-TOOLING-001** |
| **Program** | PLAN-APS-GRAMMAR-EVOLUTION-001 |
| **Owner** | `@designer-mcp` (content) · `@designer` (APS IA) |
| **Date** | 2026-06-18 |

---

## Rule

**Grammars mature through tool loops, not chat drafts.** Each edit cycle ends in structured JSON (`tier`, `gaps`, `next_actions`, sweep histograms) — agents reason on fields only (validation-first).

---

## Iteration loop

```text
1. designer_grammar_quality_loop (fast)
2. grammar-facility-brief --grammar-id <id>   (when CMCP-GRAMMAR-FACILITY-BRIEF-001 ships)
3. Edit assets/configs/buildings/grammars/*.ron + JSON mirrors + pilot catalog + site pilots
4. validate-report arch_build_grammar <preset.json>
5. validate-report site_zone_grid <site.json>   (when CMCP-SITE-ZONE-VALIDATE-001 ships)
6. grammar_preset_pair_validate --preset-id <id>
7. grammar_eval_sweep --archetype <id> --district <style>  (+ process histogram when shipped)
8. designer_grammar_quality_loop --full --write-witness
9. grammar-set-tier --write-witness
```

**Facility grammar program:** [`plan_industrial_facility_grammar_suite_v1.md`](plan_industrial_facility_grammar_suite_v1.md)

Stop when `tier` matches target **and** `green: true` on the full loop.

---

## Tier targets (content)

| Tier | Bar | Designer deliverable |
|:---|:---|:---|
| **G0** | 1 archetype generates | Baseline (`industrial_warehouse_v1.ron`) |
| **G1** | ≥3 archetypes or ≥3 districts/lineage | [`design_grammar_archetype_family_g1_v1.md`](design_grammar_archetype_family_g1_v1.md) |
| **G2** | ≥4 presets, ≥4 F-axis values | ARCH-DNA preset family |
| **G3** | facade + detail + age in rule_chain | Layer depth in RON |
| **G4** | coverage + parity green | Building-set ship gate |

---

## Commands

```powershell
# Fast (~2s)
powershell tools/mcp/scripts/designer_grammar_iterate.ps1

# Full + witness
powershell tools/mcp/scripts/designer_grammar_iterate.ps1 -Mode full -WriteWitness

cd tools/mcp/python
python -m rust_engine_mcp.cli grammar-set-tier --write-witness
python -m rust_engine_mcp.cli grammar-eval-sweep --archetype IndustrialWarehouse --district industrial_west
```

**MCP:** `designer_grammar_quality_loop_tool`, `grammar_set_tier_tool`, `grammar_eval_sweep_tool`, `grammar_set_brief`.

---

## Witness paths

| Path | When |
|:---|:---|
| `debug_runs/designer_grammar_quality_loop_live.json` | After full loop |
| `debug_runs/grammar_set_tier_live.json` | Tier check |
| `debug_runs/grammar_set_brief_live.json` | Inventory / gaps |

---

## APS split

| Agent | Owns |
|:---|:---|
| **@designer-mcp** | RON content, sweeps, tier witnesses |
| **@designer** | Exposure map (what APS shows per tier) |
| **@coder-mcp** | `apply_grammar_tier` in APS panels |
