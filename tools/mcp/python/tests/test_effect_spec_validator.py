"""VSS-T4-003 — effect_spec_v1 validate_report."""

from __future__ import annotations

from pathlib import Path

from rust_engine_mcp.paths import repo_root
from rust_engine_mcp.validators import run_validator
from rust_engine_mcp.validators.effect_spec import validate_effect_spec

SPARK_SHOWER = (
    repo_root() / "tools/mcp/schemas/examples/effect_spec_spark_shower_v1.json"
)


def test_spark_shower_example_passes() -> None:
    report = validate_effect_spec(SPARK_SHOWER, compression_level=4)
    assert report.status == "passed", report.to_dict()
    assert report.error_count == 0


def test_run_validator_effect_spec_spark_shower() -> None:
    rel = "tools/mcp/schemas/examples/effect_spec_spark_shower_v1.json"
    report = run_validator("effect_spec", rel, compression_level=3)
    assert report.status == "passed"
    assert report.validator == "effect_spec"


def test_spawn_hook_mismatch_fails(tmp_path: Path) -> None:
    import json

    bad = json.loads(SPARK_SHOWER.read_text(encoding="utf-8"))
    bad["spawn_hook"] = "attach_entity"
    path = tmp_path / "bad_hook.json"
    path.write_text(json.dumps(bad), encoding="utf-8")
    report = validate_effect_spec(path)
    assert report.status == "failed"
    assert report.error_count >= 1
