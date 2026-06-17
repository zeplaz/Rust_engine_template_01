"""MCP-WIT-011 — cross-queue integrity validator."""

from __future__ import annotations

import json
import time
from pathlib import Path
from typing import Any

from rust_engine_mcp.paths import repo_root

from .queue_registry import (
    REGISTRY_REL,
    _witness_rel_from_row,
    iter_registry_rows,
    load_queue_registry,
    normalize_queue_status,
    validate_queue_registry,
)
from .report import KnownFix, ValidationIssue, ValidationReport
from .witness_honesty import _exit_predicate_fails, load_witness_integrity_catalog, validate_witness_honesty

WITNESS_REL = "debug_runs/queue_integrity_reconcile_live.json"

KNOWN_FIXES = (
    KnownFix(
        signature="WIT-QUEUE-CONTRADICTION",
        fix="Reconcile queue boards — same id cannot be done in one queue and blocked in another",
        confidence=0.92,
    ),
    KnownFix(
        signature="WIT-EXIT-PREDICATE",
        fix="Fix witness sub-fields or set status blocked until exit_predicate.must passes",
        confidence=0.9,
    ),
    KnownFix(
        signature="WIT-SNAG-DONE",
        fix="Clear snag field or set status reopened/blocked",
        confidence=0.88,
    ),
)


def _row_slice_id(row: dict[str, Any], id_field: str = "id") -> str:
    return str(row.get(id_field) or row.get("id") or "")


def _row_raw_status(row: dict[str, Any], status_field: str = "status") -> str:
    return str(row.get(status_field) or row.get("status") or "").strip().lower()


def witness_path_for_row(row: dict[str, Any], entry: dict[str, Any] | None = None) -> str:
    if entry:
        return _witness_rel_from_row(row, entry)
    for field in ("witness", "witness_json", "harness_witness"):
        val = str(row.get(field) or "").strip()
        if val:
            return val
    return ""


def check_row_done_allowed(
    row: dict[str, Any],
    *,
    root: Path | None = None,
    entry: dict[str, Any] | None = None,
) -> tuple[bool, str]:
    """MCP-WIT-013 — gate for agent_queue_update(..., enforce=True)."""
    root = root or repo_root()
    if not isinstance(row.get("exit_predicate"), dict):
        return False, "missing exit_predicate on queue row"
    witness_rel = witness_path_for_row(row, entry)
    if witness_rel and witness_rel.endswith(".json"):
        witness_path = root / witness_rel
        if witness_path.is_file():
            wdata = json.loads(witness_path.read_text(encoding="utf-8"))
            exit_fail = _exit_predicate_fails(wdata)
            if exit_fail:
                return False, exit_fail
            catalog = load_witness_integrity_catalog(repo=root)
            report = validate_witness_honesty(
                wdata,
                witness_rel=witness_rel.replace("\\", "/"),
                catalog=catalog,
                root=root,
                compression_level=3,
            )
            if report.status == "failed":
                return False, f"witness_honesty failed: {report.summary}"
        else:
            return False, f"witness missing: {witness_rel}"
    elif not isinstance(row.get("exit_predicate"), dict):
        return False, "missing exit_predicate on queue row"
    else:
        row_fail = _exit_predicate_fails(row)
        if row_fail:
            return False, row_fail
    return True, ""


def _entry_for_queue_id(registry: dict[str, Any], queue_id: str) -> dict[str, Any] | None:
    for entry in registry.get("queues") or []:
        if str(entry.get("queue_id") or "") == queue_id:
            return entry
    return None


