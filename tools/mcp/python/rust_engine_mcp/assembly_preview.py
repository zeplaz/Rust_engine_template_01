"""APS-PREVIEW-002 — assembly snapshot multi-GLB preview (Bevy worker optional, three.js fallback)."""

from __future__ import annotations

import json
import os
import socket
import subprocess
import threading
import time
import uuid
import webbrowser
from dataclasses import dataclass
from http.server import BaseHTTPRequestHandler, HTTPServer
from pathlib import Path
from typing import Any, ClassVar

from .paths import repo_root

PREVIEW_JOBS_DIR = "debug_runs/preview_jobs"
APS_PREVIEW_WITNESS_JSON = "debug_runs/aps_preview_002_live.json"
PREVIEW_WORKER_WITNESS_JSON = "debug_runs/preview_worker_smoke_live.json"
PREVIEW_SCHEMA_VERSION = 1

# Keep HTTP servers alive (daemon threads alone are GC'd when CLI exits).
_ACTIVE_PREVIEW_SERVERS: list[HTTPServer] = []


@dataclass(frozen=True)
class PreviewPlacement:
    index: int
    node_id: str
    module_id: str
    glb_path: Path
    position: tuple[float, float, float]
    rotation_euler: tuple[float, float, float]
    material_profile: str


def _load_snapshot(path: Path) -> dict[str, Any]:
    data = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(data, dict):
        raise ValueError(f"assembly snapshot must be a JSON object: {path}")
    return data


def _vec3(raw: Any, default: tuple[float, float, float] = (0.0, 0.0, 0.0)) -> tuple[float, float, float]:
    if not isinstance(raw, (list, tuple)) or len(raw) < 3:
        return default
    return (float(raw[0]), float(raw[1]), float(raw[2]))


def collect_preview_placements(snapshot: dict[str, Any]) -> tuple[list[PreviewPlacement], list[str]]:
    """Resolve GLB paths from module_placements; return (resolved, missing node_ids)."""
    root = repo_root()
    resolved: list[PreviewPlacement] = []
    missing: list[str] = []
    for i, row in enumerate(snapshot.get("module_placements") or []):
        if not isinstance(row, dict):
            continue
        rel = str(row.get("glb_path") or row.get("glb") or "").strip()
        if not rel:
            missing.append(str(row.get("node_id") or f"placement_{i}"))
            continue
        glb = (root / rel.replace("\\", "/")).resolve()
        if not glb.is_file():
            missing.append(str(row.get("node_id") or rel))
            continue
        resolved.append(
            PreviewPlacement(
                index=i,
                node_id=str(row.get("node_id") or f"node_{i}"),
                module_id=str(row.get("module_id") or glb.parent.name),
                glb_path=glb,
                position=_vec3(row.get("position")),
                rotation_euler=_vec3(row.get("rotation_euler")),
                material_profile=str(row.get("material_profile") or ""),
            )
        )
    return resolved, missing


def write_preview_job(
    snapshot_path: Path,
    *,
    out_png: Path | None = None,
    job_id: str | None = None,
) -> Path:
    """Write Bevy worker job JSON per aps_preview_004_bevy_worker_v1.md."""
    snapshot_path = snapshot_path.resolve()
    job_id = job_id or f"preview_{snapshot_path.stem}_{uuid.uuid4().hex[:8]}"
    jobs_dir = repo_root() / PREVIEW_JOBS_DIR
    jobs_dir.mkdir(parents=True, exist_ok=True)
    png = out_png or (jobs_dir / f"{job_id}.png")
    png.parent.mkdir(parents=True, exist_ok=True)
    try:
        snap_rel = snapshot_path.relative_to(repo_root()).as_posix()
    except ValueError:
        snap_rel = snapshot_path.as_posix()
    try:
        png_rel = png.relative_to(repo_root()).as_posix()
    except ValueError:
        png_rel = png.as_posix()
    body = {
        "schema_version": PREVIEW_SCHEMA_VERSION,
        "operation": "preview_assembly",
        "job_id": job_id,
        "assembly_snapshot": snap_rel,
        "camera": {"preset": "iso_ne", "distance_m": 24.0},
        "output": {"png": png_rel, "width": 512, "height": 512},
    }
    job_path = jobs_dir / f"{job_id}.json"
    job_path.write_text(json.dumps(body, indent=2) + "\n", encoding="utf-8")
    return job_path


