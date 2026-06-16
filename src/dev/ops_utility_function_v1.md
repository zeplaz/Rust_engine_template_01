# OPS utility function `v1`

| Field | Value |
|:---|:---|
| **⟨ID⟩** | **OPS-UTILITY-001** |
| **Source** | $ref:docs/reference/outside/effwecny_mpc_draft.md§utility-layer (L1713+) |
| **Parent** | $ref:src/dev/plan_ops_metalogic_split_v1.md |

**Purpose:** Formal economics so OPS does not drown in telemetry. Supervisor answers *which model / prompt / tool / workflow / review chain* maximizes utility.

---

## Utility function

```text
U = Q − λ·Ct − μ·Cm − ν·Dp
```

| Sym | Range | Meaning | Repo source |
|:---|:---|:---|:---|
| `Q` | 0–100 | Quality rollup | **`quality_signal` derived** — $ref:src/dev/ops_metrics_goodhart_guard_v1.md · **not** agent self-score |
| `Ct` | 0–∞ | Token cost (normalized) | `tokens_in` + `tokens_out` per run |
| `Cm` | 0–∞ | Compute cost (normalized) | Blender/Bevy ms, `cargo` duration |
| `Dp` | 0–∞ | Maintenance debt | open triage rows + warning registry count |

### λ defaults (planner-signed — tune on pilot data)

| λ | Default | Rationale |
|:---|:---:|:---|
| `λ` (tokens) | **0.02** | 1 Q-point ≈ worth 50k tokens at median |
| `μ` (compute) | **0.01** | WRK jobs expensive but bounded |
| `ν` (debt) | **0.15** | Authority drift costly — `@debug-intelligence` |

**Reject rule:** propose ΔWF only if `ΔU > 0` OR `ΔQ ≥ 5` with `ΔCt ≤ 10%`.

**Bad trade (🟡):** `ΔQ > 0` and `ΔCt / ΔQ > 0.5` — Jacobian `dC/dQ` too steep.

---

## Supervisor questions (compressed)

```text
max U s.t. Q↑ Ct↓ Cx↓ Au↑
```

| Question | OPS surface |
|:---|:---|
| Which model? | `v_agent_success` / Slutsky `ΔModel` |
| Which prompt? | `prompt_hash` + `∂Q/∂Prompt` |
| Which tool? | `tool_usage` + `∂Q/∂Tool` |
| Which workflow? | HANDOFF lane + `FTR` |
| Which review chain? | `iteration` vs diminishing returns |

---

## Anti-pattern (draft L1734–L1744)

```text
User → LLM → LLM → LLM → LLM   # every step re-explains reality
```

**Replace with:**

```text
User → Supervisor → MCP/BLANG → Compressed State → Agent
```

Compressed state example — $ref:src/dev/ops_agent_compression_v1.md§project-brief-shape.

---

## Changelog

| Ver | Date | Note |
|:---|:---|:---|
| v1.0.0 | 2026-06-08 | U(agent) + λ defaults from draft L1713+ |
| v1.1.0 | 2026-06-08 | Q from quality_signal only — Goodhart guard L2104+ |
