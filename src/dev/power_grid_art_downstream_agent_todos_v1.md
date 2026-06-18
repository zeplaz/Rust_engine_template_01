# Power grid art — downstream agent board

| Field | Value |
|:---|:---|
| **Program** | PLAN-POWER-GRID-ART-ASSETS-001 |
| **Queue** | [`power_grid_art_downstream_queue.json`](../tools/orchestrator/queues/power_grid_art_downstream_queue.json) |
| **Plan** | [`plan_power_grid_art_assets_v1.md`](plan_power_grid_art_assets_v1.md) |
| **Rule** | Spec signed ≠ shipped — bpy + promote + validate_asset |

---

## Mode: on-call absorption (ORCH-PWR-DOWNSTREAM-001 done)

**No standing picks.** Queue sequenced; slices activate on notify only.  
Absorption: [`designer_oncall_absorption_v1.md`](designer_oncall_absorption_v1.md)

| Trigger | Absorb |
|:---|:---|
| Orchestrator activates slice | MCP-PWR-UTILITY-MANIFEST → bpy → promote |
| Promote lands | DMCP-QC-SUBSTATION / DMCP-QC-TRANSFORMER |
| Assets in `assets/models/modules/` | DESIGN-PROC-ART-ACCEPTANCE-001 |
| Product HUD request | COD-ART-HUD-ICON-ATLAS-001 |

## Parallel picks (frozen — not issued)

| Agent | ID | Task |
|:---|:---|:---|
| **@coder-mcp** | **MCP-PWR-UTILITY-MANIFEST-001** | Batch manifest + module spec stubs |
| **@designer-mcp** | **DMCP-SPEC-NUCLEAR-PWR-001** | Nuclear 6×6 AssetSpec from massing handoff |
| **@coder B** | **COD-ART-HUD-ICON-ATLAS-001** | power_hud_atlas PNG + RON + Rust registration |

---

## @coder-mcp — bpy + promote chain

| # | ID | Depends | Exit |
|:---|:---|:---|:---|
| 1 | MCP-PWR-UTILITY-MANIFEST-001 | ORCH | manifest + spec validate |
| 2a | MCP-PWR-SUBSTATION-BATCH-001 | manifest | staging GLB + asset_glb |
| 2b | MCP-PWR-TRANSFORMER-BATCH-001 | manifest | staging GLB (parallel) |
| 3a | MCP-PWR-PROMOTE-SUBSTATION-001 | 2a | models/modules + catalog |
| 3b | MCP-PWR-PROMOTE-TRANSFORMER-001 | 2b | supersedes lod0 stub |

**Upstream specs (done):** DMCP-SPEC-SUBSTATION-YARD-001 · DMCP-SPEC-TRANSFORMER-PAD-001

---

## @designer-mcp

| ID | Deliverable |
|:---|:---|
| **DMCP-SPEC-NUCLEAR-PWR-001** | `kit_nuclear_pwr_production_001.json` + dmcp doc |
| DMCP-QC-SUBSTATION-001 | After promote — artist QC |
| DMCP-QC-TRANSFORMER-001 | After promote — artist QC |

---

## @coder B

| ID | Deliverable |
|:---|:---|
| **COD-ART-HUD-ICON-ATLAS-001** | `power_hud_atlas.png` · RON · `icon_atlas.rs` wire |

---

## Definition of done (this wave)

- [ ] Substation + transformer **production GLBs promoted**
- [ ] Catalog JSON points at `assets/models/modules/.../model.glb`
- [ ] Nuclear **spec signed** (bpy deferred)
- [ ] HUD power icons in atlas + lib tests green
- [ ] Close witness WIT-HON pass

```text
[/PWR-DOWNSTREAM] manifest + nuclear spec + HUD parallel · then bpy→promote
```
