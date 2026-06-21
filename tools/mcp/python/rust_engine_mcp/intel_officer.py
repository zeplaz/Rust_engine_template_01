"""Intel officer — false-positive surveillance, cull candidates, supervised reopen."""

from __future__ import annotations

import json
import re
import time
from pathlib import Path
from typing import Any

from rust_engine_mcp import agent_queue
from rust_engine_mcp.paths import repo_root
from rust_engine_mcp.validators.queue_integrity import collect_queue_integrity, witness_path_for_row
from rust_engine_mcp.validators.queue_registry import iter_registry_rows, load_queue_registry
from rust_engine_mcp.validators.witness_honesty import (
    load_witness_integrity_catalog,
    validate_witness_honesty,
)

WITNESS_REL = "debug_runs/agent_ops/intel_officer_sweep_live.json"
SIGNOFF_REGISTRY = "tools/orchestrator/queues/designer_signoff_registry.json"

STUB_MARKERS = (
    "STUB",
    "TODO",
    "TBD",
    "PLACEHOLDER",
    "NOT IMPLEMENTED",
    "OUTLINE ONLY",
    "WAVE-0 OUTLINE",
    "SPEC_ONLY",
    "WIP",
    "COMING SOON",
    "TEMPLATE — REVIEW",
    "STUB TEMPLATE",
    "DRAFT ONLY",
)

ZERO_GREEN_FIELDS = (
    ("topology_tint_visible_chunks", 0),
    ("trees_spawned", 0),
    ("trees_visible", 0),
    ("pixel_heterogeneity_wired", False),
)

FINDING_SEVERITY = {
    "INTEL-QUEUE-STALE": "error",
    "INTEL-WITNESS-DISHONEST": "error",
    "INTEL-DONE-NO-WITNESS": "error",
    "INTEL-STUB-DELIVERABLE": "error",
    "INTEL-SPEC-ONLY-DONE": "warning",
    "INTEL-SIGNOFF-FALSE": "error",
    "INTEL-GREEN-ZERO-COUNT": "error",
    "INTEL-DONE-NO-EXIT": "warning",
}


def _rel(path: Path, root: Path) -> str:
    try:
        return str(path.relative_to(root)).replace("\\", "/")
    except ValueError:
        return str(path).replace("\\", "/")


def _deliverable_stub_reason(path: Path) -> str | None:
    if not path.is_file():
        return "deliverable_missing"
    try:
        raw = path.read_text(encoding="utf-8", errors="replace")
    except OSError:
        return "deliverable_unreadable"
    if path.suffix.lower() == ".md":
        stripped = raw.strip()
        if len(stripped) < 280:
            return "deliverable_too_short"
        upper = stripped.upper()
        for marker in STUB_MARKERS:
            if marker in upper:
                return f"stub_marker:{marker.lower()}"
        if re.search(r"^#\s+.+\n\n\s*(\(stub\)|\(draft\)|\(outline\))", stripped, re.I | re.M):
            return "stub_heading"
    if path.suffix.lower() == ".json":
        try:
            data = json.loads(raw)
        except json.JSONDecodeError:
            return "deliverable_json_invalid"
        if isinstance(data, dict):
            if data.get("spec_only") is True and data.get("ship") is not False:
                return "spec_only_json"
            if data.get("validator_status") == "passed" and str(data.get("art_quality", "")).startswith(
                "rejected"
            ):
                return "art_quality_rejected"
    return None


def _green_zero_count_reason(data: dict[str, Any]) -> str | None:
    if data.get("green") is not True:
        return None
    for field, bad in ZERO_GREEN_FIELDS:
        if field in data and data.get(field) == bad:
            return f"green_true_but_{field}={bad!r}"
    return None


def _finding(
    *,
    finding_id: str,
    signature: str,
    source: str,
    slice_id: str,
    reason: str,
    recommended: str,
    queue: str = "",
    witness: str = "",
    deliverable: str = "",
) -> dict[str, Any]:
    return {
        "id": finding_id,
        "signature": signature,
        "severity": FINDING_SEVERITY.get(signature, "warning"),
        "source": source,
        "slice_id": slice_id,
        "queue": queue,
        "witness": witness,
        "deliverable": deliverable,
        "reason": reason,
        "recommended_action": recommended,
    }


