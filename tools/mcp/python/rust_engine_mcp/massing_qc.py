"""Massing preview → QC dict (CQ-76 sole owner in rust_engine_mcp).

Presentation / voxels in scope. APS panel + studio_dash import from here.
"""

from __future__ import annotations

from typing import Any


def floors_histogram(columns: list[dict[str, Any]]) -> dict[str, int]:
    """Count columns by floors band 1..4 (QC summary)."""
    hist = {"1": 0, "2": 0, "3": 0, "4": 0}
    for col in columns:
        try:
            f = int(col.get("floors", 0))
        except (TypeError, ValueError):
            continue
        key = str(f)
        if key in hist:
            hist[key] += 1
    return hist


def massing_qc_view(data: dict[str, Any]) -> dict[str, Any]:
    """Enrich massing_preview JSON for studio_dash / APS HTML QC."""
    cols = list(data.get("massing_columns") or [])
    hist = floors_histogram(cols)
    chain = data.get("grammar_rule_chain") if isinstance(data.get("grammar_rule_chain"), dict) else {}
    return {
        **data,
        "title": data.get("title") or "massing_preview QC",
        "status": "presentation_only",
        "report_kind": data.get("report_kind") or "massing_qc",
        "column_count": len(cols),
        "seed": data.get("seed"),
        "presentation_only": data.get("presentation_only", True),
        "sim_occupation": data.get("sim_occupation", False),
        "pilot_id": data.get("pilot_id"),
        "arch_build_grammar_preset_id": data.get("arch_build_grammar_preset_id"),
        "material_strategy": data.get("material_strategy"),
        "grammar_rule_chain": chain,
        "param_frame_id": data.get("param_frame_id"),
        "ref_books_dual_cite": data.get("ref_books_dual_cite"),
        "floors_histogram": hist,
        "metrics": {
            "column_count": len(cols),
            "floors_1": hist["1"],
            "floors_2": hist["2"],
            "floors_3": hist["3"],
            "floors_4": hist["4"],
            "has_grammar_chain": 1 if chain else 0,
        },
        "massing_columns": cols,
    }
