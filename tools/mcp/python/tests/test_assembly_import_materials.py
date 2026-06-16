"""BUILD-WORKER-001 — assembly_import must use ops.export_glb (Blender sys.path)."""

from __future__ import annotations

from rust_engine_mcp.paths import repo_root


def test_assembly_import_uses_ops_export_glb_import():
    path = repo_root() / "tools/mcp/blender/scripts/ops/assembly_import.py"
    text = path.read_text(encoding="utf-8")
    assert "from ops.export_glb import apply_material_profile_to_meshes" in text
    assert "apply_snapshot_material_profiles" in text
    assert "from export_glb import" not in text