def _scan_done_rows(*, root: Path) -> list[dict[str, Any]]:
    findings: list[dict[str, Any]] = []
    registry = load_queue_registry(repo=root)
    seen: set[str] = set()

    for qrel, _qid, row in iter_registry_rows(registry, repo=root):
        slice_id = str(row.get("id") or "")
        if not slice_id or slice_id in seen:
            continue
        status = str(row.get("status") or "").lower()
        if status not in ("done", "signed", "lib_done", "closed"):
            continue
        seen.add(slice_id)

        witness_rel = witness_path_for_row(row)
        docs = row.get("docs") or []
        deliverable = str(row.get("deliverable") or (docs[0] if docs else ""))

        if str(row.get("spec_only") or row.get("note") or "").lower().find("spec_only") >= 0:
            findings.append(
                _finding(
                    finding_id=f"{slice_id}:spec-only",
                    signature="INTEL-SPEC-ONLY-DONE",
                    source="queue_row",
                    slice_id=slice_id,
                    queue=qrel,
                    witness=witness_rel,
                    deliverable=deliverable,
                    reason="row marked done with spec_only note",
                    recommended="reopen",
                )
            )

        if witness_rel.endswith(".json"):
            wpath = root / witness_rel
            if not wpath.is_file():
                findings.append(
                    _finding(
                        finding_id=f"{slice_id}:no-witness",
                        signature="INTEL-DONE-NO-WITNESS",
                        source="queue_row",
                        slice_id=slice_id,
                        queue=qrel,
                        witness=witness_rel,
                        reason="done without witness file on disk",
                        recommended="reopen",
                    )
                )
        elif not witness_rel and not isinstance(row.get("exit_predicate"), dict):
            findings.append(
                _finding(
                    finding_id=f"{slice_id}:no-exit",
                    signature="INTEL-DONE-NO-EXIT",
                    source="queue_row",
                    slice_id=slice_id,
                    queue=qrel,
                    reason="done without witness path or exit_predicate",
                    recommended="blocked",
                )
            )

        for rel in {deliverable, witness_rel}:
            if not rel or rel.endswith(".json"):
                continue
            stub = _deliverable_stub_reason(root / rel)
            if stub:
                findings.append(
                    _finding(
                        finding_id=f"{slice_id}:stub:{rel}",
                        signature="INTEL-STUB-DELIVERABLE",
                        source="deliverable",
                        slice_id=slice_id,
                        queue=qrel,
                        witness=witness_rel,
                        deliverable=rel,
                        reason=stub,
                        recommended="reopen",
                    )
                )

    return findings


def _scan_signoffs(*, root: Path) -> list[dict[str, Any]]:
    path = root / SIGNOFF_REGISTRY
    if not path.is_file():
        return []
    data = json.loads(path.read_text(encoding="utf-8"))
    rows = data.get("signoffs") or []
    findings: list[dict[str, Any]] = []
    for row in rows:
        if not isinstance(row, dict):
            continue
        sid = str(row.get("id") or "")
        signoff = str(row.get("signoff") or "").upper()
        status = str(row.get("status") or "").lower()
        if signoff not in ("SIGNED", "PASS") and status not in ("done", "signed"):
            continue
        witness = str(row.get("witness") or row.get("deliverable") or "")
        if not witness:
            findings.append(
                _finding(
                    finding_id=f"signoff:{sid}:no-witness",
                    signature="INTEL-SIGNOFF-FALSE",
                    source="designer_signoff_registry",
                    slice_id=sid,
                    reason="SIGNED without witness path",
                    recommended="reopen_signoff",
                )
            )
            continue
        wpath = root / witness
        stub = _deliverable_stub_reason(wpath)
        if not wpath.is_file():
            findings.append(
                _finding(
                    finding_id=f"signoff:{sid}:missing",
                    signature="INTEL-SIGNOFF-FALSE",
                    source="designer_signoff_registry",
                    slice_id=sid,
                    witness=witness,
                    reason="SIGNED but witness file missing",
                    recommended="reopen_signoff",
                )
            )
        elif stub:
            findings.append(
                _finding(
                    finding_id=f"signoff:{sid}:stub",
                    signature="INTEL-SIGNOFF-FALSE",
                    source="designer_signoff_registry",
                    slice_id=sid,
                    witness=witness,
                    reason=stub,
                    recommended="reopen_signoff",
                )
            )
    return findings


