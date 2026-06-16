# WITNESS-EXEC-SHAPE-001 — exec-shape contract for agent bootstrap `v1`

| Field | Value |
|:---|:---|
| **⟨ID⟩** | **WITNESS-EXEC-SHAPE-001** |
| **Date** | 2026-06-08 |
| **Authority** | $ref:src/dev/debug_run_envelope.rs · $ref:debug_runs/README.md |
| **OPS scan** | `tools/orchestrator/scripts/ops_witness_index.py` |
| **Rule** | **Witness JSON wins.** Markdown exec plans are **spec** only — never substitute for live proof. |

---

## Problem (truth poison)

Agents bootstrap from compressed briefs (`ops_project_brief`, `witness_brief`, `handoff_brief`, `dsm_snapshot`). When these contain:

- hardcoded AUTH spines (`WRK○` when tensor says ★),
- stale ΔWF rows (closed infra tasks),
- truncated `_agent_meta` (first 6 keys only),
- illustrative markdown treated as disk truth,

…implementers re-queue finished work or chase fake blockers.

---

## Exec-shape (live witness JSON)

Every `debug_runs/*_live.json` written by engine or OPS must include:

### Required top-level

| Key | Type | Rule |
|:---|:---|:---|
| `gate` or `program_id` or `profile` | string | Identifies slice |
| `green` or rollup equivalent | bool | Honest — no fake true |
| `_agent_meta` | object | Full envelope — see below |

### Required `_agent_meta` (debug_run_envelope_v1)

| Key | Required |
|:---|:---:|
| `schema` | ✅ `debug_run_envelope_v1` |
| `profile` | ✅ |
| `source_system` | ✅ writer id |
| `relative_path` | ✅ |
| `written_at_epoch_secs` | ✅ |
| `agent_commands` | ✅ when cargo proof exists |
| `related_proofs` | ✅ cross-links |
| `orchestrator` | ✅ paths to reports/queues |
| `docs` | ○ slice-specific refs |

**Forbidden:** hand-authored witness without `_agent_meta` except transitional legacy (must refresh on next writer pass).

### Gate blocks (slice proof)

Nested object `{ "gate": "⟨ID⟩", "green": bool, …keys }` must mirror **PLAN-*-EXEC** witness tables — not shorter illustrative subsets.

---

## OPS rollup shape (ops_report_v2 / ops_project_brief_v1)

| Field | Source of truth | Never hardcode |
|:---|:---|:---|
| `auth_spine` | `master_chain_tensor_v1.json` → `auth_spine` glyphs | old WRK○/ATL○ strings |
| `active_picks` | `post_drain_phase3_queue.json` `status=ready` | stale HANDOFF rows |
| `delta_wf` | registry priorities **minus** witness-resolved | closed INFRA/weather |
| `metrics_tier1` | measured or `"status":"not_measured"` | fake numeric placeholders |
| `quality_score` | derived from `program_summary` scan | static 68 |

**Refresh command:**

```powershell
powershell -File tools/orchestrator/scripts/ops_intelligence_scan.ps1
```

---

## Markdown exec plans (PLAN-*-EXEC)

| Doc type | Role | Agent rule |
|:---|:---|:---|
| `plan_phaseN_exec_001_v1.md` | Witness key **spec** for open slices | Read for **open** rows only |
| `replay_live_ring_impl_plan_v1.md` | **Superseded stub** — use `plan_replay_ring_exec_001_v1.md` | Do not implement from stub |
| `OPS_WITNESS_SPINE.md` § example | **Illustrative** — not a witness | Use live JSON |

When witness on disk is **green**, exec markdown § for that slice is **historical** — close queue row, do not re-dispatch.

---

## Surface boundaries (prevent wrong re-queue)

| Surface | Lane | Re-queue when green? |
|:---|:---|:---:|
| egui Assembly QC | 4 dev tooling | **No** — `aps_bevy_qc_hud_001_v2_live.json` |
| Bevy sim HUD | 5 product chrome | **No** — slice witnesses + `sim_hud_product_close_001` |
| DSM signoff | orchestration | **No** — `dsm_signoff_001_live.json` |
| Replay parity | infra maintain | **No** — `parity_green: true` verify only |

---

## Planner hygiene checklist (PLAN-TRUTH-HYGIENE)

Each drain close:

1. Run OPS scan — refresh `ops_report_latest.json` + `ops_project_brief_v1.json`
2. Compare tensor AUTH vs ops `dsm_snapshot` line 0
3. Close queue rows where witness green
4. Move illustrative stubs to `cancelled_on_disk` in phase queue
5. Update `OPS_LANE_REGISTRY.handoff_priorities` — `resolved: true` or remove
6. Never assign from markdown § alone without `BLANG:Q+` queue row

---

## Changelog

| Ver | Date | Note |
|:---|:---|:---|
| v1.0.0 | 2026-06-08 | Truth poison remediation — tensor AUTH + envelope contract |
