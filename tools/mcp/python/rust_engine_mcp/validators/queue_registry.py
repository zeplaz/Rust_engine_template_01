"""MCP-WIT-010 — authoritative queue registry loader."""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any, Iterator

from rust_engine_mcp.paths import repo_root, schemas_dir

REGISTRY_REL = "tools/mcp/schemas/queue_registry_v1.json"

_DEFAULT_CLOSED = frozenset({"done", "lib_done", "signed", "closed"})
_DEFAULT_OPEN = frozenset({"blocked", "paused", "deferred", "open", "reopened"})
_DEFAULT_READY = frozenset({"ready", "active", "in_progress"})


def _resolve(path: str | Path) -> Path:
    p = Path(path)
    if not p.is_absolute():
        p = repo_root() / p
    return p.resolve()


def _rel(path: Path, root: Path) -> str:
    try:
        return str(path.relative_to(root)).replace("\\", "/")
    except ValueError:
        return str(path).replace("\\", "/")


def load_queue_registry(*, repo: Path | None = None) -> dict[str, Any]:
    root = repo or repo_root()
    return json.loads((root / REGISTRY_REL).read_text(encoding="utf-8"))


def validate_queue_registry(*, repo: Path | None = None) -> None:
    import jsonschema

    root = repo or repo_root()
    doc = load_queue_registry(repo=root)
    schema = json.loads((schemas_dir() / "queue_registry_v1.schema.json").read_text(encoding="utf-8"))
    jsonschema.validate(instance=doc, schema=schema)


def _normalization_sets(registry: dict[str, Any]) -> tuple[frozenset[str], frozenset[str], frozenset[str]]:
    block = registry.get("status_normalization") or {}
    closed = frozenset(str(x).lower() for x in block.get("closed") or _DEFAULT_CLOSED)
    open_ = frozenset(str(x).lower() for x in block.get("open") or _DEFAULT_OPEN)
    ready = frozenset(str(x).lower() for x in block.get("ready") or _DEFAULT_READY)
    return closed, open_, ready


def normalize_queue_status(raw: str, *, registry: dict[str, Any] | None = None) -> str:
    registry = registry or load_queue_registry()
    closed, open_, ready = _normalization_sets(registry)
    s = str(raw or "").strip().lower()
    if s in closed:
        return "closed"
    if s in open_:
        return "open"
    if s in ready:
        return "ready"
    return s or "unknown"


def _get_dot(data: Any, dot_path: str) -> Any:
    cur = data
    for part in dot_path.split("."):
        if isinstance(cur, dict):
            cur = cur.get(part)
        elif isinstance(cur, list) and part.isdigit():
            cur = cur[int(part)]
        else:
            return None
    return cur


def _witness_rel_from_row(row: dict[str, Any], entry: dict[str, Any]) -> str:
    for field in entry.get("witness_fields") or ("witness", "witness_json"):
        val = str(row.get(field) or "").strip()
        if val:
            return val
    return ""


def iter_queue_rows(queue_doc: dict[str, Any], entry: dict[str, Any]) -> Iterator[dict[str, Any]]:
    rows_path = str(entry.get("rows_path") or "rows")
    if rows_path == "p2_tasks":
        yield from queue_doc.get("p2_tasks") or []
        for bucket in entry.get("synthetic_done_buckets") or []:
            dot = str(bucket.get("dot_path") or "")
            status = str(bucket.get("status") or "done")
            ids = _get_dot(queue_doc, dot)
            if isinstance(ids, list):
                for slice_id in ids:
                    yield {
                        "id": str(slice_id),
                        "status": status,
                        "_synthetic_done_bucket": dot,
                    }
        return
    block = queue_doc.get(rows_path)
    if isinstance(block, list):
        yield from block


def iter_registry_rows(
    registry: dict[str, Any] | None = None,
    *,
    repo: Path | None = None,
    queue_filter: str | None = None,
) -> Iterator[tuple[str, str, dict[str, Any]]]:
    """Yield (queue_file_rel, queue_id, row)."""
    root = repo or repo_root()
    registry = registry or load_queue_registry(repo=root)
    needle = (queue_filter or "").strip().lower()
    for entry in registry.get("queues") or []:
        queue_id = str(entry.get("queue_id") or "")
        if needle and needle not in {queue_id.lower(), str(entry.get("path") or "").lower()}:
            continue
        qpath = root / str(entry.get("path") or "")
        if not qpath.is_file():
            continue
        try:
            queue_doc = json.loads(qpath.read_text(encoding="utf-8"))
        except (OSError, json.JSONDecodeError):
            continue
        qrel = _rel(qpath, root)
        for row in iter_queue_rows(queue_doc, entry):
            if isinstance(row, dict):
                yield qrel, queue_id, row
