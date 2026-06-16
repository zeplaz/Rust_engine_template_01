# OPS Witness Spine (Track D) — all program lanes

**SYMLANG:** `$ref:prompts/SYMBOLIC_LANGUAGE.meta.md` — DSM/ops paste blocks are **packets** (§3.2 lattice · §3.4 spine), not NL walls.

**All agents** — engine sim, construction, fire/VFX, infrastructure, economy, waves, **and** MCP art.

Registry: [`OPS_LANE_REGISTRY.json`](OPS_LANE_REGISTRY.json)

## Scan

```powershell
powershell -File tools/orchestrator/scripts/ops_intelligence_scan.ps1
```

## Read (structured — not HANDOFF prose alone)

| File | Fields |
|:---|:---|
| `debug_runs/agent_ops/ops_report_latest.json` | `dsm_snapshot`, `qce`, `delta_wf`, `program_summary` |
| `debug_runs/unified_witness_index.json` | `programs.<program_id>[]`, `construction_sub_witnesses` |

## Program lanes (`program_id`)

| program_id | Domain | Owner |
|:---|:---|:---|
| `stage5_spine` | FULL_APP readiness | `@coder` |
| `fire_vfx` | Fire ecology, streaming, GPU particles | `@coder` |
| `construction` | Phases, procedural build, scaling audit | `@coder` |
| `infrastructure` | Transport, VM, view isolation | `@coder` |
| `economy_logistics` | Industrial activation, logistics | `@coder` |
| `wave_product` | Wave S/P/C, Stage 6, WSS substrate | `@coder` |
| `stage7_play` | Behavioral sim, play scenarios | `@coder` |
| `ui_presentation` | UI shell, minimap | `@designer` / `@coder` |
| `weather` | Atmosphere sim — **parallel downtime** (Coder C) | [`plan_weather_parallel_lane_v1.md`](../../../docs/archive/2026-06-src-dev/plans/plan_weather_parallel_lane_v1.md) · witness [`plan_weather_witness_002_v1.md`](../../../docs/archive/2026-06-src-dev/plans/plan_weather_witness_002_v1.md) · `@coder` |
| `art_A` / `art_B` / `art_C` | MCP three-track | `@coder-mcp` / `@designer-mcp` |
| `agent_ops` | Orchestrator continuity | `@main-thread-orchestrator` |

## Write (new witnesses)

**Illustrative only — copy shape, not values.** Live files must match $ref:docs/archive/2026-06-src-dev/plans/witness_exec_shape_v1.md.

```json
{
  "gate": "CON-P3-WIT",
  "green": true,
  "operational_green": true,
  "profile": "CONSTRUCTION_STAGE",
  "_agent_meta": {
    "schema": "debug_run_envelope_v1",
    "profile": "CONSTRUCTION_STAGE",
    "source_system": "construction_stage_live_proof",
    "relative_path": "debug_runs/construction_stage_live.json",
    "written_at_epoch_secs": 0,
    "agent_commands": ["cargo test -p proc_A_dine01 construction:: --lib"],
    "related_proofs": ["debug_runs/stage5_full_app_live.json"],
    "orchestrator": {
      "continuation_queue": "tools/orchestrator/queues/continuation_queue.json",
      "ops_report_latest": "debug_runs/agent_ops/ops_report_latest.json"
    },
    "agent": "coder",
    "lane": "CON-P3-WIT",
    "program_id": "construction",
    "task_id": "CON-P3-WIT"
  }
}
```

Art ship witnesses also need `track`, `proceed_ship`, `art_quality`.

## Honest gates

| Class | Meaning | Action |
|:---|:---|:---|
| `honest_green` | Art + schema agree | May proceed ship path |
| `dishonest_gate` | Schema pass, art fail | **No re-queue** — operator ΔWF |
| `operational_green` | Construction spine green | Does not close infra tails |
| `readiness_green` | Stage5/6 readiness passes | Spine only |

## HANDOFF close event

Append to `debug_runs/agent_ops/events.jsonl` per `agent_run_event_v1.schema.json` with `program_id`.

```powershell
./tools/orchestrator/invoke_handoff.ps1 -Goal "..." -Lane Construction -TaskId CON-P3-WIT -Track construction -OpsScan -OpsEvent
```

Skill: [`.cursor/skills/operations-intelligence/SKILL.md`](../../../.cursor/skills/operations-intelligence/SKILL.md)
