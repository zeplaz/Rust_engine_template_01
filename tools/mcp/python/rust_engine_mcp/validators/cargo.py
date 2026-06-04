"""Cargo check validator — parses --message-format=json only (never stderr scraping)."""

from __future__ import annotations

import json
import subprocess
from datetime import datetime, timezone
from pathlib import Path

from rust_engine_mcp.paths import repo_root

from .knowledge import lookup_fixes
from .report import ValidationIssue, ValidationReport

RUSTC_KIND: dict[str, str] = {
    "E0308": "TypeMismatch",
    "E0432": "MissingImport",
    "E0433": "MissingImport",
    "E0609": "MissingField",
    "E0599": "MissingMethod",
    "E0277": "TraitNotImplemented",
    "E0382": "BorrowIssue",
    "E0597": "LifetimeIssue",
    "E0425": "MissingImport",
    "E0412": "MissingImport",
}


def _classify_rustc(code: str, message: str) -> str:
    if code in RUSTC_KIND:
        return RUSTC_KIND[code]
    msg = message.lower()
    if "cannot borrow" in msg:
        return "BorrowIssue"
    if "lifetime" in msg:
        return "LifetimeIssue"
    if "trait" in msg and "not satisfied" in msg:
        return "TraitNotImplemented"
    if "no field" in msg or "no method named" in msg:
        return "MissingField"
    if "mismatch" in msg:
        return "TypeMismatch"
    return "BuildFailure"


def _parse_cargo_json(stdout: str, root: Path) -> list[ValidationIssue]:
    issues: list[ValidationIssue] = []
    seen: set[str] = set()
    for line in stdout.splitlines():
        line = line.strip()
        if not line.startswith("{"):
            continue
        try:
            obj = json.loads(line)
        except json.JSONDecodeError:
            continue
        if obj.get("reason") != "compiler-message":
            continue
        msg = obj.get("message") or {}
        level = str(msg.get("level") or "")
        if level not in ("error", "warning"):
            continue
        spans = msg.get("spans") or []
        span = spans[0] if spans else {}
        file_name = str(span.get("file_name") or "")
        try:
            rel = str(Path(file_name).resolve().relative_to(root.resolve())).replace("\\", "/")
        except ValueError:
            rel = file_name.replace("\\", "/")
        code_obj = msg.get("code") or {}
        rustc_code = str(code_obj.get("code") or "") if isinstance(code_obj, dict) else ""
        message = str(msg.get("message") or "")
        key = f"{rel}:{span.get('line_start')}:{rustc_code}:{message[:80]}"
        if key in seen:
            continue
        seen.add(key)
        kind = _classify_rustc(rustc_code, message)
        issues.append(
            ValidationIssue(
                kind=kind,
                severity="error" if level == "error" else "warning",
                file=rel,
                line=int(span.get("line_start") or 0),
                column=int(span.get("column_start") or 0),
                symbol=_extract_symbol(message),
                hint=message[:240],
                rustc_code=rustc_code,
                signature=f"{rustc_code}_{kind}" if rustc_code else kind,
            )
        )
    return issues


def _extract_symbol(message: str) -> str:
    for token in ("`",):
        if token in message:
            parts = message.split("`")
            if len(parts) >= 2:
                return parts[1][:64]
    return ""


def _write_raw_log(stdout: str, stderr: str) -> str:
    log_dir = repo_root() / "debug_runs" / "validators"
    log_dir.mkdir(parents=True, exist_ok=True)
    stamp = datetime.now(timezone.utc).strftime("%Y%m%d_%H%M%S")
    path = log_dir / f"cargo_{stamp}.log"
    path.write_text(f"--- stdout ---\n{stdout}\n--- stderr ---\n{stderr}\n", encoding="utf-8")
    return str(path.relative_to(repo_root())).replace("\\", "/")


def validate_cargo(
    *,
    package: str | None = None,
    use_cached_orchestrator: bool = False,
    compression_level: int = 3,
) -> ValidationReport:
    root = repo_root()
    if use_cached_orchestrator:
        cached = root / "tools" / "orchestrator" / "state" / "last_run.json"
        if cached.is_file():
            return _from_orchestrator_cache(cached, compression_level)

    cmd = ["cargo", "check", "--message-format=json"]
    if package:
        cmd.extend(["-p", package])
    proc = subprocess.run(
        cmd,
        cwd=str(root),
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    issues = _parse_cargo_json(proc.stdout, root)
    errors = [i for i in issues if i.severity == "error"]
    warnings = [i for i in issues if i.severity == "warning"]
    status = "passed" if proc.returncode == 0 and not errors else "failed"
    if status == "passed" and warnings:
        status = "warning"
    known = lookup_fixes(errors + warnings[:5])
    report = ValidationReport(
        validator="cargo",
        status=status,  # type: ignore[arg-type]
        compression_level=compression_level,
        summary=f"cargo check: {len(errors)} errors, {len(warnings)} warnings",
        error_count=len(errors),
        warning_count=len(warnings),
        errors=errors,
        known_fixes=known,
        raw_log_path=_write_raw_log(proc.stdout, proc.stderr),
        confidence=0.95 if proc.stdout else 0.5,
    )
    return report.compress(compression_level)


def _from_orchestrator_cache(path: Path, compression_level: int) -> ValidationReport:
    data = json.loads(path.read_text(encoding="utf-8"))
    issues: list[ValidationIssue] = []
    for row in data.get("issues") or []:
        sev = str(row.get("severity") or "").lower()
        issues.append(
            ValidationIssue(
                kind=_classify_rustc(str(row.get("rustc_code") or ""), str(row.get("message") or "")),
                severity="error" if sev == "fatal" else ("warning" if sev == "warning" else "info"),
                file=str(row.get("file") or ""),
                line=int(row.get("line") or 0),
                symbol=str(row.get("symbol") or ""),
                hint=str(row.get("message") or "")[:240],
                rustc_code=str(row.get("rustc_code") or ""),
                signature=str(row.get("id") or ""),
            )
        )
    errors = [i for i in issues if i.severity == "error"]
    warnings = [i for i in issues if i.severity == "warning"]
    meta = data.get("meta") or {}
    ok = bool(meta.get("check_ok", True))
    status = "passed" if ok and not errors else ("warning" if ok and warnings else "failed")
    report = ValidationReport(
        validator="cargo",
        status=status,  # type: ignore[arg-type]
        compression_level=compression_level,
        summary=f"orchestrator cache: {len(errors)} errors, {len(warnings)} warnings",
        error_count=len(errors),
        warning_count=len(warnings),
        errors=errors,
        known_fixes=lookup_fixes(errors + warnings[:5]),
        raw_log_path=str(path.relative_to(repo_root())).replace("\\", "/"),
        confidence=0.8,
    )
    return report.compress(compression_level)
