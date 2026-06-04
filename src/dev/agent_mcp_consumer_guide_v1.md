# Agent MCP consumer guide `v1` — use tools, do not build them

| Field | Value |
|:---|:---|
| **ID** | **AGENT-MCP-CONSUMER-001** |
| **Date** | 2026-06-02 |
| **Rule** | **Consumers** invoke MCP/CLI and read `ValidationReport` JSON. **Builders** (`@*-mcp`) own `tools/mcp/` implementation. |

---

## Agent lanes (who builds vs who uses)

| Agent | Builds MCP? | Uses MCP tools? | Skills to attach |
|:---|:---:|:---:|:---|
| **orchestrator-mcp** | Sequences gates | Invokes verify | mcp-asset-pipeline |
| **planner-mcp** | Plans schemas/tools | Rarely | mcp-production-rules |
| **designer-mcp** | AssetSpec + sign-off | **Yes** — primary executor | mcp-asset-pipeline · mcp-production-rules · validation-first |
| **coder-mcp** | Python/CLI/bpy | **Yes** | blender-geometry · validation-first |
| **orchestrator** | No | Routes to *-mcp | — |
| **planner** | No | No (charter only) | — |
| **coder** | No | **Yes** — validate only | **validation-first** |
| **designer** | No | **Yes** — request batches via *-mcp | validation-first (review) |
| **sim-steward** | No | validate-report for triage | validation-first |
| **coparent-orchestrator** | No | Routes art to orchestrator-mcp | — |

**Never:** `@coder` / `@designer` / `@planner` editing `tools/mcp/python/` or adding MCP server tools.

---

## Consumer workflows (copy into sessions)

### @coder (after `cargo test` / `cargo check`)

```powershell
python -m rust_engine_mcp.cli validate-report cargo --compress 3
python -m rust_engine_mcp.cli validate-report bevy -p proc_A_dine01
```

- Read `status`, `errors[]`, `known_fixes[]` — **not** raw terminal walls.
- After PROC-PG-2: confirm promoted GLBs exist; do **not** run Blender — ping `@designer-mcp` if assets missing.

### @designer (art acceptance, not bpy)

1. Charter module need in design doc (module_id, style pack, tier).
2. Hand off execution: **`@designer-mcp`** runs `geometry_run_job` + validate + promote.
3. Review staging sign-off YAML under `debug_runs/art_pipeline/`.
4. Consumer verify only:

```powershell
python -m rust_engine_mcp.cli validate-report asset_glb assets/models/modules/<path>.glb --compress 3
```

- **Reject** “green because file exists” — use validation-first tier rules (`smoke` vs `lod0` vs `production`).

### @planner

- Reference MCP in exec plans as **handoff targets** (`@designer-mcp` batch id, witness path).
- Do **not** specify bpy ops in planner docs — point to `plan_designer_mcp_art_toolchain_exec_001_v1.md` and module kit.

### @orchestrator / @coparent-orchestrator

- Art parallel lane → **`@orchestrator-mcp`** (G0–G5), not `@designer` chat bpy.
- Construction/product lane → **`@planner`** + **`@coder`** (no MCP unless PG-2 asset gap).

---

## Construction + growth ↔ MCP touchpoints

| Product slice | Sim owner (`@coder`) | Art owner (`@designer-mcp`) |
|:---|:---|:---|
| CON-P2 staged sites | commit + tick | — |
| PG-1 archetypes RON | loaders only | — |
| PG-2 greybox assembly | extract + instances | `kit_greybox_*` or `kit_production_*` batches |
| OG-1 pressure | district metrics + **market saturation** | — |
| OG-2 proposals | queue → commit | dashed ghost **UX** from `@designer` doc |
| State factory / civic | `BuildingUsage::Government/Industrial` | style pack `style_industrial_*` |
| Private infill | growth proposals | commercial/residential modules |

**Tier rule:** PG-2 may use **`lod0` / greybox** modules; player-visible polish requires **`production`** tier per [`plan_module_kit_production_tier_v1.md`](plan_module_kit_production_tier_v1.md).

---

## Escalation matrix

| Symptom | Route |
|:---|:---|
| Validator wrong / missing rule | `@coder-mcp` |
| AssetSpec / batch manifest | `@designer-mcp` |
| New tool category / schema | `@planner-mcp` |
| Gate order / parallel batches | `@orchestrator-mcp` |
| ECS registry / RepresentationResult | `@coder` + bevy-simulation-grade |
| Witness vs queue drift | `@sim-steward` + debug-intelligence |

---

## Setup (human once per machine)

```powershell
.\tools\mcp\install_designer_mcp.ps1
.\tools\mcp\scripts\verify_mcp_setup.ps1
```

Cursor: **rust-engine-art** MCP green · Python **3.13** in MCP env (not bare 3.14).

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-06-02 | Consumer vs builder split for fleet prompts |
