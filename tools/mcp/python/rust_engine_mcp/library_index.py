"""Backward-compatible re-exports — prefer rust_engine_mcp.library."""

from __future__ import annotations

from .library import (
    collect_entries,
    entry_from_promoted,
    format_index_ron,
    index_ron_path,
    register_module as register_job,
    write_module_index,
)

__all__ = [
    "collect_entries",
    "entry_from_promoted",
    "format_index_ron",
    "index_ron_path",
    "register_job",
    "write_module_index",
]