def collect_queue_integrity(
    *,
    repo: Path | None = None,
    registry: dict[str, Any] | None = None,
    queue_filter: str | None = None,
) -> dict[str, Any]:
    root = repo or repo_root()
    registry = registry or load_queue_registry(repo=root)
    catalog = load_witness_integrity_catalog(repo=root)

    issues: list[ValidationIssue] = []
    status_by_id: dict[str, dict[str, str]] = {}
    contradictions: list[dict[str, Any]] = []
    stale_ids: list[dict[str, Any]] = []
    snag_done: list[dict[str, Any]] = []
    exit_predicate_failures: list[dict[str, Any]] = []

    entry_by_queue_id = {str(e.get("queue_id")): e for e in registry.get("queues") or []}

    for qrel, queue_id, row in iter_registry_rows(registry, repo=root, queue_filter=queue_filter):
        entry = entry_by_queue_id.get(queue_id) or {}
        id_field = str(entry.get("id_field") or "id")
        status_field = str(entry.get("status_field") or "status")
        slice_id = _row_slice_id(row, id_field)
        if not slice_id:
            continue

        raw_status = _row_raw_status(row, status_field)
        norm = normalize_queue_status(raw_status, registry=registry)
        status_by_id.setdefault(slice_id, {})[qrel] = norm

        snag = str(row.get("snag") or "").strip()
        if raw_status == "done" and snag:
            issues.append(
                ValidationIssue(
                    kind="WIT-SNAG-DONE",
                    severity="error",
                    file=qrel,
                    symbol=slice_id,
                    hint=f"{slice_id} status=done with snag={snag!r}",
                    signature="WIT-SNAG-DONE",
                )
            )
            snag_done.append({"id": slice_id, "queue": qrel, "snag": snag})
            stale_ids.append({"id": slice_id, "queue": qrel, "reason": "snag_done"})

        witness_rel = _witness_rel_from_row(row, entry)
        if raw_status == "done":
            if witness_rel and witness_rel.endswith(".json"):
                witness_path = root / witness_rel
                if witness_path.is_file():
                    try:
                        wdata = json.loads(witness_path.read_text(encoding="utf-8"))
                    except json.JSONDecodeError:
                        issues.append(
                            ValidationIssue(
                                kind="WIT-EXIT-PREDICATE",
                                severity="error",
                                file=witness_rel,
                                symbol=slice_id,
                                hint=f"{slice_id} done but witness unreadable: {witness_rel}",
                                signature="WIT-EXIT-PREDICATE",
                            )
                        )
                        stale_ids.append({"id": slice_id, "queue": qrel, "reason": "witness_unreadable"})
                        wdata = None
                    if wdata is not None:
                        exit_fail = _exit_predicate_fails(wdata)
                        if exit_fail:
                            issues.append(
                                ValidationIssue(
                                    kind="WIT-EXIT-PREDICATE",
                                    severity="error",
                                    file=witness_rel,
                                    symbol=slice_id,
                                    hint=f"{slice_id} done: {exit_fail}",
                                    signature="WIT-EXIT-PREDICATE",
                                )
                            )
                            exit_predicate_failures.append(
                                {"id": slice_id, "queue": qrel, "witness": witness_rel, "detail": exit_fail}
                            )
                            stale_ids.append({"id": slice_id, "queue": qrel, "reason": "exit_predicate"})
                        report = validate_witness_honesty(
                            wdata,
                            witness_rel=witness_rel.replace("\\", "/"),
                            catalog=catalog,
                            root=root,
                            compression_level=3,
                        )
                        if report.status == "failed":
                            issues.append(
                                ValidationIssue(
                                    kind="WIT-EXIT-PREDICATE",
                                    severity="error",
                                    file=witness_rel,
                                    symbol=slice_id,
                                    hint=f"{slice_id} done but witness_honesty failed",
                                    signature="WIT-EXIT-PREDICATE",
                                )
                            )
                            exit_predicate_failures.append(
                                {
                                    "id": slice_id,
                                    "queue": qrel,
                                    "witness": witness_rel,
                                    "detail": report.summary,
                                }
                            )
                            stale_ids.append({"id": slice_id, "queue": qrel, "reason": "witness_honesty"})
                elif witness_rel:
                    issues.append(
                        ValidationIssue(
                            kind="WIT-EXIT-PREDICATE",
                            severity="error",
                            file=witness_rel,
                            symbol=slice_id,
                            hint=f"{slice_id} done but witness missing: {witness_rel}",
                            signature="WIT-EXIT-PREDICATE",
                        )
                    )
                    stale_ids.append({"id": slice_id, "queue": qrel, "reason": "witness_missing"})
            elif not isinstance(row.get("exit_predicate"), dict) and not witness_rel:
                issues.append(
                    ValidationIssue(
                        kind="WIT-EXIT-PREDICATE",
                        severity="error",
                        file=qrel,
                        symbol=slice_id,
                        hint=f"{slice_id} done without exit_predicate or witness",
                        signature="WIT-EXIT-PREDICATE",
                    )
                )
                stale_ids.append({"id": slice_id, "queue": qrel, "reason": "missing_exit_predicate"})

    for slice_id, by_queue in status_by_id.items():
        norms = set(by_queue.values())
        if "closed" in norms and "open" in norms:
            queues = ", ".join(f"{q}={s}" for q, s in sorted(by_queue.items()))
            issues.append(
                ValidationIssue(
                    kind="WIT-QUEUE-CONTRADICTION",
                    severity="error",
                    file="queue_registry",
                    symbol=slice_id,
                    hint=f"{slice_id} contradiction: {queues}",
                    signature="WIT-QUEUE-CONTRADICTION",
                )
            )
            contradictions.append({"id": slice_id, "by_queue": dict(by_queue)})
            stale_ids.append({"id": slice_id, "queue": "cross_queue", "reason": "contradiction"})

    def _issue_priority(issue: ValidationIssue) -> int:
        if issue.signature == "WIT-QUEUE-CONTRADICTION":
            return 0
        if issue.signature == "WIT-SNAG-DONE":
            return 1
        return 2

    issues.sort(key=lambda i: (_issue_priority(i), i.symbol or "", i.file))
    errors = [i for i in issues if i.severity == "error"]
    return {
        "registry": REGISTRY_REL,
        "queue_filter": queue_filter,
        "issue_count": len(issues),
        "error_count": len(errors),
        "warning_count": len(issues) - len(errors),
        "contradiction_count": len(contradictions),
        "contradictions": contradictions,
        "snag_done": snag_done,
        "exit_predicate_failures": exit_predicate_failures,
        "stale_ids": stale_ids,
        "issues": issues,
        "green": len(errors) == 0,
    }


