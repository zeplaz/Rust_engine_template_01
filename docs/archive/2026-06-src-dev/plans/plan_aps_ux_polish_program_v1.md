# PLAN-APS-UX-POLISH-PROGRAM-001 — professional polish phases `v1`

| Field | Value |
|:---|:---|
| **Program ID** | **PLAN-APS-UX-POLISH-PROGRAM-001** |
| **Rules** | [`aps_ux_professional_polish_rules_v1.md`](aps_ux_professional_polish_rules_v1.md) |
| **Parent** | [`plan_aps_artist_tool_exec_v1.md`](plan_aps_artist_tool_exec_v1.md) |
| **Status** | **ACTIVE** — Phase 1 ready for `@coder-mcp` |
| **Date** | 2026-06-03 |

**Order:** Phase **1–2** before tokens/density. Frozen UI is the #1 “unprofessional” signal.

---

## Phase map

| Phase | ID | What | Owner | Impact | Exec doc |
|:---:|:---|:---|:---|:---|:---|
| **1** | **APS-UX-ASYNC-001** | Thread MCP calls + job strip (Cancel, tab-switch OK) | @coder-mcp | **Biggest jank fix** | [`plan_aps_ux_async_001_exec_v1.md`](plan_aps_ux_async_001_exec_v1.md) |
| **2** | **APS-UX-NONBLOCK-001** | Replace routine modals with inline + status log | @coder-mcp | Feels professional | §2 in rules + migration table below |
| **3** | **APS-UX-SCROLL-001** | Scroll focus chain; fix catalog list wheel | @coder-mcp | Fixes broken scroll | rules §4 |
| **4** | **APS-UX-DENSITY-001** | Collapse Assembly tags + grammar by default | @coder-mcp + @designer sign | **1280×800** target; **1440×900** max; **960×600** floor | audit issue #4 |
| **5** | **APS-UX-TOKENS-001** | Shared theme: fonts ≥9pt, spacing, dynamic wrap | @coder-mcp | Visual consistency | audit top 5 #2,#7 |
| **6** | **EGUI-DEV-UX-001** | QC load spinner, diagnostics tabs | @coder | Dev tool polish | [`design_aps_bevy_qc_hud_v1.md`](design_aps_bevy_qc_hud_v1.md) |

---

## Phase 2 preview — modal migration (APS-UX-NONBLOCK-001)

**~82 `messagebox` calls** across APS today. Migrate by category:

| Category | Count (approx) | Replace with |
|:---|:---:|:---|
| Success after save/pack/bake | 25+ | Status log + toast + inline success state |
| Validation failed | 15+ | Inline FAIL line (color + text) + log |
| “Select X first” guard | 20+ | Disable button + `disabled` hint text |
| Confirm bake / overwrite | 3 | Inline confirm bar or modal allowlist |
| Error with CLI dump | 10+ | Log full text; inline one-liner |

**Do not migrate in Phase 2:** file overwrite on production path, unsaved snapshot tab away.

---

## Orchestrator paste

```text
@coder-mcp — APS professional polish (rules signed):

READ: docs/archive/2026-06-src-dev/plans/aps_ux_professional_polish_rules_v1.md
EXEC: docs/archive/2026-06-src-dev/plans/plan_aps_ux_async_001_exec_v1.md  (Phase 1 — ship first)

Slice APS-UX-ASYNC-001:
  JobController + job strip + thread tile-batch/pack/bake/generate/preview
  Witness: debug_runs/aps_ux_async_001_live.json
  Test: pytest tools/mcp/python/tests/test_aps_ux_async_001.py

Then APS-UX-NONBLOCK-001 (modal migration table in plan_aps_ux_polish_program_v1.md).

Do NOT start APS-UX-TOKENS-001 until Phase 1 green — tokens won't fix freeze.
```

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-06-03 | Six-phase program; Phase 1 exec brief linked |
