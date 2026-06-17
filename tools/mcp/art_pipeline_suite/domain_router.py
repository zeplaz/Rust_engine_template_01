"""Lane config — buildings vs landscape tab sets (Option D IA)."""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any

from rust_engine_mcp.paths import repo_root

from .state import ArtDomain, SuiteState

PREFS_REL = "debug_runs/aps_ui_prefs.json"

FLOW_CAVEAT = "Every button here runs the same tools the build pipeline uses."

BUILDINGS_TAB_LABELS = ("Catalog", "Materials", "Assembly", "Variants", "Atlas")
LANDSCAPE_TAB_LABELS = ("Presets", "Grammar", "States", "Atlas")

AUTHORITY_BY_LANE: dict[str, str] = {
    ArtDomain.BUILDINGS.value: (
        "What ships: the Assembly you save here (its materials + tags). "
        "Catalog data and atlas tiles only feed into it."
    ),
    ArtDomain.LANDSCAPE.value: (
        "What ships: the Landscape preset you select here. "
        "Tiles are baked through the keyframe step only."
    ),
}

PIPELINE_STEPS_BY_LANE: dict[str, tuple[tuple[str, str], ...]] = {
    ArtDomain.BUILDINGS.value: (
        ("catalog", "Catalog"),
        ("materials", "Materials"),
        ("assembly", "Assembly"),
        ("variants", "Variants"),
        ("atlas", "Atlas"),
    ),
    ArtDomain.LANDSCAPE.value: (
        ("presets", "Presets"),
        ("grammar", "Grammar"),
        ("states", "States"),
        ("atlas", "Atlas"),
    ),
}

FLOW_VERBS_BY_LANE: dict[str, tuple[tuple[str, str], ...]] = {
    ArtDomain.BUILDINGS.value: (
        ("send_to_assembly", "Send to Assembly"),
        ("bake_variants", "Bake variants"),
        ("pack_atlas", "Pack atlas"),
    ),
    ArtDomain.LANDSCAPE.value: (
        ("generate_grammar", "Generate grammar"),
        ("bake_states", "Bake states"),
        ("pack_lg5_atlas", "Pack landscape atlas"),
    ),
}

DES_APS_E1_IA_SIGN_ID = "DES-APS-E1-IA-OPTION-D-001"

CATALOG_SOURCE_BY_LANE: dict[str, str] = {
    ArtDomain.BUILDINGS.value: "building_modules",
    ArtDomain.LANDSCAPE.value: "landscape_presets",
}


def normalize_lane(raw: str | None) -> str:
    val = str(raw or ArtDomain.BUILDINGS.value).strip().lower()
    if val in (ArtDomain.LANDSCAPE.value, "landscape"):
        return ArtDomain.LANDSCAPE.value
    return ArtDomain.BUILDINGS.value


def tab_labels_for(lane: str) -> tuple[str, ...]:
    return LANDSCAPE_TAB_LABELS if normalize_lane(lane) == ArtDomain.LANDSCAPE.value else BUILDINGS_TAB_LABELS


def catalog_source_for(lane: str) -> str:
    return CATALOG_SOURCE_BY_LANE[normalize_lane(lane)]


def authority_for(lane: str) -> str:
    return AUTHORITY_BY_LANE[normalize_lane(lane)]


def pipeline_steps_for(lane: str) -> tuple[tuple[str, str], ...]:
    return PIPELINE_STEPS_BY_LANE[normalize_lane(lane)]


def flow_verbs_for(lane: str) -> tuple[tuple[str, str], ...]:
    return FLOW_VERBS_BY_LANE[normalize_lane(lane)]


def tab_count_for(lane: str) -> int:
    return len(tab_labels_for(lane))


