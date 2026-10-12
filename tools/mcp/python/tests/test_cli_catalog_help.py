"""1A — micro_tool_help must track the live argparse catalog (no phantom commands)."""

from __future__ import annotations

import json

from rust_engine_mcp.cli import build_parser
from rust_engine_mcp.cli_catalog import catalog, mcp_help_payload, subparser_map


def test_mcp_help_payload_matches_live_parser() -> None:
    parser = build_parser()
    payload = mcp_help_payload(parser)
    live = {row["name"] for row in catalog(parser)}
    grouped = {name for names in payload["commands_by_group"].values() for name in names}
    # signatures are "name args" — strip to command name
    grouped_names = {sig.split()[0] for sig in grouped}
    assert payload["command_count"] == len(live)
    assert grouped_names == live
    assert "file-digest" in live
    assert "token-savings-guide" in live
    assert "hint" in payload


def test_micro_tool_help_server_uses_catalog() -> None:
    from rust_engine_mcp import server

    raw = server.micro_tool_help()
    data = json.loads(raw)
    live = set(subparser_map(build_parser()).keys())
    grouped_names = {
        sig.split()[0] for names in data["commands_by_group"].values() for sig in names
    }
    assert grouped_names == live
    assert data["command_count"] == len(live)