def _bevy_preview_disabled() -> bool:
    return os.environ.get("RUST_ENGINE_BEVY_PREVIEW", "").strip().lower() in (
        "0",
        "false",
        "no",
    )


def wait_for_preview_png(path: Path, *, timeout_s: float = 8.0, poll_s: float = 0.1) -> bool:
    """Poll until preview PNG exists with non-trivial size (Bevy worker flush)."""
    deadline = time.perf_counter() + max(0.0, timeout_s)
    while time.perf_counter() < deadline:
        if png_preview_usable(path):
            return True
        time.sleep(max(0.02, poll_s))
    return png_preview_usable(path)


def png_preview_usable(path: Path, *, min_bytes: int = 256) -> bool:
    """Reject missing, tiny, or nearly-all-black preview PNGs."""
    if not path.is_file():
        return False
    try:
        size = path.stat().st_size
    except OSError:
        return False
    if size < min_bytes:
        return False
    try:
        from PIL import Image
    except ImportError:
        return True
    try:
        with Image.open(path) as img:
            gray = img.convert("L")
            lo, hi = gray.getextrema()
            if hi - lo < 24:
                return False
            if lo >= hi:
                return lo > 0
    except OSError:
        return False
    return True


def _bevy_worker_command(job_path: Path, root: Path) -> list[str]:
    rel_job = str(job_path.relative_to(root) if job_path.is_relative_to(root) else job_path)
    for rel in (
        "target/release/bevy_preview_worker.exe",
        "target/debug/bevy_preview_worker.exe",
        "target/release/bevy_preview_worker",
        "target/debug/bevy_preview_worker",
    ):
        exe = root / rel.replace("/", os.sep)
        if exe.is_file():
            return [str(exe), "preview-assembly", rel_job]
    return [
        "cargo",
        "run",
        "--quiet",
        "--bin",
        "bevy_preview_worker",
        "--",
        "preview-assembly",
        rel_job,
    ]


def try_bevy_preview_worker(job_path: Path, *, timeout_s: float = 120.0) -> dict[str, Any] | None:
    """Spawn bevy_preview_worker (built binary preferred; cargo run fallback)."""
    if _bevy_preview_disabled():
        return None
    root = repo_root()
    env = os.environ.copy()
    env.setdefault("BEVY_ASSET_ROOT", str(root))
    env.setdefault("CARGO_MANIFEST_DIR", str(root))
    status_path = job_path.with_suffix(".status.json")
    cmd = _bevy_worker_command(job_path, root)
    try:
        proc = subprocess.run(
            cmd,
            cwd=str(root),
            capture_output=True,
            text=True,
            timeout=timeout_s,
            check=False,
            env=env,
        )
    except (FileNotFoundError, subprocess.TimeoutExpired) as exc:
        return {"status": "failed", "error": str(exc), "mode": "bevy_worker"}
    if status_path.is_file():
        status = json.loads(status_path.read_text(encoding="utf-8"))
        png_rel = str(status.get("png") or "")
        if png_rel:
            png_path = root / png_rel.replace("\\", "/")
            wait_for_preview_png(png_path, timeout_s=min(8.0, timeout_s * 0.25))
            if not png_preview_usable(png_path):
                status = {
                    **status,
                    "status": "failed",
                    "error": "preview PNG missing, too small, or blank",
                    "mode": "bevy_worker",
                }
        return status
    if proc.returncode != 0:
        tail = (proc.stderr or proc.stdout or "").strip()[-400:]
        return {"status": "failed", "error": tail or f"exit {proc.returncode}", "mode": "bevy_worker"}
    return {"status": "done", "mode": "bevy_worker", "png": str(status_path.with_suffix(".png"))}


def _free_port() -> int:
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as sock:
        sock.bind(("127.0.0.1", 0))
        return int(sock.getsockname()[1])


