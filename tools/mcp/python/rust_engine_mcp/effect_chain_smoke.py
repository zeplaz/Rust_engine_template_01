"""PCI-21 — one-command effect chain smoke.

Chain: EffectSpec → validate → preview-capture → honest_gate → promote.

Isolated temp tree. Does not write the live registry or staging dirs.
Does not schedule cron or a daemon. CI calls this smoke from
.github/workflows/ci.yml only after test_effect_preview_capture stays
under two minutes (PCI-27). force=True remains the operator override
for a pending production honest_gate (PCI-28); this smoke never forces.
"""

from __future__ import annotations

import shutil
import tempfile
import time
from pathlib import Path
from typing import Any

from . import effect_preview_capture as epc
from . import effect_promote as ep
from .paths import repo_root as live_repo_root
from .validators.effect_spec import validate_effect_spec

BUDGET_MS = 120_000
WITNESS_REL = "debug_runs/pci21_effect_chain_smoke_live.json"
CHAIN_STEPS = ("validate", "preview_capture", "honest_gate", "promote")


def run_effect_chain_smoke(*, write_witness: bool = False) -> dict[str, Any]:
    """Run the reference-batch chain in a throwaway tree. Exit predicate is green."""
    started = time.perf_counter()
    root = live_repo_root()
    errors: list[str] = []
    effects: list[dict[str, Any]] = []
    tmp = Path(tempfile.mkdtemp(prefix="pci21_effect_chain_"))
    staging = tmp / "assets" / "staging"
    staging.mkdir(parents=True)
    (tmp / "assets" / "effects" / "registry").mkdir(parents=True)

    binds: list[tuple[Any, str, Any]] = [
        (ep, "repo_root", lambda: tmp),
        (ep, "staging_root", lambda: staging),
        (ep, "WITNESS_REL", "debug_runs/artist_vfx_pipeline_live.json"),
        (ep, "REGISTRY_REL", "assets/effects/registry"),
        (epc, "repo_root", lambda: tmp),
        (epc, "staging_root", lambda: staging),
        (epc, "WITNESS_REL", "debug_runs/artist_vfx_pipeline_live.json"),
        (epc, "REGISTRY_REL", "assets/effects/registry"),
    ]
    saved: list[tuple[Any, str, Any]] = []
    try:
        for module, name, value in binds:
            saved.append((module, name, getattr(module, name)))
            setattr(module, name, value)
        _seed_isolated_tree(root, tmp)
        for rel in ep.REFERENCE_SPEC_RELS:
            effects.append(_run_one(tmp / rel, rel, errors))
    except Exception as exc:  # noqa: BLE001 — smoke returns a failed body, not a traceback wall
        errors.append(str(exc)[:240])
    finally:
        for module, name, old in reversed(saved):
            setattr(module, name, old)
        shutil.rmtree(tmp, ignore_errors=True)

    elapsed_ms = int((time.perf_counter() - started) * 1000)
    honest = bool(effects) and all(
        row.get("validate_status") == "passed"
        and row.get("honest_gate") == "honest"
        and row.get("promoted_honest_gate") == "honest"
        for row in effects
    )
    under_budget = elapsed_ms <= BUDGET_MS
    body: dict[str, Any] = {
        "schema": "effect_chain_smoke_v1",
        "id": "PCI-21",
        "command": "effect-chain-smoke",
        "batch_id": ep.REFERENCE_BATCH_ID,
        "steps": list(CHAIN_STEPS),
        "count": len(effects),
        "effects": effects,
        "errors": errors,
        "elapsed_ms": elapsed_ms,
        "budget_ms": BUDGET_MS,
        "under_budget": under_budget,
        "cron_scheduled": False,
        "writes_live_registry": False,
        "force": False,
        "green": honest and under_budget and not errors and len(effects) == len(ep.REFERENCE_SPEC_RELS),
    }
    if write_witness:
        body["witness"] = WITNESS_REL
        path = root / WITNESS_REL
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(_dump(body), encoding="utf-8")
    return body


def _seed_isolated_tree(root: Path, tmp: Path) -> None:
    for rel in ep.REFERENCE_SPEC_RELS:
        src = root / rel
        if not src.is_file():
            raise FileNotFoundError(f"reference EffectSpec missing: {rel}")
        dest = tmp / rel
        dest.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(src, dest)
        spec = ep.load_json_file(src)
        for _role, shader_rel in ep._shader_pack_entries(spec):
            shader_src = root / shader_rel
            if not shader_src.is_file():
                raise FileNotFoundError(f"shader_pack missing: {shader_rel}")
            shader_dest = tmp / shader_rel
            shader_dest.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(shader_src, shader_dest)


def _run_one(spec_path: Path, rel: str, errors: list[str]) -> dict[str, Any]:
    row: dict[str, Any] = {"spec": rel.replace("\\", "/")}
    report = validate_effect_spec(spec_path, compression_level=4)
    row["validate_status"] = report.status
    if report.status == "failed":
        errors.append(f"validate failed: {rel}")
        return row
    captured = epc.capture_effect_preview(
        rel, pack_first=True, force=False, write_registry=False
    )
    row["effect_id"] = captured.get("effect_id")
    row["honest_gate"] = captured.get("honest_gate")
    row["frames_captured"] = captured.get("frames_captured")
    if captured.get("honest_gate") != "honest":
        errors.append(f"honest_gate not honest: {rel}")
        return row
    promoted = ep.promote_effect(rel, force=False, pack_first=False)
    gate = (promoted.get("preview_witness") or {}).get("honest_gate")
    row["promoted_honest_gate"] = gate
    if gate != "honest":
        errors.append(f"promote lost honest_gate: {rel}")
    return row


def _dump(body: dict[str, Any]) -> str:
    import json

    return json.dumps(body, indent=2) + "\n"
