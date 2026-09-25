"""One-shot snapshot for the ops image-conclusion review surface.

Reads, when present:
  debug_runs/scan_debug_conclusion_live.json
  debug_runs/**/pixel_matrix_*.json
  matrix paths named by that conclusion

Writes tools/orchestrator/dashboard/image_conclusion_review.json.
No server. No heartbeat. Same inputs → same JSON (sort_keys, no wall clock).
"""

from __future__ import annotations

import json
import re
import sys
from pathlib import Path
from typing import Any

SCHEMA = "image_conclusion_review_v1"
CONCLUSION_REL = "debug_runs/scan_debug_conclusion_live.json"
MATRIX_GLOB = "debug_runs/**/pixel_matrix_*.json"
KINDS = ("gui", "ui", "world", "art")
LIST_KEYS = (
    "items",
    "entries",
    "rows",
    "conclusions",
    "matrices",
    "results",
    "scans",
    "matrix_paths",
    "paths",
)
HONEST_STATUS = frozenset({"honest", "ok", "passed", "pass", "green", "true"})
ABSENT_STATUS = frozenset({"absent", "missing", "none", "empty", "not_found", "null"})
PIXEL_NOTE = (
    "Python pixel pipeline: land matrices under debug_runs/ as pixel_matrix_*.json "
    "(kinds gui, ui, world, art). Also follow any matrix path listed in "
    "debug_runs/scan_debug_conclusion_live.json."
)


def repo_root() -> Path:
    return Path(__file__).resolve().parents[3]


def posix_rel(root: Path, path: Path) -> str | None:
    try:
        resolved = path.resolve()
        rel = resolved.relative_to(root.resolve())
    except (OSError, ValueError):
        return None
    return rel.as_posix()


_KIND_RE = {
    kind: re.compile(rf"(^|[_\-./]){kind}([_\-./]|$)")
    for kind in KINDS
}


def infer_kind(*texts: str) -> str:
    blob = " ".join(t for t in texts if t).lower().replace("\\", "/")
    for kind in KINDS:
        if _KIND_RE[kind].search(blob):
            return kind
    return "other"


def honesty_of(status: str, honest_flag: Any, absent_flag: Any, path_exists: bool | None) -> str:
    # A missing matrix file is absent even when the conclusion claims honest.
    if path_exists is False or absent_flag is True or status in ABSENT_STATUS:
        return "absent"
    if honest_flag is True or status in HONEST_STATUS:
        return "honest"
    return "unmarked"


def _status_text(value: Any) -> str:
    if isinstance(value, bool):
        return "honest" if value else "absent"
    if value is None:
        return ""
    return str(value).strip().lower()


def record_from_mapping(node: dict[str, Any]) -> dict[str, Any] | None:
    path = ""
    for key in ("matrix_path", "matrix_rel", "matrix", "path", "rel", "file", "witness"):
        raw = node.get(key)
        if isinstance(raw, str) and raw.strip():
            path = raw.strip().replace("\\", "/")
            break
    kind_raw = ""
    for key in ("kind", "lane", "surface", "domain", "category"):
        raw = node.get(key)
        if isinstance(raw, str) and raw.strip():
            kind_raw = raw.strip().lower()
            break
    status = ""
    for key in ("status", "conclusion_status", "result"):
        if key in node:
            status = _status_text(node.get(key))
            if status:
                break
    if not path and not kind_raw and not status and "honest" not in node:
        return None
    kind = kind_raw if kind_raw in KINDS or kind_raw == "other" else infer_kind(kind_raw, path)
    return {
        "kind": kind,
        "status": status,
        "path": path,
        "honest_flag": node.get("honest") if isinstance(node.get("honest"), bool) else None,
        "absent_flag": node.get("absent") if isinstance(node.get("absent"), bool) else None,
    }