def _scan_witness_greens(*, root: Path, max_files: int = 120) -> list[dict[str, Any]]:
    catalog = load_witness_integrity_catalog(repo=root)
    findings: list[dict[str, Any]] = []
    scanned = 0
    base = root / "debug_runs"
    if not base.is_dir():
        return findings

    for path in sorted(base.rglob("*_live.json")):
        if scanned >= max_files:
            break
        scanned += 1
        rel = _rel(path, root)
        try:
            data = json.loads(path.read_text(encoding="utf-8"))
        except (OSError, json.JSONDecodeError):
            continue
        if data.get("green") is not True:
            continue

        zero_reason = _green_zero_count_reason(data)
        if zero_reason:
            findings.append(
                _finding(
                    finding_id=f"witness:{rel}:zero",
                    signature="INTEL-GREEN-ZERO-COUNT",
                    source="witness",
                    slice_id=rel,
                    witness=rel,
                    reason=zero_reason,
                    recommended="demote_witness",
                )
            )

        report = validate_witness_honesty(
            data, witness_rel=rel, catalog=catalog, root=root, compression_level=4
        )
        if report.status == "failed":
            findings.append(
                _finding(
                    finding_id=f"witness:{rel}:dishonest",
                    signature="INTEL-WITNESS-DISHONEST",
                    source="witness",
                    slice_id=rel,
                    witness=rel,
                    reason=report.summary[:200],
                    recommended="demote_witness",
                )
            )
    return findings


def intel_officer_sweep(
    *,
    queue_filter: str = "",
    include_witness_scan: bool = True,
    compression_level: int = 3,
) -> dict[str, Any]:
    """Surveillance sweep — false-positive done/green candidates (report-only)."""
    root = repo_root()
    qfilter = queue_filter.strip() or None
    integrity = collect_queue_integrity(repo=root, queue_filter=qfilter)

    findings: list[dict[str, Any]] = []
    for stale in integrity.get("stale_ids") or []:
        sid = str(stale.get("id") or "")
        findings.append(
            _finding(
                finding_id=f"queue:{sid}:{stale.get('reason')}",
                signature="INTEL-QUEUE-STALE",
                source="queue_integrity",
                slice_id=sid,
                queue=str(stale.get("queue") or ""),
                reason=str(stale.get("reason") or "stale"),
                recommended="reopen",
            )
        )

    findings.extend(_scan_done_rows(root=root))
    findings.extend(_scan_signoffs(root=root))
    if include_witness_scan:
        findings.extend(_scan_witness_greens(root=root))

    by_sig: dict[str, int] = {}
    errors = 0
    for f in findings:
        by_sig[f["signature"]] = by_sig.get(f["signature"], 0) + 1
        if f.get("severity") == "error":
            errors += 1

    # dedupe by finding id
    deduped: dict[str, dict[str, Any]] = {}
    for f in findings:
        deduped[str(f["id"])] = f
    findings = list(deduped.values())
    findings.sort(key=lambda x: (0 if x.get("severity") == "error" else 1, x.get("slice_id", "")))

    cap = {1: 500, 2: 24, 3: 12, 4: 0}.get(max(1, min(4, compression_level)), 12)
    visible = findings if cap == 0 else findings[:cap]

    green = errors == 0
    return {
        "schema": "intel_officer_sweep_v1",
        "ok": True,
        "green": green,
        "verdict": "PASS" if green else "FAIL",
        "finding_count": len(findings),
        "error_count": errors,
        "by_signature": by_sig,
        "cull_candidates": visible,
        "cull_candidates_truncated": len(findings) > len(visible),
        "queue_integrity_green": integrity.get("green"),
        "stale_count": len(integrity.get("stale_ids") or []),
        "session_loop": [
            "intel_officer_sweep() → review cull_candidates",
            "intel_officer_apply(ids, dry_run=true) → preview reopen",
            "intel_officer_apply(ids, dry_run=false) → supervised Q✓ rollback",
            "Re-run sweep until green:true",
        ],
        "commands": [
            "python -m rust_engine_mcp.cli intel-officer-sweep",
            "python -m rust_engine_mcp.cli intel-officer-apply --ids ID1,ID2 --dry-run",
            "python -m rust_engine_mcp.cli validate-report queue_integrity --compress 3",
        ],
    }


QUEUE_ROW_KEYS = ("drain", "tasks", "p2_tasks", "multi_parallel_ready", "active")


def _utc_now_iso() -> str:
    return time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime())


def _strip_synthetic_done_ids(raw: dict[str, Any], slice_id: str) -> bool:
    changed = False
    for bucket_key in ("coder_mcp_drain",):
        block = raw.get(bucket_key)
        if not isinstance(block, dict):
            continue
        for list_key in ("done_coder_mcp", "done_designer_mcp", "done_planner_mcp"):
            ids = block.get(list_key)
            if isinstance(ids, list) and slice_id in ids:
                block[list_key] = [x for x in ids if str(x) != slice_id]
                changed = True
    return changed


