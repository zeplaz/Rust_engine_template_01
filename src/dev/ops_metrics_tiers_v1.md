# OPS metrics tiers `v1`

| Field | Value |
|:---|:---|
| **⟨ID⟩** | **OPS-METRICS-TIERS-001** |
| **Source** | $ref:docs/reference/outside/effwecny_mpc_draft.md (L1864–L2447) |
| **Parent** | $ref:src/dev/plan_ops_metalogic_split_v1.md |
| **Guard** | $ref:src/dev/ops_metrics_goodhart_guard_v1.md — **required** |

Store **distilled measurements**, not conversations. **No self-reported quality.**

---

## Tier 0 — raw run row (ingest)

| Field | Type | Notes |
|:---|:---|:---|
| `run_id` | uuid | |
| `agent_id` | text | `@coder`, … |
| `model` | text | |
| `prompt_hash` | text | |
| `task_type` | text | `⟨ID⟩` |
| `tokens_in` / `tokens_out` | int | |
| `tool_calls` | int | |
| `file_reads` | int | |
| `runtime_ms` | int | |
| `quality_signal` | text | `poor` \| `ok` \| `good` — derived only |
| `validator_passed` | bool | observable |
| `designer_approved` | bool | observable |
| `runtime_pass` | bool | witness green |
| `reopened` | bool | queue regression |
| `rework_count` | int | |
| `bugs_found` / `bugs_created` | int | |
| `cost_usd` | float | |
| `timestamp` | ts | |

Schema: $ref:src/dev/ops_sql_schema_v1.md · JSON: `agent_run_event_v1`.

---

## Tier 1 — second-order ratios

| Sym | Formula | Use |
|:---|:---|:---|
| `Q/T` | quality / tokens | efficiency |
| `Q/$` | quality / cost_usd | budget |
| `B/KT` | bugs × 1000 / tokens | defect density |
| `FTR` | first-time-right % | slice closed without retry |
| `RTR` | retry rate | parent_run_id chain |
| `TTF` | time to fix | bug open → witness green |
| `DR` | decision reversals | queue row reopened |
| `CI` | complexity index | files touched × systems |
| `KE` | useful_outputs / files_read | knowledge efficiency |
| `ARA` | files_read / files_needed | read amplification — $ref:src/dev/ops_truth_memory_split_v1.md |
| `BIP` | bug introduction rate | bugs_introduced / runs |
| `BDP` | bug discovery rate | bugs_found / runs |
| `VR` | validator fail % | 1 − validator_pass rate |
| `SGP` | ship gate pass % | index ship_allowed |

**OPS report keys:** `ops_report_latest.json` → `metrics_tier1.{q_per_token, ke, …}`

---

## Tier 2 — third-order (marginal / acceleration)

| Sym | Meaning | Compute |
|:---|:---|:---|
| `dQ/dT` | quality gain per token | finite diff on paired runs |
| `d²Q/dT²` | quality acceleration | diminishing returns detector |
| `dC/dQ` | cost per quality gain | **bad trade** if > threshold |
| `dR/dQ` | risk reduction per Q | authority violations ↓ |

### Review-loop policy (draft L1960–L1968)

Plot Q vs tokens; find knee in curve.

| Signal | Action |
|:---|:---|
| `d²Q/dT² < 0` and `dQ/dT < ε` | **Stop** — e.g. max 2 review loops |
| `dQ/dT` still high at loop 3 | Allow loop 4 |

Default ε: **0.001 Q per 1k tokens** (tune on pilot).

---

## Tier 1b — stability

| Sym | Meaning |
|:---|:---|
| `CHL` | correction half-life days per failure class |
| `RIP` | reintroduction rate after fix |

---

## BLANG symbols

```text
Q/T  Q/$  KE  FTR  RTR  CHL  dQ/dT  dC/dQ  quality_signal
```

Emit in `ops_report_latest.json` — not prose in HANDOFF.

---

## Changelog

| Ver | Date | Note |
|:---|:---|:---|
| v1.0.0 | 2026-06-08 | Tier 0–2 from draft L1864+ |
| v1.1.0 | 2026-06-08 | L2104+ Goodhart guard, KE, CHL, quality_signal |