def iter_records(node: Any, depth: int = 0) -> list[dict[str, Any]]:
    if depth > 6:
        return []
    found: list[dict[str, Any]] = []
    if isinstance(node, dict):
        for key in LIST_KEYS:
            child = node.get(key)
            if isinstance(child, list):
                for item in child:
                    if isinstance(item, str) and item.strip():
                        found.append(
                            {
                                "kind": infer_kind(item),
                                "status": "",
                                "path": item.strip().replace("\\", "/"),
                                "honest_flag": None,
                                "absent_flag": None,
                            }
                        )
                    else:
                        found.extend(iter_records(item, depth + 1))
            elif isinstance(child, dict):
                found.extend(iter_records(child, depth + 1))
        for kind in KINDS:
            child = node.get(kind)
            if isinstance(child, dict):
                rec = record_from_mapping(child) or {
                    "kind": kind,
                    "status": "",
                    "path": "",
                    "honest_flag": None,
                    "absent_flag": None,
                }
                rec["kind"] = kind
                found.append(rec)
            elif isinstance(child, str) and child.strip():
                found.append(
                    {
                        "kind": kind,
                        "status": "",
                        "path": child.strip().replace("\\", "/"),
                        "honest_flag": None,
                        "absent_flag": None,
                    }
                )
        rec = record_from_mapping(node)
        if rec and (rec["path"] or rec["kind"] in KINDS or rec["status"]):
            # Skip the conclusion root when it only carries overall status.
            if depth > 0:
                found.append(rec)
    elif isinstance(node, list):
        for item in node:
            found.extend(iter_records(item, depth + 1))
    return found


def conclusion_status(data: dict[str, Any] | None, present: bool) -> str:
    if not present or data is None:
        return "absent"
    for key in ("status", "conclusion_status", "overall_status", "result"):
        if key in data:
            text = _status_text(data.get(key))
            if text:
                return text
    if data.get("green") is True:
        return "green"
    if data.get("honest") is True:
        return "honest"
    return "present"


def load_json(path: Path) -> tuple[Any | None, str | None]:
    try:
        return json.loads(path.read_text(encoding="utf-8-sig")), None
    except (OSError, json.JSONDecodeError) as exc:
        return None, str(exc)


def safe_repo_path(root: Path, raw: str) -> Path | None:
    text = raw.strip().replace("\\", "/")
    if not text or "://" in text:
        return None
    candidate = Path(text)
    if candidate.is_absolute():
        rel = posix_rel(root, candidate)
        if rel is None:
            return None
        return root / rel
    if text.startswith("/") or ".." in Path(text).parts:
        return None
    return root / text


def row_for(
    *,
    kind: str,
    status: str,
    path: str,
    honest_flag: Any,
    absent_flag: Any,
    path_exists: bool | None,
    source: str,
) -> dict[str, str]:
    honesty = honesty_of(status, honest_flag, absent_flag, path_exists)
    shown = status or ("absent" if honesty == "absent" else "present" if path_exists else "absent")
    return {
        "honesty": honesty,
        "kind": kind if kind in KINDS else "other",
        "path": path,
        "source": source,
        "status": shown,
    }


def build_review(root: Path) -> dict[str, Any]:
    conclusion_path = root / CONCLUSION_REL
    present = conclusion_path.is_file()
    data: dict[str, Any] | None = None
    if present:
        loaded, err = load_json(conclusion_path)
        if isinstance(loaded, dict):
            data = loaded
        elif err:
            data = {"status": "unreadable"}
        else:
            data = {"status": "present"}

    by_path: dict[str, dict[str, str]] = {}
    loose: list[dict[str, str]] = []

    matrix_files = sorted(root.glob(MATRIX_GLOB), key=lambda p: p.as_posix().lower())
    for matrix in matrix_files:
        rel = posix_rel(root, matrix)
        if not rel or not rel.startswith("debug_runs/"):
            continue
        loaded, err = load_json(matrix)
        status = ""
        honest_flag = None
        absent_flag = None
        kind_hint = ""
        if isinstance(loaded, dict):
            status = _status_text(loaded.get("status"))
            if isinstance(loaded.get("honest"), bool):
                honest_flag = loaded["honest"]
            if isinstance(loaded.get("absent"), bool):
                absent_flag = loaded["absent"]
            if isinstance(loaded.get("kind"), str):
                kind_hint = loaded["kind"]
        elif err:
            status = "unreadable"
        by_path[rel] = row_for(
            kind=infer_kind(kind_hint, rel),
            status=status,
            path=rel,
            honest_flag=honest_flag,
            absent_flag=absent_flag,
            path_exists=True,
            source="glob",
        )

    if data is not None and data.get("status") != "unreadable":
        for rec in iter_records(data, 0):
            raw_path = rec.get("path") or ""
            resolved = safe_repo_path(root, raw_path) if raw_path else None
            rel = posix_rel(root, resolved) if resolved is not None else ""
            if raw_path and not rel:
                rel = ""
            exists = resolved.is_file() if resolved is not None else (False if raw_path else None)
            built = row_for(
                kind=rec.get("kind") or infer_kind(rel),
                status=rec.get("status") or "",
                path=rel or raw_path.replace("\\", "/"),
                honest_flag=rec.get("honest_flag"),
                absent_flag=rec.get("absent_flag"),
                path_exists=exists,
                source="conclusion",
            )
            key = built["path"]
            if key:
                by_path[key] = built
            else:
                loose.append(built)

    rows = sorted(by_path.values(), key=lambda r: (r["kind"], r["path"], r["status"]))
    rows.extend(sorted(loose, key=lambda r: (r["kind"], r["status"])))

    counts = {kind: 0 for kind in KINDS}
    honest = 0
    absent = 0
    for row in rows:
        if row["kind"] in counts:
            counts[row["kind"]] += 1
        if row["honesty"] == "honest":
            honest += 1
        elif row["honesty"] == "absent":
            absent += 1

    return {
        "schema": SCHEMA,
        "generated_at": None,
        "source": {
            "conclusion_present": present,
            "conclusion_rel": CONCLUSION_REL,
            "matrix_glob": MATRIX_GLOB,
            "pixel_pipeline_note": PIXEL_NOTE,
            "refresh": "python tools/orchestrator/dashboard/build_image_conclusion_review.py",
        },
        "kpi": {
            "absent": absent,
            "count_art": counts["art"],
            "count_gui": counts["gui"],
            "count_ui": counts["ui"],
            "count_world": counts["world"],
            "honest": honest,
            "last_conclusion_status": conclusion_status(data, present),
        },
        "rows": rows,
    }


