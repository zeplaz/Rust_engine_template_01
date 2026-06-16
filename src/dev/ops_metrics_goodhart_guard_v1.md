# OPS metrics — Goodhart guard `v1`

| Field | Value |
|:---|:---|
| **⟨ID⟩** | **OPS-METRICS-GOODHART-001** |
| **Source** | $ref:docs/reference/outside/effwecny_mpc_draft.md (L2105–L2447) |
| **Parent** | $ref:src/dev/plan_ops_metalogic_split_v1.md |

**Thesis:** The danger is not too few metrics — it is **fake, self-referential, Goodharted** metrics agents optimize while real quality stagnates.

---

## Critical rule

**Never store opinions. Store observable pipeline events.**

| ❌ Store | ✅ Store |
|:---|:---|
| Agent says quality = 9/10 | `validator_passed: true` |
| Reviewer score 96 (unanchored) | `designer_approved: false` |
| `intelligence_score` | `rework_count: 4` |
| `reasoning_depth` | `reopened: true` |
| Self-reported satisfaction | `build_succeeded`, `test_passed`, `revert_after_n_days` |

Every metric must trace to an **observable event**:

```text
Validator Passed · Designer Approved · Bug Fixed · Build Worked ·
Runtime Matched · User Kept Change
```

If it cannot trace → **noise candidate** — do not ingest.

---

## Denylist (do not add columns)

| Metric | Why |
|:---|:---|
| Agent confidence | self-referential |
| Agent satisfaction | optimizable theater |
| Quality score (self-reported) | Goodhart |
| Reasoning depth | token padding |
| Prompt complexity | not outcome |
| Thinking tokens | cost, not value |
| `fn_novelty_score` | defer — low traceability |

---

## Allowlist (reality-anchored)

| Metric | Event source |
|:---|:---|
| Build success % | `cargo test` / orchestrate exit |
| Test pass % | CI / lib harness |
| Review pass % | witness `green` |
| Revert rate | git revert / queue reopen |
| Bug introduction rate | `bugs_introduced` / run |
| Bug discovery rate | `bugs_found` |
| Time to resolution | witness green − bug open ts |
| Designer approval % | designer sign-off registry |
| Validator fail % | `validate_p0_gate_plain` |
| Ship gate pass % | `ship_allowed` in index row |

---

## `quality_signal` (derived — not LLM score)

**Do not build `LLM Quality Score` yet.** Derive:

```text
quality_signal =
  validator_pass
+ designer_pass
+ runtime_pass
+ no_revert_after_N_days
```

| Component | Repo witness |
|:---|:---|
| `validator_pass` | P0 gate / MCP validators |
| `designer_pass` | `designer_signoff_registry.json` |
| `runtime_pass` | `*_live.json` `green: true` |
| `no_revert_after_N_days` | queue row stays `done` |

Map to scalar `Q` 0–100 for $ref:src/dev/ops_utility_function_v1.md — **never** from agent prose.

### Example (warehouse run)

```json
{
  "validator_passed": true,
  "designer_approved": false,
  "reopened": true,
  "rework_count": 4,
  "quality_signal": "poor"
}
```

No reviewer opinion field required.

---

## Knowledge Efficiency (KE)

```text
KE = useful_outputs / files_read
```

| Agent | files_read | useful_outputs | KE |
|:---|:---:|:---:|:---:|
| A | 100 | 1 bug fixed | 0.01 |
| B | 8 | 3 bugs fixed | 0.375 |

**Useful output** = witness green · bug fixed · ship gate pass · merged PR — not lines written.

Emit: `metrics_tier1.knowledge_efficiency` in OPS report.

---

## Agent Read Amplification (ARA)

```text
ARA = files_read / files_needed
```

| Agent | files_read | files_needed | ARA |
|:---|:---:|:---:|:---:|
| A | 120 | 4 | **30** — bad |
| B | 8 | 4 | **2** — good |

`files_needed` = `$ref:` list from `ops_claim_task` / exec doc — not agent-declared.

Emit: `metrics_tier1.ara` · pair with KE.

**Source:** $ref:src/dev/ops_truth_memory_split_v1.md§ARA

---

## Correction half-life

```text
Bug Introduced → Bug Fixed → Bug Reintroduced? → time until stable
```

Track per `failure_class` + `file_prefix` — reveals repeated mistakes.

SQL (gate): `failures.correction_half_life_days` materialized view.

JSON (now): append to `ops_report_latest.json` → `failure_stability[]`.

---

## Traceability checklist (ingest review)

Before adding a column or KPI:

| # | Question |
|:---:|:---|
| 1 | What **event** produced this value? |
| 2 | Can an agent game it without improving outcomes? |
| 3 | Is there a witness JSON path today? |
| 4 | Does it duplicate an existing allowlist metric? |

If any fail → reject or defer.

---

## Changelog

| Ver | Date | Note |
|:---|:---|:---|
| v1.0.0 | 2026-06-08 | Goodhart guard from draft L2105+ |