def verify_option_d_ia_contract() -> dict[str, Any]:
    """Static contract from design_aps_domain_ia_sign_v1.md — tab sets + lane-scoped chrome."""
    buildings_tabs = tab_labels_for(ArtDomain.BUILDINGS.value)
    landscape_tabs = tab_labels_for(ArtDomain.LANDSCAPE.value)
    buildings_flow = [k for k, _ in flow_verbs_for(ArtDomain.BUILDINGS.value)]
    landscape_flow = [k for k, _ in flow_verbs_for(ArtDomain.LANDSCAPE.value)]
    buildings_pipe = [k for k, _ in pipeline_steps_for(ArtDomain.BUILDINGS.value)]
    landscape_pipe = [k for k, _ in pipeline_steps_for(ArtDomain.LANDSCAPE.value)]
    tab_set_swap = len(buildings_tabs) == 5 and len(landscape_tabs) == 4 and buildings_tabs != landscape_tabs
    flow_lane_scoped = buildings_flow == [
        "send_to_assembly",
        "bake_variants",
        "pack_atlas",
    ] and landscape_flow == ["generate_grammar", "bake_states", "pack_lg5_atlas"]
    pipeline_lane_scoped = buildings_pipe == [
        "catalog",
        "materials",
        "assembly",
        "variants",
        "atlas",
    ] and landscape_pipe == ["presets", "grammar", "states", "atlas"]
    ok = tab_set_swap and flow_lane_scoped and pipeline_lane_scoped
    return {
        "option_d_ia_contract_ok": ok,
        "tab_set_swap": tab_set_swap,
        "buildings_tab_count": len(buildings_tabs),
        "landscape_tab_count": len(landscape_tabs),
        "flow_lane_scoped": flow_lane_scoped,
        "pipeline_lane_scoped": pipeline_lane_scoped,
        "buildings_flow_keys": buildings_flow,
        "landscape_flow_keys": landscape_flow,
        "buildings_pipeline_keys": buildings_pipe,
        "landscape_pipeline_keys": landscape_pipe,
    }


def verify_option_d_shell_implementation() -> dict[str, Any]:
    """Runtime shell: dual notebook page sets — not label-rename on one 5-tab notebook."""
    try:
        from . import app as app_mod
    except ImportError as exc:
        return {"option_d_shell_ok": False, "shell_error": str(exc)}
    shell_ok = bool(getattr(app_mod, "OPTION_D_DUAL_NOTEBOOK", False))
    return {
        "option_d_shell_ok": shell_ok,
        "dual_notebook_flag": shell_ok,
        "reuses_building_panels_in_landscape_lane": not shell_ok,
    }


def des_aps_e1_ia_verdict(*, repo: Path | None = None) -> dict[str, Any]:
    """Static contract from design_aps_domain_ia_sign_v1.md — tab sets + lane-scoped chrome."""
    _ = repo
    contract = verify_option_d_ia_contract()
    shell = verify_option_d_shell_implementation()
    ok = bool(contract.get("option_d_ia_contract_ok")) and bool(shell.get("option_d_shell_ok"))
    reasons: list[str] = []
    if not contract.get("tab_set_swap"):
        reasons.append("tab_set_swap: expected 5 buildings + 4 landscape tab labels")
    if not contract.get("flow_lane_scoped"):
        reasons.append("flow_lane_scoped: flow verbs must differ per lane per sign-off")
    if not contract.get("pipeline_lane_scoped"):
        reasons.append("pipeline_lane_scoped: pipeline STEPS must differ per lane per sign-off")
    if not shell.get("option_d_shell_ok"):
        reasons.append("shell: dual notebook page sets required (not rename on shared Assembly/Variants)")
    return {
        "id": DES_APS_E1_IA_SIGN_ID,
        "verdict": "pass" if ok else "fail",
        "design_ref": "src/dev/design_aps_domain_ia_sign_v1.md",
        "reasons": reasons,
        **contract,
        **shell,
    }


def clear_cross_lane_selection(state: SuiteState, lane: str) -> None:
    """Lane switch does not carry selection across lanes."""
    if normalize_lane(lane) == ArtDomain.LANDSCAPE.value:
        state.selected_module_id = None
        state.selected_module_ids = []
    else:
        state.selected_landscape_preset_id = None


def load_active_lane(*, repo: Path | None = None) -> str:
    root = repo or repo_root()
    path = root / PREFS_REL
    if not path.is_file():
        return ArtDomain.BUILDINGS.value
    try:
        body = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError):
        return ArtDomain.BUILDINGS.value
    return normalize_lane(body.get("active_lane") or body.get("art_domain"))


def save_active_lane(lane: str, *, repo: Path | None = None) -> dict[str, Any]:
    root = repo or repo_root()
    path = root / PREFS_REL
    path.parent.mkdir(parents=True, exist_ok=True)
    norm = normalize_lane(lane)
    body = {"active_lane": norm, "art_domain": norm}
    path.write_text(json.dumps(body, indent=2) + "\n", encoding="utf-8")
    return body