def _three_js_html(title: str, placements_json: str) -> str:
    return f"""<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8"/>
  <title>{title}</title>
  <style>
    html, body {{ margin: 0; height: 100%; overflow: hidden; background: #1a1a1e; color: #ccc;
      font-family: system-ui, sans-serif; }}
    #bar {{ padding: 8px 12px; font-size: 13px; border-bottom: 1px solid #333; }}
    #canvas {{ width: 100%; height: calc(100% - 36px); display: block; }}
  </style>
</head>
<body>
  <div id="bar">{title} · three.js multi-GLB (APS-PREVIEW-002 degraded) · drag orbit · scroll zoom</div>
  <canvas id="canvas"></canvas>
  <script type="importmap">
  {{"imports":{{"three":"https://cdn.jsdelivr.net/npm/three@0.170.0/build/three.module.js",
    "three/addons/":"https://cdn.jsdelivr.net/npm/three@0.170.0/examples/jsm/"}}}}
  </script>
  <script type="module">
  import * as THREE from 'three';
  import {{ OrbitControls }} from 'three/addons/controls/OrbitControls.js';
  import {{ GLTFLoader }} from 'three/addons/loaders/GLTFLoader.js';
  const placements = {placements_json};
  const canvas = document.getElementById('canvas');
  const renderer = new THREE.WebGLRenderer({{ canvas, antialias: true }});
  renderer.setPixelRatio(window.devicePixelRatio);
  renderer.shadowMap.enabled = true;
  const scene = new THREE.Scene();
  scene.background = new THREE.Color(0x2a2a30);
  const camera = new THREE.PerspectiveCamera(45, 1, 0.05, 500);
  const controls = new OrbitControls(camera, canvas);
  controls.enableDamping = true;
  scene.add(new THREE.AmbientLight(0xffffff, 0.55));
  const key = new THREE.DirectionalLight(0xffffff, 1.1);
  key.position.set(8, 14, 10);
  key.castShadow = true;
  scene.add(key);
  const fill = new THREE.DirectionalLight(0xaaccff, 0.35);
  fill.position.set(-6, 4, -8);
  scene.add(fill);
  const loader = new GLTFLoader();
  const group = new THREE.Group();
  scene.add(group);
  let pending = placements.length;
  const box = new THREE.Box3();
  placements.forEach((p) => {{
    loader.load(p.url, (gltf) => {{
      const root = gltf.scene;
      root.position.set(p.position[0], p.position[1], p.position[2]);
      root.rotation.set(p.rotation[0], p.rotation[1], p.rotation[2], 'XYZ');
      root.traverse((c) => {{ if (c.isMesh) {{ c.castShadow = true; c.receiveShadow = true; }} }});
      group.add(root);
      box.expandByObject(root);
      pending -= 1;
      if (pending === 0) frameCamera();
    }}, undefined, () => {{ pending -= 1; if (pending === 0) frameCamera(); }});
  }});
  function frameCamera() {{
    const center = box.getCenter(new THREE.Vector3());
    const size = box.getSize(new THREE.Vector3());
    const span = Math.max(size.x, size.y, size.z, 2);
    const dist = span * 2.2;
    camera.position.set(center.x + dist * 0.85, center.y + dist * 0.65, center.z + dist * 0.85);
    controls.target.copy(center);
    controls.update();
  }}
  function resize() {{
    const w = canvas.clientWidth, h = canvas.clientHeight;
    renderer.setSize(w, h, false);
    camera.aspect = w / Math.max(h, 1);
    camera.updateProjectionMatrix();
  }}
  window.addEventListener('resize', resize);
  resize();
  if (placements.length === 0) frameCamera();
  function tick() {{ requestAnimationFrame(tick); controls.update(); renderer.render(scene, camera); }}
  tick();
  </script>
</body>
</html>"""


