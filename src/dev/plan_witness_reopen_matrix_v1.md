# PLAN-WITNESS-REOPEN-001 — witness reopen matrix `v1`

```text
⟦SYMLANG⟧⟐v1  ◈PLAN
⟨ID⟩ PLAN-WITNESS-REOPEN-001
Date: 2026-06-16
Status: **SIGNED** (@planner-mcp)
Owner: @planner-mcp → Operator (approve) → @coder / @coder-mcp (fix)
Parent: $ref:src/dev/plan_witness_queue_integrity_mcp_v1.md § Bulk reopen
Scan: validate-report witness_honesty --scan debug_runs
```

**Goal:** Triage which **queue-done** rows fail **BLANG:WIT-HON** — separate **reopen** from **expected report-only FAIL**.

**Not in scope:** bulk `status: reopened` script (Phase 6 after WIT-RUST-*).

---

## Scan snapshot (2026-06-16)

| Metric | Value |
|:---|:---|
| Files scanned | 255 |
| Files with issues | 135 |
| Total issues | 271 |
| Errors | 146 |
| Warnings | 125 |

---

## Reopen matrix — phase6 / VEG / G-PLAY (priority)

| Witness | Queue row(s) | Queue status | WIT-HON | Reopen? | Fix lane |
|:---|:---|:---:|:---:|:---:|:---|
| `veg_runtime_proof_live.json` | PLAN-VEG-RUNTIME-PROOF-001 rollup | done (planner) | **FAIL** `WIT-OPERATOR-LIB-FIXTURE` | **Yes** | @coder A — set `proof_grade` or drop `operator_visible` until L4 |
| `landscape_grammar_lg4_preview_live.json` | VEG-HARD-PREVIEW-PIXEL-001 | done | **FAIL** tint/zero / lib fixture | **Yes** | @coder A — CDR-A-LG4-PIXEL-REOPEN-001 |
| `landscape_grammar_sim_harness_live.json` | VEG harness rows | done | **FAIL** operator+lib | **Yes** | @coder A — harness vs live split |
| `g_play_product_close_live.json` | G-PLAY-CODER-VEG | done (coder) | verify on scan | **Review** | CDR-A-WIT-HON-ROLLUP-001 |
| `play_scenario_live.json` | G-PLAY-* | done | partial | **Review** | proof_grade gate |
| `stage5_full_app_live.json` | VEG-HARD-FULLAPP-001 | done | likely green | **No** | maintain |
| `mcp_landscape_grammar_sign_live.json` | MCP-LANDSCAPE-GRAMMAR-SIGN | done | green | **No** | SCHEMA scope only |

---

## Reopen matrix — by failure class

| Class | Count (approx) | Action |
|:---|:---:|:---|
| `WIT-OPERATOR-LIB-FIXTURE` | high | Reopen VEG/G-PLAY coder rows claiming operator-visible on lib harness |
| `WIT-MISSING-ENVELOPE` | high | Warning-only bulk — MCP-WIT envelope pass, not queue reopen |
| `WIT-GREEN-WITHOUT-PROOF` | medium | Reopen if queue row `status: done` |
| `WIT-QUEUE-CONTRADICTION` | low | `queue_integrity` fix first |

---

## Operator gate

```text
REOPEN APPROVED ⇔ Operator signs table § "Reopen? = Yes" rows
  → script sets status: reopened + reopen_reason (post WIT-RUST-004)
```

Until script ships: manual queue edit per `coder_queue_hardening_rules_v1.md`.

---

## Acceptance

| # | Criterion |
|:---:|:---|
| R1 | This matrix on disk |
| R2 | `witness_reopen_candidates_v1.json` emitted (optional refresh script) |
| R3 | No queue row marked `reopened` without Operator column |

---

## Sign-off

| Role | Date | Action |
|:---|:---|:---|
| **@planner-mcp** | 2026-06-16 | **SIGNED** — matrix authoritative for wave-0 |
| **Operator** | 2026-06-02 | **APPROVED** — bulk reopen gate open (Reopen? = Yes rows) |

```text
⟦/PLAN-WITNESS-REOPEN-001⟧  ΔWF→ CDR-A-WIT-HON-ROLLUP-001 · WIT-RUST-* after MCP-WIT green
```
