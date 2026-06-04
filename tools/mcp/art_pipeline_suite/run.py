"""Art Pipeline Suite launcher."""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path

_MCP_ROOT = Path(__file__).resolve().parent.parent
if str(_MCP_ROOT) not in sys.path:
    sys.path.insert(0, str(_MCP_ROOT))
_PY_ROOT = _MCP_ROOT / "python"
if str(_PY_ROOT) not in sys.path:
    sys.path.insert(0, str(_PY_ROOT))


def _ensure_deps() -> None:
    try:
        import jsonschema  # noqa: F401
    except ModuleNotFoundError:
        req = _MCP_ROOT / "requirements.txt"
        print(
            "Art Pipeline Suite needs rust_engine_mcp dependencies.\n"
            f"  python -m pip install -r {req}\n"
            f"  python -m pip install -e {_PY_ROOT}",
            file=sys.stderr,
        )
        if sys.stdin.isatty() and input("Install now? [y/N] ").strip().lower() == "y":
            subprocess.check_call([sys.executable, "-m", "pip", "install", "-r", str(req)])
            subprocess.check_call([sys.executable, "-m", "pip", "install", "-e", str(_PY_ROOT)])
        else:
            raise SystemExit(1) from None


def main() -> None:
    _ensure_deps()
    from art_pipeline_suite.app import run_app

    run_app()


if __name__ == "__main__":
    main()
