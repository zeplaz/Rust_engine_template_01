"""Foolproof launcher — no PYTHONPATH required.

  python tools/mcp/module_viewer/run.py

Requires rust_engine_mcp deps (jsonschema, etc.):
  cd tools/mcp/python && pip install -r ../requirements.txt && pip install -e .
Or: .\\tools\\mcp\\install_designer_mcp.ps1
"""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path

_MCP_ROOT = Path(__file__).resolve().parent.parent  # tools/mcp
if str(_MCP_ROOT) not in sys.path:
    sys.path.insert(0, str(_MCP_ROOT))
_PY_ROOT = _MCP_ROOT / "python"
if str(_PY_ROOT) not in sys.path:
    sys.path.insert(0, str(_PY_ROOT))


def _ensure_rust_engine_mcp_deps() -> None:
    try:
        import jsonschema  # noqa: F401
    except ModuleNotFoundError:
        req = _MCP_ROOT / "requirements.txt"
        print(
            "Module Kit Viewer needs rust_engine_mcp dependencies (missing jsonschema).\n"
            f"  {_PY_ROOT}\n"
            f"  python -m pip install -r {req}\n"
            f"  python -m pip install -e {_PY_ROOT}\n"
            "Or run: tools/mcp/install_designer_mcp.ps1\n"
            "Use Python 3.13 (not bare 3.14) — see tools/mcp/README.md.",
            file=sys.stderr,
        )
        if sys.stdin.isatty() and input("Install now with this interpreter? [y/N] ").strip().lower() == "y":
            subprocess.check_call(
                [sys.executable, "-m", "pip", "install", "-r", str(req)],
            )
            subprocess.check_call(
                [sys.executable, "-m", "pip", "install", "-e", str(_PY_ROOT)],
            )
            import jsonschema  # noqa: F401
        else:
            raise SystemExit(1) from None


_ensure_rust_engine_mcp_deps()

from module_viewer.app import run_app  # noqa: E402 — APS suite shim

if __name__ == "__main__":
    run_app()
