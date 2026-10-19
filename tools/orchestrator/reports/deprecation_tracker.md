# Deprecation tracker

## Soft-deprecated defaults (agent orchestration — 2026-10-19)

| Item | Status | Replacement |
|:---|:---|:---|
| Mandatory full BLANG boot every session | **soft** | L0 lean / L1 program / L2 product-close tiers — `agent_orchestration_minimal_harness_v1.md` |
| `agent_flow_route` before every edit | **soft** | Only before Task/multi-owner dispatch |
| Ops/DSM scan on every cargo check | **soft** | Lane close / pre-arch / supervisor only |
| Parallel orchestrator personas as default entry | **soft** | Primary = Auto/@coder; personas kept as escalation |
| `*_drain_*` / `parallel_wave_*` as default chat queues | **soft** | `coder_active_queue.json` + `defer_registry.json` + HANDOFF; drain queues remain for fleet |
| Multi-parallel tracks as default routing | **soft** | Explicit escalate via `@coparent-orchestrator` |

_No JSON/agent file deletes in AGN-MIN-HARNESS-001._

## Compiler (last run)

_None._


## Static scan (`#[deprecated]`)

_None found._
