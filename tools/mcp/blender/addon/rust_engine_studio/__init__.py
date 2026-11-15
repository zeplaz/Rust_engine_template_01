"""Rust Engine Studio — Blender addon sharing StudioTask / headless job ops.

Install: Blender → Preferences → Add-ons → Install → select this folder's parent
zip, or symlink ``tools/mcp/blender/addon/rust_engine_studio`` into Blender addons.

Operators wrap the same headless scripts under ``tools/mcp/blender/scripts/``.
"""

from __future__ import annotations

bl_info = {
    "name": "Rust Engine Studio",
    "author": "Rust Engine",
    "version": (1, 0, 0),
    "blender": (3, 6, 0),
    "location": "View3D > Sidebar > Rust Engine",
    "description": "Enqueue/poll StudioTasks and run geometry/assembly jobs",
    "category": "Import-Export",
}

import importlib.util
import json
import sys
from pathlib import Path

import bpy
from bpy.props import StringProperty
from bpy.types import Operator, Panel


def _repo_root_from_addon() -> Path:
    # .../tools/mcp/blender/addon/rust_engine_studio/__init__.py → repo
    return Path(__file__).resolve().parents[5]


def _ensure_mcp_path() -> Path:
    root = _repo_root_from_addon()
    py = root / "tools" / "mcp" / "python"
    if str(py) not in sys.path:
        sys.path.insert(0, str(py))
    scripts = root / "tools" / "mcp" / "blender" / "scripts"
    if str(scripts) not in sys.path:
        sys.path.insert(0, str(scripts))
    return root


class RUSTENGINE_OT_poll_task(Operator):
    bl_idname = "rustengine.poll_studio_task"
    bl_label = "Poll Studio Task"
    task_id: StringProperty(name="Task ID", default="")

    def execute(self, context):
        _ensure_mcp_path()
        from rust_engine_mcp import studio_task

        tid = self.task_id.strip() or getattr(context.scene, "rustengine_task_id", "")
        st = studio_task.read_task(tid)
        if st is None:
            self.report({"ERROR"}, f"task not found: {tid}")
            return {"CANCELLED"}
        self.report({"INFO"}, f"{tid}: {st.get('status')}")
        print(json.dumps(st, indent=2))
        return {"FINISHED"}


class RUSTENGINE_OT_run_geometry(Operator):
    bl_idname = "rustengine.run_geometry_job"
    bl_label = "Run Geometry Job JSON"
    job_path: StringProperty(name="Job JSON", subtype="FILE_PATH", default="")

    def execute(self, context):
        root = _ensure_mcp_path()
        from rust_engine_mcp import studio_task
        from rust_engine_mcp.paths import jobs_root

        path = Path(self.job_path)
        if not path.is_file():
            self.report({"ERROR"}, f"missing job: {path}")
            return {"CANCELLED"}
        job = json.loads(path.read_text(encoding="utf-8"))
        seed = int((job.get("params") or {}).get("seed", job.get("seed", 0)))
        task = studio_task.enqueue(
            kind="geometry",
            seed=seed,
            payload_path=str(path),
            task_id=str(job.get("job_id") or f"geo_{path.stem}"),
            blender_session="addon",
        )
        # Interactive path: import headless runner into this Blender process.
        run_job = root / "tools" / "mcp" / "blender" / "scripts" / "run_job.py"
        if not run_job.is_file():
            self.report({"ERROR"}, "run_job.py missing")
            return {"CANCELLED"}
        studio_task.mark_status(task["task_id"], "running")
        # Delegate to the same script argv interface.
        sys.argv = ["run_job.py", "--", "--job", str(path)]
        try:
            spec = importlib.util.spec_from_file_location("rustengine_run_job", run_job)
            assert spec and spec.loader
            mod = importlib.util.module_from_spec(spec)
            spec.loader.exec_module(mod)
            if hasattr(mod, "main"):
                mod.main()
            studio_task.mark_status(task["task_id"], "done")
            self.report({"INFO"}, f"done {task['task_id']}")
            return {"FINISHED"}
        except Exception as exc:  # noqa: BLE001 — surface to Blender UI
            studio_task.mark_status(task["task_id"], "failed", error=str(exc))
            self.report({"ERROR"}, str(exc))
            return {"CANCELLED"}


class RUSTENGINE_OT_open_staging(Operator):
    bl_idname = "rustengine.open_staging"
    bl_label = "Open Staging Folder"

    def execute(self, context):
        root = _ensure_mcp_path()
        staging = root / "assets" / "staging"
        staging.mkdir(parents=True, exist_ok=True)
        bpy.ops.wm.path_open(filepath=str(staging))
        return {"FINISHED"}


class RUSTENGINE_OT_consume_handoff(Operator):
    bl_idname = "rustengine.consume_handoff"
    bl_label = "Consume APS Handoff"

    def execute(self, context):
        _ensure_mcp_path()
        from rust_engine_mcp import studio_task

        markers = sorted(studio_task.studio_tasks_dir().glob("*.blender_handoff"))
        if not markers:
            self.report({"WARNING"}, "no handoff markers")
            return {"CANCELLED"}
        data = json.loads(markers[-1].read_text(encoding="utf-8"))
        context.scene.rustengine_task_id = str(data.get("task_id") or "")
        payload = data.get("payload_path") or ""
        self.report({"INFO"}, f"loaded handoff {context.scene.rustengine_task_id}")
        if payload:
            context.scene.rustengine_job_path = payload
        return {"FINISHED"}


class RUSTENGINE_PT_panel(Panel):
    bl_label = "Rust Engine Studio"
    bl_idname = "RUSTENGINE_PT_panel"
    bl_space_type = "VIEW_3D"
    bl_region_type = "UI"
    bl_category = "Rust Engine"

    def draw(self, context):
        layout = self.layout
        sc = context.scene
        layout.prop(sc, "rustengine_task_id")
        layout.prop(sc, "rustengine_job_path")
        layout.operator("rustengine.consume_handoff")
        layout.operator("rustengine.poll_studio_task").task_id = sc.rustengine_task_id
        op = layout.operator("rustengine.run_geometry_job")
        op.job_path = sc.rustengine_job_path
        layout.operator("rustengine.open_staging")


_CLASSES = (
    RUSTENGINE_OT_poll_task,
    RUSTENGINE_OT_run_geometry,
    RUSTENGINE_OT_open_staging,
    RUSTENGINE_OT_consume_handoff,
    RUSTENGINE_PT_panel,
)


def register():
    bpy.types.Scene.rustengine_task_id = StringProperty(name="Task ID", default="")
    bpy.types.Scene.rustengine_job_path = StringProperty(
        name="Job JSON", subtype="FILE_PATH", default=""
    )
    for cls in _CLASSES:
        bpy.utils.register_class(cls)


def unregister():
    for cls in reversed(_CLASSES):
        bpy.utils.unregister_class(cls)
    del bpy.types.Scene.rustengine_task_id
    del bpy.types.Scene.rustengine_job_path


if __name__ == "__main__":
    register()
