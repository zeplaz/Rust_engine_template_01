"""Browser-based GLB preview via local HTTP + model-viewer."""

from __future__ import annotations

import socket
import threading
import webbrowser
from http.server import BaseHTTPRequestHandler, HTTPServer
from pathlib import Path
from typing import ClassVar


class _PreviewHandler(BaseHTTPRequestHandler):
    glb_path: ClassVar[Path]
    html: ClassVar[bytes]

    def log_message(self, _fmt: str, *_args) -> None:  # noqa: ANN001
        return

    def do_GET(self) -> None:
        if self.path in ("/", "/index.html"):
            self.send_response(200)
            self.send_header("Content-Type", "text/html; charset=utf-8")
            self.end_headers()
            self.wfile.write(self.html)
            return
        if self.path == "/model.glb":
            data = self.glb_path.read_bytes()
            self.send_response(200)
            self.send_header("Content-Type", "model/gltf-binary")
            self.send_header("Content-Length", str(len(data)))
            self.end_headers()
            self.wfile.write(data)
            return
        self.send_error(404)


def _free_port() -> int:
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as sock:
        sock.bind(("127.0.0.1", 0))
        return int(sock.getsockname()[1])


def preview_in_browser(glb_path: Path, *, title: str = "Module preview") -> str:
    glb_path = glb_path.resolve()
    if not glb_path.is_file():
        return f"GLB not found: {glb_path}"

    html = f"""<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8"/>
  <title>{title}</title>
  <script type="module" src="https://ajax.googleapis.com/ajax/libs/model-viewer/3.5.0/model-viewer.min.js"></script>
  <style>
    html, body {{ margin: 0; height: 100%; background: #1e1e1e; color: #ccc; font-family: system-ui, sans-serif; }}
    #bar {{ padding: 8px 12px; font-size: 13px; border-bottom: 1px solid #333; }}
    model-viewer {{ width: 100%; height: calc(100% - 36px); background: #2a2a2a; }}
  </style>
</head>
<body>
  <div id="bar">{title} · {glb_path.name} · drag to orbit · scroll to zoom</div>
  <model-viewer src="/model.glb" camera-controls touch-action="pan-y"
    shadow-intensity="0.8" exposure="1.1" environment-image="neutral" alt="module"></model-viewer>
</body>
</html>
""".encode("utf-8")

    port = _free_port()
    handler = _PreviewHandler
    handler.glb_path = glb_path
    handler.html = html
    server = HTTPServer(("127.0.0.1", port), handler)
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()
    url = f"http://127.0.0.1:{port}/"
    webbrowser.open(url)
    return url