def validate_queue_integrity(
    *,
    queue_filter: str | None = None,
    compression_level: int = 3,
    repo: Path | None = None,
) -> ValidationReport:
    validate_queue_registry(repo=repo)
    body = collect_queue_integrity(repo=repo, queue_filter=queue_filter)
    issues: list[ValidationIssue] = body.get("issues") or []
    errors = [i for i in issues if i.severity == "error"]
    status: str = "failed" if errors else ("warning" if issues else "passed")
    fixes = [k for k in KNOWN_FIXES if any(i.signature == k.signature for i in issues)]
    return ValidationReport(
        validator="test",
        status=status,  # type: ignore[arg-type]
        compression_level=compression_level,
        summary=(
            f"queue_integrity: contradictions={body.get('contradiction_count')} "
            f"errors={len(errors)} stale={len(body.get('stale_ids') or [])}"
        ),
        error_count=len(errors),
        warning_count=len(issues) - len(errors),
        errors=issues,
        known_fixes=fixes,
        confidence=0.9 if not errors else 0.78,
    ).compress(compression_level)


def refresh_queue_integrity_reconcile_witness(*, repo: Path | None = None) -> dict[str, Any]:
    root = repo or repo_root()
    body = collect_queue_integrity(repo=root)
    errors = body.get("error_count") or 0
    contradictions = body.get("contradictions") or []
    green = bool(body.get("green"))
    witness_body: dict[str, Any] = {
        "gate": "MCP-WIT-014",
        "green": green,
        "verdict": "PASS" if green else "FAIL",
        "registry": REGISTRY_REL,
        "contradiction_count": body.get("contradiction_count"),
        "error_count": errors,
        "warning_count": body.get("warning_count"),
        "contradictions": contradictions[:24],
        "snag_done": (body.get("snag_done") or [])[:24],
        "exit_predicate_failures": (body.get("exit_predicate_failures") or [])[:24],
        "stale_ids": (body.get("stale_ids") or [])[:48],
        "proceed_bulk_reopen": False,
        "note": "green:false until contradictions triaged — report-only reconcile",
        "commands": [
            "python -m rust_engine_mcp.cli validate-report queue_integrity --compress 3",
            "python -m rust_engine_mcp.cli queue-integrity-reconcile-witness",
        ],
        "_agent_meta": {
            "schema": "queue_integrity_reconcile_live_v1",
            "written_at_epoch_secs": int(time.time()),
            "profile": "MCP_WIT_QUEUE_INTEGRITY",
            "source_system": "queue_integrity",
            "relative_path": WITNESS_REL,
            "ritual": "BLANG:WIT-HON queue_integrity" if green else None,
        },
    }
    out = root / WITNESS_REL
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(witness_body, indent=2) + "\n", encoding="utf-8")
    witness_body["written"] = WITNESS_REL
    return witness_body