def _update_rows_in_doc(raw: dict[str, Any] | list[Any], slice_id: str, status: str, note: str) -> int:
    if isinstance(raw, list):
        changed = 0
        for row in raw:
            if not isinstance(row, dict) or str(row.get("id") or "") != slice_id:
                continue
            row["status"] = status
            if note:
                row["note"] = note
            row["updated_at"] = _utc_now_iso()
            if status in ("reopened", "blocked", "open") and row.get("signoff") == "SIGNED":
                row["signoff"] = "PENDING"
            changed += 1
        return changed
    changed = 0
    for key in QUEUE_ROW_KEYS:
        rows = raw.get(key)
        if not isinstance(rows, list):
            continue
        for row in rows:
            if not isinstance(row, dict) or str(row.get("id") or "") != slice_id:
                continue
            row["status"] = status
            if note:
                row["note"] = note
            row["updated_at"] = _utc_now_iso()
            if status in ("reopened", "blocked", "open") and row.get("signoff") == "SIGNED":
                row["signoff"] = "PENDING"
            changed += 1
    return changed


def update_slice_all_queue_files(
    slice_id: str,
    status: str,
    *,
    note: str = "",
    root: Path | None = None,
) -> list[dict[str, Any]]:
    """Update every queue JSON row with matching id (dual Q✓ across home queues)."""
    root = root or repo_root()
    needle = slice_id.strip()
    results: list[dict[str, Any]] = []
    queues_dir = root / "tools" / "orchestrator" / "queues"
    for path in sorted(queues_dir.glob("*.json")):
        if path.name.startswith("_"):
            continue
        try:
            raw = json.loads(path.read_text(encoding="utf-8"))
        except (OSError, json.JSONDecodeError):
            continue
        if isinstance(raw, list):
            n = _update_rows_in_doc(raw, needle, status, note)
            if n:
                path.write_text(json.dumps(raw, indent=2) + "\n", encoding="utf-8")
                rel = str(path.relative_to(root)).replace("\\", "/")
                results.append({"path": rel, "rows_updated": n})
            continue
        n = _update_rows_in_doc(raw, needle, status, note)
        if _strip_synthetic_done_ids(raw, needle):
            n += 1
        if n:
            path.write_text(json.dumps(raw, indent=2) + "\n", encoding="utf-8")
            rel = str(path.relative_to(root)).replace("\\", "/")
            results.append({"path": rel, "rows_updated": n})
    if results:
        agent_queue._sync_dispatch_row(needle, status, note)
    return results


def _demote_witness(witness_rel: str, *, note: str, root: Path) -> dict[str, Any] | None:
    if not witness_rel.endswith(".json"):
        return None
    path = root / witness_rel
    if not path.is_file():
        return None
    data = json.loads(path.read_text(encoding="utf-8"))
    if data.get("green") is not True:
        return {"witness": witness_rel, "skipped": "not_green"}
    data["green"] = False
    data["verdict"] = "FAIL"
    data["intel_officer_note"] = note
    data["intel_officer_demoted_at"] = int(time.time())
    path.write_text(json.dumps(data, indent=2) + "\n", encoding="utf-8")
    return {"witness": witness_rel, "demoted": True}


def _reopen_signoff(slice_id: str, *, note: str, root: Path) -> dict[str, Any]:
    path = root / SIGNOFF_REGISTRY
    raw = json.loads(path.read_text(encoding="utf-8"))
    rows = raw.get("signoffs") or []
    updated = 0
    for row in rows:
        if str(row.get("id")) != slice_id:
            continue
        row["status"] = "open"
        row["signoff"] = "PENDING"
        row["intel_officer_note"] = note
        row["reopened_at"] = time.strftime("%Y-%m-%d", time.gmtime())
        if "verdict" in row:
            row["verdict"] = "FAIL"
        updated += 1
    if updated:
        path.write_text(json.dumps(raw, indent=2) + "\n", encoding="utf-8")
        return {
            "ok": True,
            "slice_id": slice_id,
            "registry": SIGNOFF_REGISTRY,
            "rows_updated": updated,
        }
    return {"ok": False, "slice_id": slice_id, "error": "signoff row not found"}