class _AssemblyPreviewHandler(BaseHTTPRequestHandler):
    html: ClassVar[bytes]
    glb_by_id: ClassVar[dict[int, Path]]
    placements_payload: ClassVar[str]

    def log_message(self, _fmt: str, *_args) -> None:  # noqa: ANN001
        return

    def do_GET(self) -> None:
        if self.path in ("/", "/index.html"):
            self.send_response(200)
            self.send_header("Content-Type", "text/html; charset=utf-8")
            self.end_headers()
            self.wfile.write(self.html)
            return
        if self.path == "/placements.json":
            payload = self.placements_payload.encode("utf-8")
            self.send_response(200)
            self.send_header("Content-Type", "application/json")
            self.send_header("Content-Length", str(len(payload)))
            self.end_headers()
            self.wfile.write(payload)
            return
        if self.path.startswith("/glb/"):
            try:
                idx = int(self.path.split("/")[-1])
            except ValueError:
                self.send_error(404)
                return
            glb = self.glb_by_id.get(idx)
            if glb is None or not glb.is_file():
                self.send_error(404)
                return
            data = glb.read_bytes()
            self.send_response(200)
            self.send_header("Content-Type", "model/gltf-binary")
            self.send_header("Content-Length", str(len(data)))
            self.end_headers()
            self.wfile.write(data)
            return
        self.send_error(404)


def preview_assembly_browser(
    placements: list[PreviewPlacement],
    *,
    title: str = "Assembly preview",
    open_browser: bool = True,
) -> str:
    """Serve multi-GLB three.js preview; returns local URL."""
    if not placements:
        return "No resolved GLB placements to preview."

    glb_by_id = {p.index: p.glb_path for p in placements}
    payload = [
        {
            "url": f"/glb/{p.index}",
            "node_id": p.node_id,
            "module_id": p.module_id,
            "material_profile": p.material_profile,
            "position": list(p.position),
            "rotation": list(p.rotation_euler),
        }
        for p in placements
    ]
    html = _three_js_html(title, json.dumps(payload)).encode("utf-8")

    port = _free_port()
    handler = _AssemblyPreviewHandler
    handler.html = html
    handler.glb_by_id = glb_by_id
    handler.placements_payload = json.dumps(payload)
    server = HTTPServer(("127.0.0.1", port), handler)
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()
    _ACTIVE_PREVIEW_SERVERS.append(server)
    url = f"http://127.0.0.1:{port}/"
    if open_browser:
        webbrowser.open(url)
    return url


def shutdown_preview_servers() -> int:
    """Stop all preview HTTP servers started in this process."""
    stopped = 0
    for server in list(_ACTIVE_PREVIEW_SERVERS):
        try:
            server.shutdown()
            stopped += 1
        except Exception:
            pass
    _ACTIVE_PREVIEW_SERVERS.clear()
    return stopped


def _png_bytes_are_black(png: bytes, *, span_threshold: int = 16) -> bool:
    """True if a PNG byte blob is missing or visually (near-)uniformly black.

    The trimesh/pyglet offscreen path frequently returns an all-black image when
    the camera is not framing the geometry or the GL context degraded. Callers
    use this to fall back to a labeled placeholder instead of showing black.
    """
    if not png or len(png) < 64:
        return True
    try:
        from PIL import Image
    except ImportError:
        return False
    import io as _io

    try:
        with Image.open(_io.BytesIO(png)) as img:
            gray = img.convert("L")
            lo, hi = gray.getextrema()
    except Exception:
        return True
    # Uniform dark frame (e.g. (0, 0)) or a frame with no usable contrast.
    if hi - lo < span_threshold and hi < 24:
        return True
    return False


