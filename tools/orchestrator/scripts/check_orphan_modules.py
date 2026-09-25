"""PCI-38 — orphan module CI check.

Registry scanned: assets/configs/buildings/_module_index.ron
(ProceduralModuleRegistry source, MODULE_INDEX_RON).

A row is an orphan when its glb path is not a file. Paths under
assets/staging/ are untracked work-in-progress and never fail this check.
The scanner reads the named registry only; it does not walk staging.
"""

from __future__ import annotations

import json
import re
import tempfile
from pathlib import Path

_REPO = Path(__file__).resolve().parents[3]

MODULE_REGISTRY = "assets/configs/buildings/_module_index.ron"
STAGING_PREFIX = "assets/staging/"

_MODULE_ID = re.compile(r'\s*"([^"]+)"')
_GLB = re.compile(r'glb:\s*"([^"]+)"')


def _norm(rel: str) -> str:
    return rel.replace("\\", "/").lstrip("./")


def is_untracked_staging(rel: str) -> bool:
    norm = _norm(rel)
    return norm == "assets/staging" or norm.startswith(STAGING_PREFIX)


def registry_rows(text: str) -> list[tuple[str, str]]:
    rows: list[tuple[str, str]] = []
    for chunk in text.split("module_id:"):
        mid = _MODULE_ID.match(chunk)
        glb = _GLB.search(chunk)
        if mid and glb:
            rows.append((mid.group(1), _norm(glb.group(1))))
    return rows


def orphan_glbs(root: Path, text: str) -> list[dict[str, str]]:
    """Missing non-staging glbs named by the registry. Staging rows are skipped."""
    missing: list[dict[str, str]] = []
    for module_id, rel in registry_rows(text):
        if is_untracked_staging(rel):
            continue
        if not (root / rel).is_file():
            missing.append({"module_id": module_id, "glb": rel})
    return missing


def _self_test() -> None:
    sample = """
(
    entries: [
        (
            module_id: "kept",
            glb: "assets/models/modules/kept/model.glb",
        ),
        (
            module_id: "staged",
            glb: "assets/staging/wip_untracked/model.glb",
        ),
        (
            module_id: "gone",
            glb: "assets/models/modules/gone/model.glb",
        ),
    ],
)
"""
    with tempfile.TemporaryDirectory() as tmp:
        root = Path(tmp)
        kept = root / "assets/models/modules/kept/model.glb"
        kept.parent.mkdir(parents=True)
        kept.write_bytes(b"glb")
        # Untracked staging folder is present and still must not fail the check.
        staged = root / "assets/staging/wip_untracked/model.glb"
        staged.parent.mkdir(parents=True)
        staged.write_bytes(b"glb")
        missing = orphan_glbs(root, sample)
    ids = [row["module_id"] for row in missing]
    if ids != ["gone"]:
        raise SystemExit(f"self-test: expected orphan 'gone', got {ids}")
    if any(is_untracked_staging(row["glb"]) for row in missing):
        raise SystemExit("self-test: untracked staging was reported as an orphan")


def main() -> int:
    _self_test()
    registry = _REPO / MODULE_REGISTRY
    if not registry.is_file():
        print(json.dumps({"gate": "PCI-38", "error": f"missing registry {MODULE_REGISTRY}"}))
        return 1
    text = registry.read_text(encoding="utf-8")
    rows = registry_rows(text)
    skipped = sum(1 for _mid, rel in rows if is_untracked_staging(rel))
    missing = orphan_glbs(_REPO, text)
    report = {
        "gate": "PCI-38",
        "self_test": "passed",
        "registry": MODULE_REGISTRY,
        "scanned": len(rows),
        "staging_rows_skipped": skipped,
        "orphan_count": len(missing),
        "orphans": missing[:20],
    }
    print(json.dumps(report, indent=2))
    if missing:
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
