#!/usr/bin/env python3
"""Refresh landscape grammar SIGN + preset-batch witnesses."""

from __future__ import annotations

import json
import sys
from pathlib import Path

MCP_PYTHON = Path(__file__).resolve().parents[1] / "python"
if str(MCP_PYTHON) not in sys.path:
    sys.path.insert(0, str(MCP_PYTHON))

from rust_engine_mcp.landscape_grammar_presets import (  # noqa: E402
    refresh_mcp_landscape_grammar_sign_witness,
    write_landscape_grammar_presets_witness,
)


def main() -> int:
    batch = write_landscape_grammar_presets_witness()
    sign = refresh_mcp_landscape_grammar_sign_witness()
    green = bool(batch.get("green")) and bool(sign.get("green"))
    print(
        json.dumps(
            {
                "green": green,
                "batch_witness": batch.get("written"),
                "sign_witness": sign.get("written"),
                "failed": (batch.get("batch") or {}).get("preset_validation", {}).get("failed", 0),
            },
            ensure_ascii=True,
        )
    )
    return 0 if green else 1


if __name__ == "__main__":
    raise SystemExit(main())