def try_render_glb_thumbnail_bytes(glb_path: Path, *, resolution: tuple[int, int] = (256, 256)) -> bytes | None:
    """Single-module thumbnail PNG bytes via trimesh (optional dependency).

    Returns ``None`` when trimesh is unavailable, the GLB cannot be loaded, the
    scene has no geometry, OR the render came back blank/near-black — in every
    one of those cases the caller is expected to degrade to a labeled placeholder
    rather than display a black tile (APS-PREVIEW-001 / B2).
    """
    try:
        import trimesh
    except ImportError:
        return None
    try:
        loaded = trimesh.load(str(glb_path), force="scene")
    except Exception:
        return None
    if isinstance(loaded, trimesh.Trimesh):
        scene = trimesh.Scene(loaded)
    else:
        scene = loaded
    if not getattr(scene, "geometry", None):
        return None

    # Frame the geometry explicitly. trimesh's default camera often points away
    # from a freshly loaded GLB (Y-up vs Z-up, off-center origin), which is the
    # most common cause of a fully black thumbnail. Setting a look-at transform
    # from the scene bounds makes the single-module preview reliable.
    try:
        bounds = scene.bounds
        if bounds is not None:
            center = bounds.mean(axis=0)
            extents = bounds[1] - bounds[0]
            span = float(max(extents)) or 1.0
            distance = span * 2.2
            scene.camera_transform = scene.camera.look_at(
                points=bounds,
                center=center,
                distance=distance,
            )
    except Exception:
        # Framing is best-effort; fall through to the default camera.
        pass

    png: bytes | None = None
    try:
        png = scene.save_image(resolution=resolution)
    except Exception:
        png = None
    if png is None:
        return None
    if _png_bytes_are_black(png):
        return None
    return png


def try_render_thumbnail_png(placements: list[PreviewPlacement], out_png: Path) -> bool:
    """Best-effort orthographic PNG via trimesh (optional dependency)."""
    try:
        import trimesh
        import numpy as np
    except ImportError:
        return False
    scene = trimesh.Scene()
    for p in placements[:48]:
        try:
            loaded = trimesh.load(str(p.glb_path), force="scene")
        except Exception:
            continue
        if isinstance(loaded, trimesh.Trimesh):
            loaded = trimesh.Scene(loaded)
        tx = trimesh.transformations.euler_matrix(
            p.rotation_euler[0], p.rotation_euler[1], p.rotation_euler[2], axes="sxyz"
        )
        tx[0:3, 3] = p.position
        scene.add_geometry(loaded, transform=tx)
    if not scene.geometry:
        return False
    try:
        png_bytes = scene.save_image(resolution=(512, 512))
    except Exception:
        return False
    if not png_bytes:
        return False
    out_png.parent.mkdir(parents=True, exist_ok=True)
    out_png.write_bytes(png_bytes)
    return out_png.is_file()


def preview_assembly(
    snapshot_path: Path | str,
    *,
    out_png: Path | str | None = None,
    open_browser: bool = True,
    try_bevy: bool = True,
    serve_seconds: float = 0.0,
) -> dict[str, Any]:
    """Preview assembly snapshot — Bevy worker when enabled, else three.js multi-GLB."""
    raw = Path(snapshot_path)
    if raw.is_file():
        snapshot_path = raw.resolve()
    else:
        candidate = (repo_root() / raw).resolve()
        snapshot_path = candidate if candidate.is_file() else raw.resolve()
    snapshot = _load_snapshot(snapshot_path)
    placements, missing = collect_preview_placements(snapshot)
    assembly_id = str(snapshot.get("assembly_id") or snapshot_path.stem)

    jobs_dir = repo_root() / PREVIEW_JOBS_DIR
    jobs_dir.mkdir(parents=True, exist_ok=True)
    png_path = Path(out_png).resolve() if out_png else jobs_dir / f"{assembly_id}_thumb.png"

    job_path = write_preview_job(snapshot_path, out_png=png_path)
    mode = "browser_threejs"
    preview_url = ""
    bevy_status: dict[str, Any] | None = None
    elapsed_ms = 0

    if try_bevy:
        t0 = time.perf_counter()
        bevy_status = try_bevy_preview_worker(job_path)
        elapsed_ms = int((time.perf_counter() - t0) * 1000)
        if bevy_status and bevy_status.get("status") == "done":
            mode = "bevy_worker"
            png_from_bevy = bevy_status.get("png")
            if png_from_bevy:
                candidate = repo_root() / str(png_from_bevy).replace("\\", "/")
                if candidate.is_file():
                    png_path = candidate
            if not png_preview_usable(png_path):
                bevy_status = {
                    **(bevy_status or {}),
                    "status": "failed",
                    "error": "Bevy worker PNG blank or unusable — falling back to browser",
                    "mode": "bevy_worker",
                }
                mode = "browser_threejs"
            else:
                write_preview_worker_smoke_witness(
                    {
                        "mode": mode,
                        "png": _rel_repo(png_path) if png_path.is_file() else "",
                        "modules_loaded": len(placements),
                        "elapsed_ms": elapsed_ms,
                        "missing_glb": missing[:32],
                        "bevy_status": bevy_status,
                    }
                )

    if mode != "bevy_worker":
        if placements:
            title = f"{assembly_id} · {len(placements)} modules"
            preview_url = preview_assembly_browser(placements, title=title, open_browser=open_browser)
            if not png_path.is_file():
                try_render_thumbnail_png(placements, png_path)
        mode = "browser_threejs"

    server_note = ""
    if preview_url and serve_seconds > 0:
        server_note = f"server_alive_for_s={serve_seconds}"
        time.sleep(max(0.0, float(serve_seconds)))

    profiles = sorted({p.material_profile for p in placements if p.material_profile})
    green = bool(placements) and len(missing) == 0

    return {
        "gate_id": "APS-PREVIEW-002",
        "green": green,
        "mode": mode,
        "assembly_id": assembly_id,
        "snapshot": _rel_repo(snapshot_path),
        "modules_loaded": len(placements),
        "modules_requested": len(snapshot.get("module_placements") or []),
        "missing_glb": missing[:32],
        "material_profiles_sample": profiles[:8],
        "preview_url": preview_url,
        "preview_url_hint": (
            "Open in browser while this process runs. CLI without --serve-seconds exits and kills the server."
            if preview_url and serve_seconds <= 0
            else ""
        ),
        "server_note": server_note,
        "preview_job": _rel_repo(job_path),
        "png": _rel_repo(png_path) if png_path.is_file() else "",
        "bevy_status": bevy_status,
        "elapsed_ms": elapsed_ms,
    }


