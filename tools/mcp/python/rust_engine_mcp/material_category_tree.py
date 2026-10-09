"""APS-MAT-003 — load material_category_tree_v1.json for Materials tab browse."""

from __future__ import annotations

import json
import re
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


def _id_tokens(value: str) -> list[str]:
    return [token for token in re.split(r"[^a-z0-9]+", value.lower()) if token]


def _rule_matches(match: str, profile_id: str) -> bool:
    """Whole tokens only — ``wood`` matches ``wood_siding``, not ``plywood``."""
    needle = _id_tokens(match)
    if not needle:
        return False
    tokens = _id_tokens(profile_id)
    width = len(needle)
    return any(tokens[i : i + width] == needle for i in range(len(tokens) - width + 1))


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
    rules = sorted(
        (r for r in (tree.get("infer_rules") or []) if isinstance(r, dict) and r.get("match")),
        key=lambda r: int(r.get("priority") or 0),
        reverse=True,
    )
    for rule in rules:
        if _rule_matches(str(rule.get("match") or ""), pid):
            return str(rule.get("category_path") or "")
    return None


def tree_roots() -> list[dict[str, Any]]:
    tree = load_material_category_tree()
    roots = list(tree.get("roots") or [])
    return sorted(roots, key=lambda r: int(r.get("sort_order") or 0))


def _label_at(nodes: list[Any], parts: list[str]) -> str | None:
    if not parts:
        return None
    head, rest = parts[0], parts[1:]
    for node in nodes:
        if not isinstance(node, dict) or str(node.get("id")) != head:
            continue
        if not rest:
            return str(node.get("label") or head)
        nested = _label_at(list(node.get("children") or []), rest)
        if nested:
            return nested
        return None
    return None


def category_label(path: str, tree: dict[str, Any] | None = None) -> str:
    """Human label for a category path (e.g. industrial/steel → Steel).

    Walks each path segment. A later segment does not borrow a sibling label.
    """
    if tree is None:
        tree = load_material_category_tree()
    if not path or path == "all":
        return "All"
    parts = [p.strip() for p in path.split("/") if p.strip()]
    if not parts:
        return path
    found = _label_at(list(tree.get("roots") or []), parts)
    if found:
        return found
    return parts[-1].replace("_", " ").title()


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
