"""P1 — Blender addon package is present and bl_info parses without importing bpy."""

from __future__ import annotations

import ast
from pathlib import Path


def test_addon_init_has_bl_info():
    root = Path(__file__).resolve().parents[2]  # tools/mcp
    init = root / "blender" / "addon" / "rust_engine_studio" / "__init__.py"
    assert init.is_file()
    tree = ast.parse(init.read_text(encoding="utf-8"))
    found = False
    for node in tree.body:
        if isinstance(node, ast.Assign):
            for t in node.targets:
                if isinstance(t, ast.Name) and t.id == "bl_info":
                    found = True
    assert found
    readme = init.parent.parent / "README.md"
    assert readme.is_file()
    text = init.read_text(encoding="utf-8")
    assert "rustengine.poll_studio_task" in text
    assert "rustengine.escalate_param_frame" in text
    assert "studio_task.escalate_from_param_frame" in text