def intel_officer_apply(
    *,
    ids: list[str],
    dry_run: bool = True,
    action: str = "reopen",
    note: str = "",
    sweep: dict[str, Any] | None = None,
) -> dict[str, Any]:
    """Apply supervised cull — reopen queue rows / demote witnesses / reopen signoffs."""
    root = repo_root()
    body = sweep or intel_officer_sweep(compression_level=4)
    candidates_by_slice: dict[str, list[dict[str, Any]]] = {}
    for c in body.get("cull_candidates") or []:
        candidates_by_slice.setdefault(str(c["slice_id"]), []).append(c)
    # full list when compression hid some
    if body.get("cull_candidates_truncated"):
        full = intel_officer_sweep(compression_level=1)
        for c in full.get("cull_candidates") or []:
            bucket = candidates_by_slice.setdefault(str(c["slice_id"]), [])
            if not any(str(x.get("id")) == str(c.get("id")) for x in bucket):
                bucket.append(c)

    applied: list[dict[str, Any]] = []
    skipped: list[dict[str, Any]] = []
    base_note = note.strip() or "INTEL-OFFICER sweep reopen — false positive green/done"

    def _actions_for_slice(sid: str) -> list[dict[str, Any]]:
        cands = list(candidates_by_slice.get(sid) or [])
        if not cands:
            for c in body.get("cull_candidates") or []:
                if str(c.get("finding_id", "")).startswith(sid) or str(c.get("id", "")) == sid:
                    cands.append(c)
        if not cands:
            cands = [
                {
                    "slice_id": sid,
                    "recommended_action": "reopen" if action == "reopen" else action,
                    "signature": "INTEL-MANUAL-CULL",
                    "reason": "manual intel_officer_apply",
                }
            ]
        return cands

    for sid in ids:
        sid = sid.strip()
        if not sid:
            continue
        cands = _actions_for_slice(sid)
        slice_id = str(cands[0].get("slice_id") or sid)
        recs: list[str] = []
        for cand in cands:
            rec = str(cand.get("recommended_action") or action)
            if rec not in recs:
                recs.append(rec)
        row_note = f"{base_note} · {cands[0].get('reason', '')[:120]}"

        if dry_run:
            for rec in recs:
                applied.append(
                    {
                        "dry_run": True,
                        "slice_id": slice_id,
                        "recommended": rec,
                        "signature": cands[0].get("signature"),
                        "would": rec,
                    }
                )
            continue

        for rec in recs:
            if rec == "demote_witness":
                witness = str(cands[0].get("witness") or slice_id)
                result = _demote_witness(witness, note=row_note, root=root)
                applied.append({"slice_id": slice_id, "action": "demote_witness", **(result or {})})
                continue

            if rec == "reopen_signoff":
                result = _reopen_signoff(slice_id, note=row_note, root=root)
                applied.append(result)
                continue

            st = "reopened" if rec == "reopen" else "blocked"
            if st not in agent_queue.VALID_STATUS:
                st = "blocked"
            files = update_slice_all_queue_files(slice_id, st, note=row_note, root=root)
            if files:
                applied.append({"slice_id": slice_id, "action": st, "files": files})
                continue
            try:
                out = agent_queue.agent_queue_update(slice_id, st, note=row_note, queue="auto")
                applied.append({"slice_id": slice_id, "action": st, **out})
            except (KeyError, ValueError) as exc:
                skipped.append({"slice_id": slice_id, "error": str(exc)})

    return {
        "schema": "intel_officer_apply_v1",
        "ok": True,
        "dry_run": dry_run,
        "requested": ids,
        "applied": applied,
        "skipped": skipped,
        "hint": "Re-run intel_officer_sweep until green:true after apply",
    }


def refresh_intel_officer_sweep_witness(*, repo: Path | None = None) -> dict[str, Any]:
    root = repo or repo_root()
    body = intel_officer_sweep(compression_level=3)
    witness_body = {
        **body,
        "gate": "INTEL-OFFICER-001",
        "written_at_epoch_secs": int(time.time()),
        "_agent_meta": {
            "schema": "intel_officer_sweep_live_v1",
            "written_at_epoch_secs": int(time.time()),
            "profile": "INTEL_OFFICER",
            "source_system": "intel_officer",
            "relative_path": WITNESS_REL,
            "ritual": "BLANG:WIT-HON intel_officer_sweep" if body.get("green") else None,
        },
    }
    out = root / WITNESS_REL
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(witness_body, indent=2) + "\n", encoding="utf-8")
    witness_body["written"] = WITNESS_REL
    return witness_body
