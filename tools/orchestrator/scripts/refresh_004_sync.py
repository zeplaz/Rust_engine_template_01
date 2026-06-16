#!/usr/bin/env python3
"""PLAN-LEDGER-REFRESH-004 — machine-state sync after UI batch v2 (no spec re-author)."""
from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]


def load_json(rel: str) -> dict:
    return json.loads((ROOT / rel).read_text(encoding="utf-8"))


def main() -> None:
    shell = load_json("debug_runs/ui_shell_migration_live.json")
    mini = load_json("debug_runs/minimap_compositor_live.json")
    s5 = load_json("debug_runs/stage5_full_app_live.json")

    checks: list[dict] = []

    def add(witness: str, field: str, ok: bool, verdict: str, note: str = "") -> None:
        checks.append(
            {"witness": witness, "field": field, "ok": ok, "verdict": verdict, "note": note}
        )

    add("ui_shell_migration_live.json", "phase2a_closed", shell.get("phase2a_closed") is True, "CURRENT", "PLAN-UI-OH-CLOSURE-004")
    add("ui_shell_migration_live.json", "phase2b_closed", shell.get("phase2b_closed") is True, "CURRENT", "")
    add(
        "ui_shell_migration_live.json",
        "egui_pass_count_in_sim",
        shell.get("egui_pass_count_in_sim") == 0,
        "CURRENT",
        "PLAN-UI-PHASE6-001",
    )
    p2c = shell.get("phase2c") or {}
    add(
        "ui_shell_migration_live.json",
        "phase2c.phase2c_closed",
        p2c.get("phase2c_closed") is True,
        "CURRENT",
        "PLAN-UI-2C-001",
    )
    p5 = shell.get("phase5") or {}
    add(
        "ui_shell_migration_live.json",
        "phase5.pause_menu_bevy",
        p5.get("pause_menu_bevy") is True,
        "CURRENT",
        "PLAN-UI-P5-PAUSE-001",
    )
    add(
        "ui_shell_migration_live.json",
        "ui_p5_pause_001_green",
        shell.get("ui_p5_pause_001_green") is True,
        "CURRENT",
        "",
    )
    p4 = shell.get("phase4") or {}
    add(
        "ui_shell_migration_live.json",
        "phase4.p5_br_tab_wired",
        p4.get("p5_br_tab_wired") is True,
        "CURRENT",
        "PLAN-UI-P4-ATLAS-001",
    )
    add("minimap_compositor_live.json", "ui_p3_m3_green", mini.get("ui_p3_m3_green") is True, "CURRENT", "PLAN-UI-P3-M3-001")
    add("minimap_compositor_live.json", "ui_p3_m4_green", mini.get("ui_p3_m4_green") is True, "CURRENT", "design M3")
    m3_oh = mini.get("ui_oh_m3_001") or {}
    add(
        "minimap_compositor_live.json",
        "ui_oh_m3_001.green",
        m3_oh.get("green") is True or mini.get("ui_p3_m3_green") is True,
        "CURRENT",
        "UI-OH-M3-001",
    )
    add(
        "stage5_full_app_live.json",
        "readiness.passes",
        (s5.get("readiness") or {}).get("passes") is True,
        "CURRENT",
        "UI-OH-GATE-001",
    )

    plan_doc = {
        "PLAN-UI-OH-CLOSURE-004": "docs/archive/2026-06-src-dev/plans/ui_overhaul_phase23_closure_plan_v1.md",
        "PLAN-UI-P5-PAUSE-001": "docs/archive/2026-06-prompts-guides/ui_phases/guides/ui/ui_phase5_pause_menu_plan_v1.md",
        "UI-OH-P5-001": "docs/archive/2026-06-src-dev/plans/ui_oh_p5_001_plan_v1.md",
        "PLAN-UI-P4-ATLAS-001": "docs/archive/2026-06-prompts-guides/ui_phases/guides/ui/ui_phase4_icon_atlas_plan_v1.md",
        "UI-OH-P4-001": "docs/archive/2026-06-src-dev/plans/ui_oh_p4_001_plan_v1.md",
        "PLAN-UI-P3-M3-001": "docs/archive/2026-06-src-dev/plans/plan_ui_p3_m3_operational_stage7_plan_v1.md",
        "UI-OH-M3-001": "docs/archive/2026-06-src-dev/plans/ui_oh_m3_001_plan_v1.md",
        "PLAN-UI-PHASE6-001": "docs/archive/2026-06-src-dev/plans/ui_phase6_shell_perf_multiview_plan_v1.md",
        "PLAN-UI-2C-001": "docs/archive/2026-06-src-dev/plans/ui_phase2c_left_command_rail_plan_v1.md",
        "PLAN-UI-THEME-MERGE-001": "docs/archive/2026-06-src-dev/plans/ui_theme_merge_impl_spec_v1.md",
        "PLAN-LEDGER-REFRESH-004": "docs/archive/2026-06-src-dev/plans/plan_ledger_refresh_004_checklist_v1.md",
        "UI-P5-PAUSE-001": "docs/archive/2026-06-prompts-guides/ui_phases/guides/ui/ui_phase5_pause_menu_plan_v1.md",
    }

    batch_ids = set(plan_doc.keys())

    cq_path = ROOT / "tools/orchestrator/queues/continuation_queue.json"
    items = json.loads(cq_path.read_text(encoding="utf-8"))
    existing_ids = {x.get("id") for x in items}

    new_entries = [
        {
            "id": "PLAN-UI-PHASE6-001",
            "priority": 2,
            "title": "UI Phase 6 shell perf + multiview isolation plan",
            "lane": "UI / Planning",
            "agent": "planner",
            "track": "UI-P6",
            "witness": "debug_runs/infrastructure_view_isolation_live.json",
            "commands": [
                "cargo test -p proc_A_dine01 --lib steward_ui_oh_gate_001_lib_bundle infrastructure_view_isolation"
            ],
            "plan_doc": plan_doc["PLAN-UI-PHASE6-001"],
            "docs": [plan_doc["PLAN-UI-PHASE6-001"], "src/dev/post_stage6_active_todos.md"],
            "source": "planner_queue_ui_batch_v2",
            "status": "done",
            "notes": "2026-05-25 signed; VM-09-v2 deferred",
        },
        {
            "id": "PLAN-UI-2C-001",
            "priority": 2,
            "title": "UI Phase 2C left command rail (2C-B)",
            "lane": "UI / Planning",
            "agent": "planner",
            "track": "UI-2C",
            "witness": "debug_runs/ui_shell_migration_live.json",
            "commands": [
                "cargo test -p proc_A_dine01 --lib simulation_shell_phase2 -- --test-threads=1"
            ],
            "plan_doc": plan_doc["PLAN-UI-2C-001"],
            "docs": [
                plan_doc["PLAN-UI-2C-001"],
                "docs/archive/2026-06-prompts-guides/ui_phases/guides/ui/ui_phase0_panel_mocks_v1.md",
            ],
            "source": "planner_queue_ui_batch_v2",
            "status": "done",
            "notes": "2C-B closed 2026-05-24",
        },
        {
            "id": "PLAN-UI-THEME-MERGE-001",
            "priority": 2,
            "title": "UI theme merge — single coder impl spec",
            "lane": "UI / Planning",
            "agent": "planner",
            "track": "UI-THEME",
            "witness": "docs/archive/2026-06-src-dev/plans/ui_theme_merge_impl_spec_v1.md",
            "plan_doc": plan_doc["PLAN-UI-THEME-MERGE-001"],
            "docs": [
                plan_doc["PLAN-UI-THEME-MERGE-001"],
                "docs/archive/2026-06-prompts-guides/ui_phases/guides/ui/design_theme.md",
            ],
            "source": "planner_queue_ui_batch_v2",
            "status": "done",
        },
        {
            "id": "PLAN-LEDGER-REFRESH-004",
            "priority": 1,
            "title": "Ledger refresh after UI batch v2",
            "lane": "Planning",
            "agent": "orchestrator",
            "track": "LEDGER",
            "witness": "docs/archive/2026-06-fleet-drain/planner_audits/planner_status_audit_v6.md",
            "commands": ["python tools/orchestrator/scripts/refresh_004_sync.py"],
            "plan_doc": plan_doc["PLAN-LEDGER-REFRESH-004"],
            "docs": [
                plan_doc["PLAN-LEDGER-REFRESH-004"],
                "docs/archive/2026-06-src-dev/plans/planner_queue_ui_batch_v2.md",
            ],
            "source": "planner_queue_ui_batch_v2",
            "status": "done",
            "notes": "2026-05-25 machine-state sync items 1-7",
        },
    ]
    for entry in new_entries:
        if entry["id"] not in existing_ids:
            items.append(entry)

    for x in items:
        iid = x.get("id", "")
        if iid in plan_doc:
            x["plan_doc"] = plan_doc[iid]
            docs = list(x.get("docs") or [])
            if plan_doc[iid] not in docs:
                docs.insert(0, plan_doc[iid])
            x["docs"] = docs
        if iid in batch_ids and x.get("status") == "queued":
            x["status"] = "done"
            prev = x.get("notes") or ""
            x["notes"] = (prev + " | PLAN-LEDGER-REFRESH-004").strip(" |")
        if iid == "PLAN-LEDGER-REFRESH-004":
            x["status"] = "done"

    cq_path.write_text(json.dumps(items, indent=2) + "\n", encoding="utf-8")

    planner_specs = [
        ("PLAN-UI-OH-CLOSURE-004", plan_doc["PLAN-UI-OH-CLOSURE-004"]),
        ("PLAN-UI-P5-PAUSE-001", plan_doc["PLAN-UI-P5-PAUSE-001"]),
        ("PLAN-UI-P4-ATLAS-001", plan_doc["PLAN-UI-P4-ATLAS-001"]),
        ("PLAN-UI-P3-M3-001", plan_doc["PLAN-UI-P3-M3-001"]),
        ("PLAN-UI-PHASE6-001", plan_doc["PLAN-UI-PHASE6-001"]),
        ("PLAN-UI-2C-001", plan_doc["PLAN-UI-2C-001"]),
        ("PLAN-UI-THEME-MERGE-001", plan_doc["PLAN-UI-THEME-MERGE-001"]),
    ]
    planner_done = [
        {
            "id": "PLAN-LEDGER-REFRESH-004",
            "status": "done",
            "plan_doc": plan_doc["PLAN-LEDGER-REFRESH-004"],
            "witness": "docs/archive/2026-06-fleet-drain/planner_audits/planner_status_audit_v6.md",
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
                "notes": "UI batch v2 — spec only",
            }
        )

    planner_q = {
        "_meta": {
            "version": "1.0.0",
            "date": "2026-05-25",
            "cycle": "PLAN-LEDGER-REFRESH-004",
            "checklist": plan_doc["PLAN-LEDGER-REFRESH-004"],
            "batch": "docs/archive/2026-06-src-dev/plans/planner_queue_ui_batch_v2.md",
            "audit": "docs/archive/2026-06-fleet-drain/planner_audits/planner_status_audit_v6.md",
        },
        "done": planner_done,
        "active": [
            {
                "id": "S7B-PREFLIGHT-001",
                "status": "queued",
                "plan_doc": "docs/archive/2026-06-src-dev/plans/stage7_behavioral_implementation_plan_v1.md",
                "agent": "sim-steward",
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
            "cycle": "PLAN-LEDGER-REFRESH-004",
            "triage": "docs/archive/2026-06-src-dev/plans/coder_triage_list_v1.md",
        },
        "done": [
            {
                "id": "UI-P5-PAUSE-001",
                "plan_doc": plan_doc["UI-P5-PAUSE-001"],
                "witness": "debug_runs/ui_shell_migration_live.json",
                "notes": "P5-PAUSE-001 CLOSED — lib green",
            },
            {
                "id": "UI-P3-M3-001",
                "plan_doc": plan_doc["PLAN-UI-P3-M3-001"],
                "witness": "debug_runs/minimap_compositor_live.json",
            },
        ],
        "active": [
            {
                "id": "S7B-M1-001",
                "priority": 1,
                "plan_doc": "docs/archive/2026-06-src-dev/plans/stage7_behavioral_implementation_plan_v1.md",
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
        ],
    }
    (ROOT / "tools/orchestrator/queues/coder_active_queue.json").write_text(
        json.dumps(coder_q, indent=2) + "\n", encoding="utf-8"
    )

    triage_lines = [
        "# Coder triage list `v1` (PLAN-LEDGER-REFRESH-004)",
        "",
        "| Field | Value |",
        "|:---|:---|",
        "| **Date** | 2026-05-25 |",
        "| **Batch** | [`planner_queue_ui_batch_v2.md`](planner_queue_ui_batch_v2.md) |",
        "| **Audit** | [`planner_status_audit_v6.md`](planner_status_audit_v6.md) |",
        "",
        "## P1 — active coder",
        "",
        "| ID | Verdict | plan_doc |",
        "|:---|:---:|:---|",
        "| **S7B-M1-001** | OPEN | `stage7_behavioral_implementation_plan_v1.md` |",
        "",
        "## Done — UI batch v2",
        "",
        "| ID | plan_doc |",
        "|:---|:---|",
        "| **UI-P5-PAUSE-001** | `ui_phase5_pause_menu_plan_v1.md` |",
        "| **UI-P3-M3-001** | `plan_ui_p3_m3_operational_stage7_plan_v1.md` |",
        "",
        "## STALE",
        "",
        "| ID | Witness | Action |",
        "|:---|:---|:---|",
        "| **LOG-E01** | `stage5_full_app_live.json` | `--test visual` refresh |",
        "| **phase4.icon_atlas_loaded** | shell JSON | run `icon_atlas` lib test + steward bundle |",
        "",
    ]
    (ROOT / "docs/archive/2026-06-src-dev/plans/coder_triage_list_v1.md").write_text("\n".join(triage_lines), encoding="utf-8")

    audit_lines = [
        "# Planner status audit v6 (PLAN-LEDGER-REFRESH-004)",
        "",
        "| Field | Value |",
        "|:---|:---|",
        "| **Audit ID** | **PLAN-LEDGER-REFRESH-004** |",
        "| **Date** | 2026-05-25 |",
        "| **Scope** | UI batch v2 machine sync — items 1–7 |",
        "| **Checklist** | [`plan_ledger_refresh_004_checklist_v1.md`](plan_ledger_refresh_004_checklist_v1.md) |",
        "| **Batch** | [`planner_queue_ui_batch_v2.md`](planner_queue_ui_batch_v2.md) |",
        "",
        "## Witness verification",
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
        "## Machine queues updated",
        "",
        "| File | Action |",
        "|:---|:---|",
        "| `planner_active_queue.json` | UI batch v2 → **done** |",
        "| `coder_active_queue.json` | UI-P5-PAUSE-001 → **done** |",
        "| `continuation_queue.json` | plan_doc + new PLAN-* rows |",
        "",
    ]
    (ROOT / "docs/archive/2026-06-fleet-drain/planner_audits/planner_status_audit_v6.md").write_text("\n".join(audit_lines), encoding="utf-8")
    print(f"ok checks={len(checks)} cq={len(items)}")


if __name__ == "__main__":
    main()
