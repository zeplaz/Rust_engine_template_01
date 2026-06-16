"""APS-MAT-003 — load material_category_tree_v1.json for Materials tab browse."""

from __future__ import annotations

import json
from functools import lru_cache
from pathlib import Path
from typing import Any

from .paths import repo_root

TREE_REL = "assets/materials/profiles/material_category_tree_v1.json"
APS_MAT_003_WITNESS = "debug_runs/aps_mat_003_category_tree_live.json"


def tree_path() -> Path:
    return repo_root() / TREE_REL


@lru_cache(maxsize=1)
def load_material_category_tree() -> dict[str, Any]:
    path = tree_path()
    if not path.is_file():
        return {"schema": "material_category_tree_v1", "roots": [], "infer_rules": [], "profile_bindings": []}
    return json.loads(path.read_text(encoding="utf-8"))


def _binding_map(tree: dict[str, Any]) -> dict[str, str]:
    out: dict[str, str] = {}
    for row in tree.get("profile_bindings") or []:
        if isinstance(row, dict) and row.get("profile_id") and row.get("category_path"):
            out[str(row["profile_id"])] = str(row["category_path"])
    return out


def infer_category_from_tree(profile_id: str, tree: dict[str, Any] | None = None) -> str | None:
    """Resolve category_path via profile_bindings then infer_rules (highest priority first)."""
    if tree is None:
        tree = load_material_category_tree()
    pid = profile_id.strip()
    if not pid:
        return None
    bindings = _binding_map(tree)
    if pid in bindings:
        return bindings[pid]
    pid_l = pid.lower()
    rules = sorted(
        (r for r in (tree.get("infer_rules") or []) if isinstance(r, dict) and r.get("match")),
        key=lambda r: int(r.get("priority") or 0),
        reverse=True,
    )
    for rule in rules:
        match = str(rule.get("match") or "").lower()
        if match and match in pid_l:
            return str(rule.get("category_path") or "")
    return None


def tree_roots() -> list[dict[str, Any]]:
    tree = load_material_category_tree()
    roots = list(tree.get("roots") or [])
    return sorted(roots, key=lambda r: int(r.get("sort_order") or 0))


def category_label(path: str, tree: dict[str, Any] | None = None) -> str:
    """Human label for a category path (e.g. industrial/steel → Steel)."""
    if tree is None:
        tree = load_material_category_tree()
    if not path or path == "all":
        return "All"
    parts = [p.strip() for p in path.split("/") if p.strip()]
    if not parts:
        return path
    leaf = parts[-1]
    for root in tree.get("roots") or []:
        if str(root.get("id")) == parts[0]:
            if len(parts) == 1:
                return str(root.get("label") or parts[0])
            for child in root.get("children") or []:
                if str(child.get("id")) == leaf:
                    return str(child.get("label") or leaf)
            return leaf.replace("_", " ").title()
    return leaf.replace("_", " ").title()


def refresh_aps_mat_003_witness() -> bool:
    tree = load_material_category_tree()
    roots = tree_roots()
    widget_src = (repo_root() / "tools/mcp/art_pipeline_suite/material_library_widget.py").read_text(
        encoding="utf-8"
    )
    uses_tree = "material_category_tree" in widget_src and "tree_roots" in widget_src
    infer_ok = infer_category_from_tree("steel_panel_01") == "industrial/steel"
    payload = {
        "program_id": "APS-MAT-003",
        "gate": "APS-MAT-003",
        "green": bool(roots) and uses_tree and infer_ok,
        "tree_id": tree.get("tree_id"),
        "root_count": len(roots),
        "infer_rules_count": len(tree.get("infer_rules") or []),
        "profile_bindings_count": len(tree.get("profile_bindings") or []),
        "widget_wired": uses_tree,
        "infer_sample": infer_category_from_tree("steel_panel_01"),
    }
    path = repo_root() / APS_MAT_003_WITNESS
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
    return bool(payload["green"])