def write_review(root: Path, dest: Path) -> dict[str, Any]:
    review = build_review(root)
    dest.write_text(json.dumps(review, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    return review


def _self_test() -> None:
    import tempfile

    with tempfile.TemporaryDirectory() as tmp:
        root = Path(tmp)
        review = build_review(root)
        assert review["kpi"]["last_conclusion_status"] == "absent"
        assert review["kpi"]["honest"] == 0
        assert review["kpi"]["absent"] == 0
        assert review["rows"] == []

        (root / "debug_runs").mkdir()
        (root / "debug_runs" / "scan_debug_conclusion_live.json").write_text(
            json.dumps(
                {
                    "status": "failed",
                    "gui": {"status": "honest", "matrix": "debug_runs/pixel_matrix_gui.json"},
                    "items": [
                        {"kind": "ui", "status": "absent", "path": "debug_runs/pixel_matrix_ui.json"},
                        {"kind": "art", "honest": True, "matrix_path": "debug_runs/matrices/pixel_matrix_art.json"},
                    ],
                }
            ),
            encoding="utf-8",
        )
        (root / "debug_runs" / "pixel_matrix_gui.json").write_text(
            json.dumps({"kind": "gui", "honest": True}), encoding="utf-8"
        )
        (root / "debug_runs" / "nested").mkdir()
        (root / "debug_runs" / "nested" / "pixel_matrix_world.json").write_text(
            json.dumps({"status": "present"}), encoding="utf-8"
        )
        review = build_review(root)
        assert review["kpi"]["last_conclusion_status"] == "failed"
        assert review["kpi"]["count_gui"] == 1
        assert review["kpi"]["count_ui"] == 1
        assert review["kpi"]["count_world"] == 1
        assert review["kpi"]["count_art"] == 1
        assert review["kpi"]["honest"] == 1
        assert review["kpi"]["absent"] == 2
        by_kind = {row["kind"]: row for row in review["rows"]}
        assert by_kind["world"]["honesty"] == "unmarked"
        assert by_kind["ui"]["honesty"] == "absent"
        assert by_kind["art"]["honesty"] == "absent"
        assert by_kind["gui"]["honesty"] == "honest"
        again = build_review(root)
        assert again == review


def main(argv: list[str]) -> int:
    if "--self-test" in argv:
        _self_test()
        print("self-test ok")
        return 0
    root = repo_root()
    dest = Path(__file__).resolve().parent / "image_conclusion_review.json"
    review = write_review(root, dest)
    kpi = review["kpi"]
    print(
        f"wrote {dest.relative_to(root).as_posix()} "
        f"status={kpi['last_conclusion_status']} honest={kpi['honest']} absent={kpi['absent']}"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
