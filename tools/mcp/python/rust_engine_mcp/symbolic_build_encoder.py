"""Symbolic build digest — cargo JSON → SYMLANG symcodes + trip hits for agents.

Human-improvable: optional notes at debug_runs/symbolic_digests/{digest_id}.notes.md
"""

from __future__ import annotations

import hashlib
import json
import re
from dataclasses import asdict, dataclass, field
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Literal

from rust_engine_mcp.paths import repo_root

from .validators.cargo import validate_cargo
from .validators.report import ValidationIssue, ValidationReport

DigestStatus = Literal["passed", "failed", "warning"]
DigestSource = Literal["cargo_check", "cargo_test", "validate_cargo_cached"]


@dataclass
class SymbolicBuildDigest:
    schema: str = "symbolic_build_digest_v1"
    digest_id: str = ""
    source: DigestSource = "cargo_check"
    status: DigestStatus = "passed"
    compression_level: int = 3
    summary: str = ""
    symcodes: list[str] = field(default_factory=list)
    trip_hits: list[str] = field(default_factory=list)
    touched_files: list[str] = field(default_factory=list)
    validation_report: dict[str, Any] = field(default_factory=dict)
    human_notes_path: str = ""
    program_id: str = "VSS-001"
    generated_at: str = ""

    def to_dict(self) -> dict[str, Any]:
        return asdict(self)

    def compress(self, level: int) -> SymbolicBuildDigest:
        level = max(1, min(4, level))
        cap = {1: 50, 2: 20, 3: 8, 4: 3}.get(level, 8)
        out = SymbolicBuildDigest(
            schema=self.schema,
            digest_id=self.digest_id,
            source=self.source,
            status=self.status,
            compression_level=level,
            summary=self.summary,
            symcodes=self.symcodes[:cap],
            trip_hits=self.trip_hits,
            touched_files=self.touched_files[:cap] if level < 4 else self.touched_files,
            validation_report=(
                self.validation_report
                if level <= 2
                else {
                    k: self.validation_report.get(k)
                    for k in ("validator", "status", "summary", "error_count", "warning_count")
                    if k in self.validation_report
                }
            ),
            human_notes_path=self.human_notes_path,
            program_id=self.program_id,
            generated_at=self.generated_at,
        )
        if level >= 4:
            out.symcodes = out.symcodes[:3]
            out.touched_files = out.touched_files[:5]
        return out


def _sig8(*parts: str) -> str:
    h = hashlib.sha256("|".join(parts).encode("utf-8")).hexdigest()
    return h[:8]


def issue_to_symcode(issue: ValidationIssue) -> str:
    sev = "ERR" if issue.severity == "error" else "WRN" if issue.severity == "warning" else "INF"
    path = issue.file or "-"
    line = str(issue.line or 0)
    rustc = issue.rustc_code or "-"
    sig = _sig8(path, line, issue.kind, issue.symbol, issue.hint[:120] if issue.hint else "")
    return f"{sev}◈{issue.kind}◈{path}:{line}◈{rustc}◈{sig}"


def _load_trip_registry(root: Path) -> dict[str, Any]:
    path = root / "tools/orchestrator/queues/vss_trip_artifact_registry.json"
    if not path.is_file():
        return {}
    return json.loads(path.read_text(encoding="utf-8"))


def _load_tribunal_registry(root: Path) -> dict[str, Any]:
    path = root / "tools/orchestrator/queues/vss_tribunal_touch_registry.json"
    if not path.is_file():
        return {}
    return json.loads(path.read_text(encoding="utf-8"))


def _glob_match(rel_path: str, pattern: str) -> bool:
    """Minimal glob: ** and * only."""
    rel = rel_path.replace("\\", "/")
    pat = pattern.replace("\\", "/")
    if pat.endswith("/**"):
        prefix = pat[:-3]
        return rel.startswith(prefix)
    rx = "^" + re.escape(pat).replace(r"\*\*", ".*").replace(r"\*", "[^/]*") + "$"
    return re.match(rx, rel) is not None


