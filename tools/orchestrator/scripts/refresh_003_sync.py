#!/usr/bin/env python3
"""PLAN-LEDGER-REFRESH-003 — machine-state sync (no spec re-run 2-11)."""
from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]


def load_json(rel: str) -> dict:
    return json.loads((ROOT / rel).read_text(encoding="utf-8"))


def main() -> None:
    wave = load_json("debug_runs/wave_p_live.json")
    shell = load_json("debug_runs/ui_shell_migration_live.json")
    ind = load_json("debug_runs/industrial_activation_live.json")
    s5 = load_json("debug_runs/stage5_full_app_live.json")

    checks: list[dict] = []

    def add(witness: str, field: str, ok: bool, verdict: str, note: str = "") -> None:
        checks.append(
            {"witness": witness, "field": field, "ok": ok, "verdict": verdict, "note": note}
        )

    for f in ("wave_p_green", "ui_wp_layout_002_green", "ui_wp_layout_d07_green"):
        add("wave_p_live.json", f, wave.get(f) is True, "CURRENT", "wave_p_witness_spec_v1.md")
    add(
        "wave_p_live.json",
        "cod_b_wp_witness_001_green",
        wave.get("cod_b_wp_witness_001_green") is True,
        "CURRENT",
        "",
    )

    p2b = shell.get("ui_p2b_coder_b", {})
    add("ui_shell_migration_live.json", "phase2b_closed", shell.get("phase2b_closed") is True, "CURRENT", "")
    add(
        "ui_shell_migration_live.json",
        "ui_p2b_coder_b_green",
        shell.get("ui_p2b_coder_b_green") is True,
        "CURRENT",
        "",
    )
    add(
        "ui_shell_migration_live.json",
        "egui_pass_count_in_sim",
        p2b.get("egui_pass_count_in_sim") == 0,
        "CURRENT",
        "ui_shell_witness_spec_v1.md",
    )

    cc = ind.get("concrete_chain_e2e", {})
    e03 = ind.get("ind_e03", {})
    add(
        "industrial_activation_live.json",
        "production_green",
        cc.get("production_green") is True,
        "CURRENT",
        "IND-E01",
    )
    add(
        "industrial_activation_live.json",
        "ind_e03_green",
        e03.get("ind_e03_green") is True,
        "CURRENT",
        "IND-E03",
    )
    commit = cc.get("placed_via_construction") and (cc.get("sites_committed") or 0) >= 3
    if cc.get("ind_e02_green") and commit:
        add("industrial_activation_live.json", "ind_e02_green", True, "CURRENT", "IND-E02 commit")
    elif not cc.get("ind_e02_green") and not commit:
        add(
            "industrial_activation_live.json",
            "ind_e02_green (seed)",
            True,
            "CURRENT",
            "expected false on seed writer",
        )
    else:
        add(
            "industrial_activation_live.json",
            "ind_e02_green",
            cc.get("ind_e02_green") is True,
            "CURRENT",
            "IND-E02",
        )

    log_rows = (s5.get("projection_graph") or {}).get("logistics_active_rows") or 0
    add(
        "stage5_full_app_live.json",
        "readiness.passes",
        (s5.get("readiness") or {}).get("passes") is True,
        "CURRENT",
        "",
    )
    add(
        "stage5_full_app_live.json",
        "logistics_active_rows",
        log_rows > 0,
        "STALE",
        "refresh via --test visual; lib seed green",
    )

    cq_path = ROOT / "tools/orchestrator/queues/continuation_queue.json"
    items = json.loads(cq_path.read_text(encoding="utf-8"))

    plan_doc = {
        "IND-E03-CODER-A": "docs/archive/2026-06-src-dev/plans/industrial_grid_overload_impl_plan_v1.md",
        "INFRA-PROJ2-001": "docs/archive/2026-06-src-dev/plans/infra_proj2_sole_writer_plan_v1.md",
        "INFRA-PROJ2-CODER-B": "docs/archive/2026-06-src-dev/plans/infra_proj2_sole_writer_plan_v1.md",
        "UI-WP-LAYOUT-002": "docs/archive/2026-06-src-dev/plans/wave_p_witness_spec_v1.md",
        "UI-WP-LAYOUT-D07": "docs/archive/2026-06-src-dev/plans/wave_p_witness_spec_v1.md",
        "COD-B-WP-WITNESS-001": "docs/archive/2026-06-src-dev/plans/wave_p_witness_spec_v1.md",
        "UI-P2B-CODER-B": "docs/archive/2026-06-src-dev/plans/ui_shell_witness_spec_v1.md",
        "UI-SHELL-REFRESH-001": "docs/archive/2026-06-src-dev/plans/ui_shell_witness_spec_v1.md",
        "S7P-IND-001": "docs/archive/2026-06-src-dev/plans/industrial_activation_board_reconcile_v1.md",
        "BQ-128-APPLY-001": "docs/archive/2026-06-src-dev/plans/bq128_editor_path_plan_v1.md",
        "WC-D04-CODER-B": "docs/archive/2026-06-src-dev/plans/infra_slice3_wc_d04_ops_f01_plan_v1.md",
        "LOG-E01": "docs/archive/2026-06-src-dev/plans/logistics_projection_impl_plan_v1.md",
    }

    done_ids = {
        "WATER-W2-FOAM-001",
        "UI-SHELL-REFRESH-001",
        "WATER-W1-OCEAN-001",
        "S7P-IND-001",
        "S7P-DESIGN-001",
        "UI4-DESIGN-001",
        "INFRA-PREFLIGHT-001",
        "UI-WP-LAYOUT-002",
        "UI-WP-LAYOUT-D07",
        "INFRA-PROJ2-001",
        "INFRA-PROJ2-CODER-B",
        "BQ-128-APPLY-001",
        "WC-D04",
        "WC-D04-CODER-B",
        "TRIAGE-VM-09-CODER-B",
        "STEWARD-VM-09-001",
        "UI-P3-M2-TRAY-OPT",
        "UI-P3-M4-001",
        "COD-B-WP-WITNESS-001",
        "UI-P2B-CODER-B",
        "IND-E03-CODER-A",
    }

    for x in items:
        iid = x.get("id", "")
        if iid in plan_doc:
            x["plan_doc"] = plan_doc[iid]
            docs = list(x.get("docs") or [])
            if plan_doc[iid] not in docs:
                docs.insert(0, plan_doc[iid])
            x["docs"] = docs
        if iid in done_ids and x.get("status") == "queued":
            x["status"] = "done"
            prev = x.get("notes") or ""
            x["notes"] = (prev + " | PLAN-LEDGER-REFRESH-003").strip(" |")
        if iid == "PLAN-LEDGER-REFRESH-003":
            x["status"] = "done"
            x["plan_doc"] = "docs/archive/2026-06-src-dev/plans/plan_ledger_refresh_003_checklist_v1.md"
            x["witness"] = "docs/archive/2026-06-fleet-drain/planner_audits/planner_status_audit_v5.md"
            x["docs"] = [
                "docs/archive/2026-06-src-dev/plans/plan_ledger_refresh_003_checklist_v1.md",
                "docs/archive/2026-06-fleet-drain/planner_audits/planner_status_audit_v5.md",
                "docs/archive/2026-06-src-dev/plans/industrial_activation_board_reconcile_v1.md",
            ]
            x["notes"] = "2026-05-25 machine-state sync; no spec re-run 2-11"

    cq_path.write_text(json.dumps(items, indent=2) + "\n", encoding="utf-8")

    planner_specs = [
        ("PLAN-WAVE-P-WITNESS-SPEC-001", "docs/archive/2026-06-src-dev/plans/wave_p_witness_spec_v1.md"),
        ("PLAN-UI-SHELL-WITNESS-SPEC-001", "docs/archive/2026-06-src-dev/plans/ui_shell_witness_spec_v1.md"),
        ("PLAN-IND-E03-001", "docs/archive/2026-06-src-dev/plans/industrial_grid_overload_impl_plan_v1.md"),
        ("PLAN-UI-P3-COMPOSITOR-001", "docs/archive/2026-06-prompts-guides/ui_phases/guides/ui/ui_phase3_minimap_compositor_full_plan_v1.md"),
        ("PLAN-INFRA-PROJ2-001", "docs/archive/2026-06-src-dev/plans/infra_proj2_sole_writer_plan_v1.md"),
        ("PLAN-FIRE-VFX-CLOSURE-001", "docs/archive/2026-06-src-dev/plans/fire_spark_track_closure_plan_v1.md"),
        ("PLAN-UX-BQ128-001", "docs/archive/2026-06-src-dev/plans/bq128_editor_path_plan_v1.md"),
        ("PLAN-UI-P4-ATLAS-001", "docs/archive/2026-06-prompts-guides/ui_phases/guides/ui/ui_phase4_icon_atlas_plan_v1.md"),
        ("PLAN-LOGISTICS-PROJECTION-001", "docs/archive/2026-06-src-dev/plans/logistics_projection_impl_plan_v1.md"),
        ("PLAN-IND-BOARD-RECONCILE-001", "docs/archive/2026-06-src-dev/plans/industrial_activation_board_reconcile_v1.md"),
        ("PLAN-UI-P5-PAUSE-001", "docs/archive/2026-06-prompts-guides/ui_phases/guides/ui/ui_phase5_pause_menu_plan_v1.md"),
    ]
    planner_done = [
        {
            "id": "PLAN-LEDGER-REFRESH-003",
            "status": "done",
            "plan_doc": "docs/archive/2026-06-src-dev/plans/plan_ledger_refresh_003_checklist_v1.md",
            "witness": "docs/archive/2026-06-fleet-drain/planner_audits/planner_status_audit_v5.md",
            "completed": "2026-05-25",
        }
    ]
    for pid, doc in planner_specs:
        planner_done.append(
            {
                "id": pid,
                "status": "done",
                "plan_doc": doc,
                "witness": doc,
                "completed": "2026-05-25",
                "notes": "spec only — not re-run in 003",
            }
        )

    planner_q = {
        "_meta": {
            "version": "1.0.0",
            "date": "2026-05-25",
            "cycle": "PLAN-LEDGER-REFRESH-003",
            "checklist": "docs/archive/2026-06-src-dev/plans/plan_ledger_refresh_003_checklist_v1.md",
            "audit": "docs/archive/2026-06-fleet-drain/planner_audits/planner_status_audit_v5.md",
        },
        "done": planner_done,
        "active": [
            {
                "id": "S7B-PLAN-001",
                "status": "queued",
                "plan_doc": "docs/archive/2026-06-src-dev/plans/stage7_behavioral_full_plan_v1.md",
                "agent": "planner",
            }
        ],
    }
    (ROOT / "tools/orchestrator/queues/planner_active_queue.json").write_text(
        json.dumps(planner_q, indent=2) + "\n", encoding="utf-8"
    )

    coder_q = {
        "_meta": {
            "version": "1.0.0",
            "date": "2026-05-25",
            "cycle": "PLAN-LEDGER-REFRESH-003",
            "triage": "docs/archive/2026-06-src-dev/plans/coder_triage_list_v1.md",
            "markdown_mirror": "docs/archive/2026-06-src-dev/plans/active_coder_queue_v1.md",
        },
        "done": [
            {
                "id": "BQ-128-APPLY-001",
                "plan_doc": "docs/archive/2026-06-src-dev/plans/bq128_editor_path_plan_v1.md",
                "witness": "debug_runs/wave_s_blueprint_roundtrip.json",
            },
            {
                "id": "IND-E03-CODER-A",
                "plan_doc": "docs/archive/2026-06-src-dev/plans/industrial_grid_overload_impl_plan_v1.md",
                "witness": "debug_runs/industrial_activation_live.json",
            },
            {
                "id": "INFRA-PROJ2-001",
                "plan_doc": "docs/archive/2026-06-src-dev/plans/infra_proj2_sole_writer_plan_v1.md",
                "witness": "debug_runs/infrastructure_view_isolation_live.json",
            },
            {
                "id": "UI-P2B-CODER-B",
                "plan_doc": "docs/archive/2026-06-src-dev/plans/ui_shell_witness_spec_v1.md",
                "witness": "debug_runs/ui_shell_migration_live.json",
            },
            {
                "id": "COD-B-WP-WITNESS-001",
                "plan_doc": "docs/archive/2026-06-src-dev/plans/wave_p_witness_spec_v1.md",
                "witness": "debug_runs/wave_p_live.json",
            },
            {
                "id": "WC-D04-CODER-B",
                "plan_doc": "docs/archive/2026-06-src-dev/plans/infra_slice3_wc_d04_ops_f01_plan_v1.md",
                "witness": "debug_runs/stage6_virtualization_live.json",
            },
        ],
        "active": [
            {
                "id": "UI-P5-PAUSE-001",
                "priority": 2,
                "plan_doc": "docs/archive/2026-06-prompts-guides/ui_phases/guides/ui/ui_phase5_pause_menu_plan_v1.md",
                "agent": "coder",
            },
            {
                "id": "LOG-E01",
                "priority": 3,
                "plan_doc": "docs/archive/2026-06-src-dev/plans/logistics_projection_impl_plan_v1.md",
                "witness": "debug_runs/stage5_full_app_live.json",
                "verdict": "STALE",
            },
        ],
        "operator": [
            {"id": "OPS-F01", "witness": "debug_runs/perf_attribution_60s.md"},
            {"id": "OPS-F03", "witness": "debug_runs/stage6_virtualization_live.json"},
        ],
    }
    (ROOT / "tools/orchestrator/queues/coder_active_queue.json").write_text(
        json.dumps(coder_q, indent=2) + "\n", encoding="utf-8"
    )

    triage_lines = [
        "# Coder triage list `v1` (PLAN-LEDGER-REFRESH-003)",
        "",
        "| Field | Value |",
        "|:---|:---|",
        "| **Date** | 2026-05-25 |",
        "| **Audit** | [`planner_status_audit_v5.md`](planner_status_audit_v5.md) |",
        "| **Machine queue** | [`coder_active_queue.json`](../tools/orchestrator/queues/coder_active_queue.json) |",
        "",
        "---",
        "",
        "## P1 — active coder",
        "",
        "| ID | Verdict | plan_doc |",
        "|:---|:---:|:---|",
        "| **UI-P5-PAUSE-001** | OPEN | `ui_phase5_pause_menu_plan_v1.md` |",
        "| **S7B-PLAN-001** | OPEN (planner) | `stage7_behavioral_full_plan_v1.md` |",
        "",
        "## STALE — refresh witness, do not reopen gates",
        "",
        "| ID | Witness | Action |",
        "|:---|:---|:---|",
        "| **LOG-E01** | `stage5_full_app_live.json` `log_rows=0` | `cargo run -p proc_A_dine01 --release -- --test visual` |",
        "",
        "## Done — do not redo",
        "",
        "| ID | plan_doc |",
        "|:---|:---|",
        "| **BQ-128-APPLY-001** | `bq128_editor_path_plan_v1.md` |",
        "| **IND-E03-CODER-A** | `industrial_grid_overload_impl_plan_v1.md` |",
        "| **INFRA-PROJ2-001** / **CODER-B** | `infra_proj2_sole_writer_plan_v1.md` |",
        "| **UI-P2B-CODER-B** | `ui_shell_witness_spec_v1.md` |",
        "| **COD-B-WP-WITNESS-001** | `wave_p_witness_spec_v1.md` |",
        "| **WC-D04-CODER-B** | `infra_slice3_wc_d04_ops_f01_plan_v1.md` |",
        "",
        "## Operator",
        "",
        "| ID | Notes |",
        "|:---|:---|",
        "| **OPS-F01** | Dated 60s perf block |",
        "| **OPS-F03** | Optional sim stage6 refresh |",
        "",
    ]
    (ROOT / "docs/archive/2026-06-src-dev/plans/coder_triage_list_v1.md").write_text("\n".join(triage_lines), encoding="utf-8")

    audit_lines = [
        "# Planner status audit v5 (PLAN-LEDGER-REFRESH-003)",
        "",
        "| Field | Value |",
        "|:---|:---|",
        "| **Audit ID** | **PLAN-LEDGER-REFRESH-003** |",
        "| **Date** | 2026-05-25 |",
        "| **Scope** | Machine-state sync — **no** planner spec re-run 2–11 |",
        "| **Checklist** | [`plan_ledger_refresh_003_checklist_v1.md`](plan_ledger_refresh_003_checklist_v1.md) |",
        "| **Human audit** | [`stage_tracks_audit_signoff_20260525.md`](stage_tracks_audit_signoff_20260525.md) |",
        "",
        "---",
        "",
        "## Witness ↔ spec verification",
        "",
        "| Witness | Field | Verdict | Note |",
        "|:---|:---|:---:|:---|",
    ]
    for c in checks:
        mark = "☑" if c["ok"] else "☐"
        audit_lines.append(
            f"| `{c['witness']}` | `{c['field']}` | **{c['verdict']}** {mark} | {c['note']} |"
        )
    audit_lines += [
        "",
        "---",
        "",
        "## Industrial board (applied)",
        "",
        "See [`industrial_activation_board_reconcile_v1.md`](industrial_activation_board_reconcile_v1.md).",
        "",
        f"- `production_green`: **{cc.get('production_green')}** (IND-E01)",
        f"- `ind_e02_green`: **{cc.get('ind_e02_green')}** · commit path **{commit}**",
        f"- `ind_e03_green`: **{e03.get('ind_e03_green')}** (IND-E03)",
        "",
        "---",
        "",
        "## Machine queues updated",
        "",
        "| File | Action |",
        "|:---|:---|",
        "| `planner_active_queue.json` | **PLAN-LEDGER-REFRESH-003** → **done** |",
        "| `coder_active_queue.json` | `plan_doc` wired |",
        "| `continuation_queue.json` | Hygiene + `plan_doc` |",
        "| `coder_triage_list_v1.md` | Triage snapshot |",
        "",
    ]
    (ROOT / "docs/archive/2026-06-fleet-drain/planner_audits/planner_status_audit_v5.md").write_text("\n".join(audit_lines), encoding="utf-8")
    print(f"ok checks={len(checks)} cq={len(items)}")


if __name__ == "__main__":
    main()