def _rel_repo(path: Path) -> str:
    try:
        return path.relative_to(repo_root()).as_posix()
    except ValueError:
        return path.as_posix()


def write_preview_worker_smoke_witness(result: dict[str, Any]) -> Path:
    """APS-PREVIEW-004 — Bevy worker path witness."""
    out = repo_root() / PREVIEW_WORKER_WITNESS_JSON
    out.parent.mkdir(parents=True, exist_ok=True)
    body = {
        "gate_id": "APS-PREVIEW-004",
        "green": result.get("mode") == "bevy_worker" and bool(result.get("png")),
        "mode": result.get("mode"),
        "png": result.get("png"),
        "modules_loaded": result.get("modules_loaded"),
        "elapsed_ms": result.get("elapsed_ms"),
        "missing_glb": result.get("missing_glb") or [],
    }
    bevy = result.get("bevy_status") or {}
    if bevy:
        body["bevy_status"] = bevy
    out.write_text(json.dumps(body, indent=2) + "\n", encoding="utf-8")
    return out


def write_aps_preview_002_witness(result: dict[str, Any]) -> Path:
    out = repo_root() / APS_PREVIEW_WITNESS_JSON
    out.parent.mkdir(parents=True, exist_ok=True)
    body = {k: v for k, v in result.items() if k != "bevy_status" or v}
    out.write_text(json.dumps(body, indent=2) + "\n", encoding="utf-8")
    return out


def preview_assembly_from_dict(
    snapshot: dict[str, Any],
    *,
    out_png: Path | str | None = None,
    open_browser: bool = True,
    serve_seconds: float = 0.0,
) -> dict[str, Any]:
    """Preview in-memory snapshot (APS tab) via temp file."""
    jobs_dir = repo_root() / PREVIEW_JOBS_DIR
    jobs_dir.mkdir(parents=True, exist_ok=True)
    assembly_id = str(snapshot.get("assembly_id") or "assembly_preview")
    tmp = jobs_dir / f"{assembly_id}_preview_tmp.json"
    tmp.write_text(json.dumps(snapshot, indent=2) + "\n", encoding="utf-8")
    return preview_assembly(
        tmp,
        out_png=out_png,
        open_browser=open_browser,
        try_bevy=True,
        serve_seconds=serve_seconds,
    )