def trip_hits_for_files(root: Path, files: list[str]) -> list[str]:
    hits: list[str] = []
    trip_reg = _load_trip_registry(root)
    trib_reg = _load_tribunal_registry(root)
    for art in trip_reg.get("artifacts", []):
        trip_id = str(art.get("trip_id", ""))
        for trigger in art.get("triggers_on_touch", []):
            for f in files:
                if _glob_match(f, trigger) or trigger in f:
                    hits.append(trip_id)
    for touch in trib_reg.get("touches", []):
        trip_id = str(touch.get("trip_id", ""))
        for glob in touch.get("globs", []):
            for f in files:
                if _glob_match(f, glob):
                    hits.append(trip_id)
    return sorted(set(hits))


def encode_digest_from_report(
    report: ValidationReport,
    *,
    source: DigestSource = "cargo_check",
    compression_level: int = 3,
    program_id: str = "VSS-001",
    extra_touched: list[str] | None = None,
) -> SymbolicBuildDigest:
    root = repo_root()
    symcodes = [issue_to_symcode(i) for i in report.errors]
    touched = sorted(
        {i.file for i in report.errors if i.file}
        | set(extra_touched or [])
    )
    trip_hits = trip_hits_for_files(root, touched)
    digest_id = _sig8(report.summary, str(len(symcodes)), ",".join(touched[:20]))
    notes_rel = f"debug_runs/symbolic_digests/{digest_id}.notes.md"
    notes_path = root / notes_rel
    notes_path.parent.mkdir(parents=True, exist_ok=True)
    if not notes_path.is_file():
        notes_path.write_text(
            f"# Symbolic digest notes — {digest_id}\n\n"
            f"Human-editable. Preserved across encoder re-runs.\n\n"
            f"<!-- program: {program_id} -->\n",
            encoding="utf-8",
        )
    status: DigestStatus
    if report.status == "failed":
        status = "failed"
    elif report.status == "warning":
        status = "warning"
    else:
        status = "passed"
    digest = SymbolicBuildDigest(
        digest_id=digest_id,
        source=source,
        status=status,
        compression_level=compression_level,
        summary=report.summary,
        symcodes=symcodes,
        trip_hits=trip_hits,
        touched_files=touched,
        validation_report=report.compress(compression_level).to_dict(),
        human_notes_path=notes_rel.replace("\\", "/"),
        program_id=program_id,
        generated_at=datetime.now(timezone.utc).isoformat(),
    )
    return digest.compress(compression_level)


def symbolic_build_digest(
    *,
    package: str | None = None,
    compression_level: int = 3,
    use_cached: bool = False,
    extra_touched: list[str] | None = None,
    program_id: str = "VSS-001",
    write_witness: bool = True,
) -> SymbolicBuildDigest:
    report = validate_cargo(
        package=package,
        use_cached_orchestrator=use_cached,
        compression_level=1,
    )
    digest = encode_digest_from_report(
        report,
        compression_level=compression_level,
        program_id=program_id,
        extra_touched=extra_touched,
    )
    if write_witness:
        root = repo_root()
        out = root / "debug_runs/symbolic_build_encoding_live.json"
        out.parent.mkdir(parents=True, exist_ok=True)
        payload = {
            "schema": "debug_run_envelope_v1",
            "program_id": program_id,
            "body": digest.to_dict(),
        }
        out.write_text(json.dumps(payload, indent=2), encoding="utf-8")
        digest_path = root / "debug_runs/symbolic_digests" / f"{digest.digest_id}.json"
        digest_path.write_text(json.dumps(digest.to_dict(), indent=2), encoding="utf-8")
    return digest


def main() -> None:
    import argparse

    parser = argparse.ArgumentParser(description="Symbolic build digest encoder")
    parser.add_argument("--package", default="")
    parser.add_argument("--compress", type=int, default=3)
    parser.add_argument("--cached", action="store_true")
    parser.add_argument("--touched", default="", help="comma-separated extra touched files")
    args = parser.parse_args()
    extra = [t.strip() for t in args.touched.split(",") if t.strip()]
    digest = symbolic_build_digest(
        package=args.package or None,
        compression_level=args.compress,
        use_cached=args.cached,
        extra_touched=extra or None,
    )
    print(json.dumps(digest.to_dict(), indent=2))


if __name__ == "__main__":
    main()
