"""ParamFrame → QC dict for studio_dash HTML (Studio DCC / IO-09 · ECS-15 · PCG-06)."""

from __future__ import annotations

from typing import Any


def frame_qc_view(data: dict[str, Any]) -> dict[str, Any]:
    """Shape ParamFrame JSON into a render_report payload (no schema validate here)."""
    chain = list(data.get("grammar_chain") or [])
    modifiers = list(data.get("modifiers") or [])
    graphs = list(data.get("connection_graph_ids") or [])
    intent = data.get("massing_intent") if isinstance(data.get("massing_intent"), dict) else {}
    escalate = data.get("escalate") if isinstance(data.get("escalate"), dict) else {}

    enabled_mods = sum(1 for m in modifiers if isinstance(m, dict) and m.get("enabled", True))

    return {
        **data,
        "title": data.get("title") or f"param_frame QC · {data.get('frame_id', '')}",
        "status": "presentation_only",
        "report_kind": "frame_qc",
        "frame_id": data.get("frame_id"),
        "seed": data.get("seed"),
        "presentation_only": data.get("presentation_only", True),
        "sim_occupation": data.get("sim_occupation", False),
        "grammar_chain": chain,
        "modifiers": modifiers,
        "connection_graph_ids": graphs,
        "massing_intent": intent,
        "escalate": escalate,
        "metrics": {
            "grammar_steps": len(chain),
            "modifiers": len(modifiers),
            "modifiers_enabled": enabled_mods,
            "connection_graphs": len(graphs),
            "has_escalate_massing": 1 if escalate.get("massing_preview_path") else 0,
            "has_escalate_assembly": 1 if escalate.get("assembly_snapshot_path") else 0,
        },
    }