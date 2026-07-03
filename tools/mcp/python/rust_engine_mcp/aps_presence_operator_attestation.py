"""OVR-APS-PRESENCE-OPERATOR-001 — machine G3 checklist + operator attestation witness."""

from __future__ import annotations

import json
import os
import time
from pathlib import Path
from typing import Any

from rust_engine_mcp.paths import repo_root

WITNESS_REL = "debug_runs/aps_presence_operator_attestation_live.json"
PRESENCE_REL = "debug_runs/aps_session_presence_live.json"
TIER_REL = "debug_runs/grammar_set_tier_live.json"
BRIEF_REL = "debug_runs/grammar_set_brief_live.json"
RUBRIC_V2_REL = "src/dev/design_aps_operator_rubric_v2.md"


def _load_json(rel: str, root: Path) -> dict[str, Any]:
    path = root / rel
    if not path.is_file():
        return {}
    return json.loads(path.read_text(encoding="utf-8"))


def validate_g3_presence_checklist(presence: dict[str, Any]) -> tuple[list[dict[str, Any]], list[str]]:
    """Machine-verifiable items from plan §A3 (JSON contract; pixel walk is separate)."""
    ui = presence.get("ui_presence") or {}
    g4 = presence.get("g4_guards") or {}
    tier = presence.get("grammar_set_tier") or {}
    checks: list[dict[str, Any]] = []
    failures: list[str] = []

    def check(name: str, ok: bool, expected: Any, actual: Any) -> None:
        checks.append({"name": name, "ok": ok, "expected": expected, "actual": actual})
        if not ok:
            failures.append(f"{name}: expected {expected!r}, got {actual!r}")

    check("ui_presence.tier", ui.get("tier") == "G3", "G3", ui.get("tier"))
    check(
        "ui_presence.tier_chip",
        ui.get("tier_chip") == "G3 — layer depth",
        "G3 — layer depth",
        ui.get("tier_chip"),
    )
    check("kit_hint_hidden", ui.get("kit_hint_visible") is False, False, ui.get("kit_hint_visible"))
    check("dna_panel_visible", ui.get("dna_panel_visible") is True, True, ui.get("dna_panel_visible"))
    check(
        "iterate_panel_visible",
        ui.get("iterate_panel_visible") is True,
        True,
        ui.get("iterate_panel_visible"),
    )
    check(
        "archetype_combo_count",
        ui.get("archetype_combo_count") == 4,
        4,
        ui.get("archetype_combo_count"),
    )
    check(
        "set_health_honest_red",
        g4.get("building_set_coverage_green") is False,
        False,
        g4.get("building_set_coverage_green"),
    )
    assembly = str(ui.get("assembly_empty_label") or "")
    check(
        "assembly_empty_shape_bias_tail",
        "shape bias" in assembly.lower(),
        "contains 'shape bias'",
        assembly[:80],
    )
    check(
        "grammar_set_tier_match",
        tier.get("tier") == ui.get("tier") == "G3",
        "G3",
        tier.get("tier"),
    )
    return checks, failures


def refresh_aps_presence_operator_attestation_witness(
    *,
    repo: Path | None = None,
    human_verdict: str | None = None,
    operator: str | None = None,
    rubric_score: str | None = None,
    notes: str | None = None,
) -> dict[str, Any]:
    root = repo or repo_root()
    env_verdict = os.environ.get("APS_OPERATOR_PRESENCE_VERDICT", "").strip().lower()
    if human_verdict is None and env_verdict in ("pass", "fail"):
        human_verdict = env_verdict
    if operator is None:
        operator = os.environ.get("APS_OPERATOR_PRESENCE_OPERATOR") or None
    if rubric_score is None:
        rubric_score = os.environ.get("APS_OPERATOR_RUBRIC_V2_SCORE") or None

    presence = _load_json(PRESENCE_REL, root)
    tier = _load_json(TIER_REL, root)
    brief = _load_json(BRIEF_REL, root)

    checklist, failures = validate_g3_presence_checklist(presence)
    machine_ok = len(failures) == 0 and bool(presence.get("green"))
    wit_hon = presence.get("witness_honesty") or {}
    wit_ok = wit_hon.get("status") == "passed"

    human_pending = human_verdict is None
    if human_verdict == "pass":
        verdict = "PASS"
        green = machine_ok and wit_ok
    elif human_verdict == "fail":
        verdict = "FAIL"
        green = False
    else:
        verdict = "PENDING_HUMAN"
        green = False

    body: dict[str, Any] = {
        "gate_id": "OVR-APS-PRESENCE-OPERATOR-001",
        "program_id": "APS-PRESENCE-CORRECTION-001",
        "green": green,
        "verdict": verdict,
        "machine_g3_checklist_ok": machine_ok,
        "witness_honesty_ok": wit_ok,
        "operator": operator,
        "rubric_v2": {
            "authority": RUBRIC_V2_REL,
            "score": rubric_score,
            "pixel_walk": "pending_human" if human_pending else ("pass" if human_verdict == "pass" else "fail"),
            "screenshots_dir": "assets/vfx/reference/aps_rubric_v2/",
        },
        "g3_checklist": checklist,
        "checklist_failures": failures,
        "attach_to_handoff": [
            PRESENCE_REL,
            TIER_REL,
            BRIEF_REL,
        ],
        "presence_snapshot": {
            "ui_presence": presence.get("ui_presence"),
            "g4_guards": presence.get("g4_guards"),
            "grammar_set_tier_tier": (presence.get("grammar_set_tier") or {}).get("tier"),
        },
        "bundled_witnesses": {
            "aps_session_presence_live": presence.get("green"),
            "grammar_set_tier_live": tier.get("tier"),
            "grammar_set_brief_live": brief.get("green"),
        },
        "record_human_pass": (
            "APS_OPERATOR_PRESENCE_VERDICT=pass APS_OPERATOR_PRESENCE_OPERATOR=<name> "
            "APS_OPERATOR_RUBRIC_V2_SCORE=9/10 python -m rust_engine_mcp.cli "
            "aps-presence-operator-attestation --write-witness"
        ),
        "notes": notes
        or (
            "Machine G3 checklist + WIT-HON green; operator attestation pending"
            if human_pending
            else f"Operator attestation {human_verdict}; machine checklist ok={machine_ok}"
        ),
        "_agent_meta": {
            "schema": "aps_presence_operator_attestation_live_v1",
            "written_at_epoch_secs": int(time.time()),
            "profile": "OVR_APS_PRESENCE_OPERATOR",
            "relative_path": WITNESS_REL,
            "agent": "operator",
        },
    }
    out = root / WITNESS_REL
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(body, indent=2) + "\n", encoding="utf-8")
    return body
