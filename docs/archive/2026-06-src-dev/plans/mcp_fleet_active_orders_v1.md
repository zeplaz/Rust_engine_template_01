# MCP fleet active orders `v1`

> **Paste prompts:** [`mcp_fleet_production_sprint_rowhouse_v1.md`](mcp_fleet_production_sprint_rowhouse_v1.md) · **Scope:** [`mcp_fleet_production_pilot_rowhouse_v1.md`](mcp_fleet_production_pilot_rowhouse_v1.md)

| Field | Value |
|:---|:---|
| **Program ID** | **MCP-ART-PROGRAM-GREEN-001** (closed) + **MCP-PROD-PILOT-ROWHOUSE-001** (active) |
| **Snap** | [`mcp_orchestrator_snap_CURRENT.md`](../../tools/orchestrator/queues/mcp_orchestrator_snap_CURRENT.md) |
| **Machine queue** | [`mcp_active_queue.json`](../../tools/orchestrator/queues/mcp_active_queue.json) |
| **Production tier plan** | [`plan_module_kit_production_tier_v1.md`](plan_module_kit_production_tier_v1.md) **SIGNED** |

---

## Program state (2026-06-03)

| Lane | Status |
|:---|:---|
| **Art program green** (lod0 + APS + TILE-REAL + TILE-ENGINE) | **CLOSED** |
| **Wave 2** (50 lod0, 7 style packs) | **CLOSED** |
| **PG-2** engine witness | **PASS** |
| **Production pilot** | **ACTIVE** — Victorian rowhouse only |

---

## Active dispatch (production pilot)

| Task | Agent | Status |
|:---|:---|:---|
| **MCP-PROD-G0-001** | @designer-mcp | **READY** — G0/G4 rowhouse only |
| **MCP-PROD-KIT-001** | @coder-mcp | **done** — `kit_production_001_live.json` G5 |
| **MCP-PROD-TILE-001** | @coder-mcp | **done** — `tile_tile_rowhouse_victorian_production_v1_live.json` G3 |
| **MCP-PROD-INDEX-001** | @coder-mcp | **done** — `_tile_atlas_index.ron` `ship_allowed: true` |

Full packets: [`mcp_fleet_production_pilot_rowhouse_v1.md`](mcp_fleet_production_pilot_rowhouse_v1.md)

---

## Unfrozen (narrow)

| ID | Scope |
|:---|:---|
| **`kit_production_001`** | 5 modules, `style_victorian`, rowhouse assembly slots |
| **`tile_rowhouse_victorian_production_v1`** | One production iso batch |

---

## Frozen

| Item | Notes |
|:---|:---|
| `kit_production_002+` | After rowhouse pilot closes |
| `kit_greybox_004+` | Smoke — never player |
| `tile.generate` (single-tile) | Use `tile_batch_run` |
| `tile_batch_warehouse_*` / `shopfront_*` / `bunker_*` production | Deferred — **designer do not open** |
| Full 50-module art acceptance | Not this pilot |

---

## Drained (on-call only)

@planner · @planner-mcp · @coder (PG-2/TILE-ENGINE done) · @designer (long-run P0–P5 PASS)

---

## Paste — @designer-mcp

> **MCP-PROD-G0-001** — `mcp_fleet_production_pilot_rowhouse_v1.md`. **Victorian rowhouse only.** G0 on production tile batch + variant set. Witness `rowhouse_production_g0_rules.yaml`. **Do not** touch warehouse/shopfront/bunker matrices.

---

## Paste — @coder-mcp

> **MCP-PROD-KIT-001** → **MCP-PROD-TILE-001** from `mcp_fleet_production_pilot_rowhouse_v1.md`. `batch_kit_production_001.manifest.json` then `tile_batch_rowhouse_victorian_production_v1.json`. Real bake, `development_tier: production`.

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v3.0.0 | 2026-06-03 | Program green closed; production pilot rowhouse active; unfreeze kit_production_001 (scoped) |
