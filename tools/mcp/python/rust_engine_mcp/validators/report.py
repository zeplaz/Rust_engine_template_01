"""Structured ValidationReport — agents consume this, not raw logs."""

from __future__ import annotations

from dataclasses import asdict, dataclass, field
from typing import Any, Literal


ValidatorId = Literal["cargo", "bevy", "mcp_schema", "asset_glb", "test", "tile", "blender"]
Status = Literal["passed", "failed", "warning"]
Severity = Literal["error", "warning", "info"]


@dataclass
class ValidationIssue:
    kind: str
    severity: Severity = "error"
    file: str = ""
    line: int = 0
    column: int = 0
    symbol: str = ""
    field: str = ""
    hint: str = ""
    rustc_code: str = ""
    api: str = ""
    replacement: str = ""
    signature: str = ""

    def to_dict(self) -> dict[str, Any]:
        d = asdict(self)
        return {k: v for k, v in d.items() if v not in ("", 0)}


@dataclass
class KnownFix:
    signature: str
    fix: str
    confidence: float = 0.0

    def to_dict(self) -> dict[str, Any]:
        return asdict(self)


@dataclass
class ValidationReport:
    validator: ValidatorId
    status: Status
    compression_level: int = 3
    schema_version: int = 1
    summary: str = ""
    error_count: int = 0
    warning_count: int = 0
    errors: list[ValidationIssue] = field(default_factory=list)
    known_fixes: list[KnownFix] = field(default_factory=list)
    raw_log_path: str = ""
    confidence: float = 1.0

    def to_dict(self) -> dict[str, Any]:
        return {
            "schema_version": self.schema_version,
            "validator": self.validator,
            "status": self.status,
            "compression_level": self.compression_level,
            "summary": self.summary,
            "error_count": self.error_count,
            "warning_count": self.warning_count,
            "errors": [e.to_dict() for e in self.errors],
            "known_fixes": [k.to_dict() for k in self.known_fixes],
            "raw_log_path": self.raw_log_path,
            "confidence": self.confidence,
        }

    def compress(self, level: int) -> ValidationReport:
        """Level 4: summary + known_fixes only; Level 3: capped issue list."""
        level = max(1, min(4, level))
        if level >= 4 and self.known_fixes:
            return ValidationReport(
                validator=self.validator,
                status=self.status,
                compression_level=4,
                summary=self.summary,
                error_count=self.error_count,
                warning_count=self.warning_count,
                known_fixes=self.known_fixes,
                confidence=self.confidence,
            )
        if level >= 4:
            return ValidationReport(
                validator=self.validator,
                status=self.status,
                compression_level=4,
                summary=self.summary,
                error_count=self.error_count,
                warning_count=self.warning_count,
                errors=[],
                known_fixes=self.known_fixes,
                confidence=self.confidence,
            )
        cap = {1: 50, 2: 20, 3: 8, 4: 3}.get(level, 8)
        return ValidationReport(
            validator=self.validator,
            status=self.status,
            compression_level=level,
            summary=self.summary,
            error_count=self.error_count,
            warning_count=self.warning_count,
            errors=self.errors[:cap],
            known_fixes=self.known_fixes,
            raw_log_path=self.raw_log_path if level <= 2 else "",
            confidence=self.confidence,
        )
