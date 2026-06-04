"""variant_set_v1 — validate, patch, bake, agent request (APS-VAR / APS-AGENT)."""

from __future__ import annotations

import copy
import json
import re
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

from rust_engine_mcp import assembly
from rust_engine_mcp import paths
from rust_engine_mcp.schemas import load_json_file, validate_variant_set
from rust_engine_mcp.tile_pipeline import (
    _run_tile_job_path,
    _variant_key,
    assembly_build_run,
    light_setup_blend_path,
    tile_dry_run_enabled,
)

VARIANT_STAGING = "assets/staging/variants"
AGENT_REQUEST_DIR = "debug_runs/art_pipeline"


def variant_staging_dir() -> Path:
    return paths.repo_root() / VARIANT_STAGING


def default_variant_set_path(variant_set_id: str, *, ext: str = ".json") -> Path:
    return variant_staging_dir() / f"{variant_set_id}{ext}"


def load_variant_set(path: str | Path) -> dict[str, Any]:
    """Load variant_set_v1 from `.json` or `.ron` (limited RON → JSON conversion)."""
    p = Path(path).resolve()
    text = p.read_text(encoding="utf-8")
    if p.suffix.lower() == ".json":
        data = json.loads(text)
    else:
        data = _ron_to_dict(text)
    validate_variant_set(data)
    return data


def save_variant_set(data: dict[str, Any], path: str | Path | None = None) -> Path:
    validate_variant_set(data)
    if path is None:
        out = default_variant_set_path(str(data["variant_set_id"]))
    else:
        out = Path(path).resolve()
    out.parent.mkdir(parents=True, exist_ok=True)
    if out.suffix.lower() == ".ron":
        out.write_text(_dict_to_ron(data) + "\n", encoding="utf-8")
    else:
        out.write_text(json.dumps(data, indent=2) + "\n", encoding="utf-8")
    return out


def variant_set_validate(path: str | Path) -> dict[str, Any]:
    data = load_variant_set(path)
    return {
        "valid": True,
        "variant_set_id": data.get("variant_set_id"),
        "assembly_id": data.get("assembly_id"),
        "variant_count": len(data.get("variants") or []),
        "path": str(Path(path).resolve()),
    }


def variant_set_patch(
    path: str | Path,
    patch: list[dict[str, Any]],
    *,
    write: bool = True,
) -> dict[str, Any]:
    """Apply RFC6902-style patch ops (add, remove, replace) deterministically."""
    p = Path(path).resolve()
    doc = load_variant_set(p)
    applied = apply_json_patch(doc, patch)
    validate_variant_set(applied)
    written: str | None = None
    if write:
        save_variant_set(applied, p)
        written = str(p)
    return {"ok": True, "applied_ops": len(patch), "written_path": written, "document": applied}


def apply_json_patch(document: dict[str, Any], operations: list[dict[str, Any]]) -> dict[str, Any]:
    doc = copy.deepcopy(document)
    for op in operations:
        kind = str(op.get("op") or "")
        path = str(op.get("path") or "")
        if kind == "add":
            _patch_add(doc, path, op.get("value"))
        elif kind == "remove":
            _patch_remove(doc, path)
        elif kind == "replace":
            _patch_replace(doc, path, op.get("value"))
        elif kind == "test":
            current = _patch_get(doc, path)
            if current != op.get("value"):
                raise ValueError(f"patch test failed at {path}")
        else:
            raise ValueError(f"unsupported patch op: {kind}")
    return doc


def _parse_pointer(path: str) -> list[str]:
    if not path.startswith("/"):
        raise ValueError(f"invalid JSON pointer: {path}")
    if path == "/":
        return []
    parts = path[1:].split("/")
    return [p.replace("~1", "/").replace("~0", "~") for p in parts]


def _patch_get(doc: Any, path: str) -> Any:
    cur = doc
    for part in _parse_pointer(path):
        if isinstance(cur, list):
            cur = cur[int(part)]
        else:
            cur = cur[part]
    return cur


def _patch_set_parent(doc: Any, path: str, *, create: bool = False) -> tuple[Any, str | int]:
    parts = _parse_pointer(path)
    if not parts:
        raise ValueError("cannot set document root")
    cur = doc
    for part in parts[:-1]:
        if isinstance(cur, list):
            cur = cur[int(part)]
        else:
            if create and part not in cur:
                cur[part] = {}
            cur = cur[part]
    last = parts[-1]
    if isinstance(cur, list):
        key: str | int = "-" if last == "-" else int(last)
    else:
        key = last
    return cur, key


def _patch_add(doc: Any, path: str, value: Any) -> None:
    parent, key = _patch_set_parent(doc, path, create=True)
    if isinstance(parent, list):
        if key == "-":
            parent.append(value)
        else:
            parent.insert(int(key), value)
    else:
        parent[key] = value


def _patch_remove(doc: Any, path: str) -> None:
    parent, key = _patch_set_parent(doc, path)
    if isinstance(parent, list):
        del parent[int(key)]
    else:
        del parent[key]


def _patch_replace(doc: Any, path: str, value: Any) -> None:
    parent, key = _patch_set_parent(doc, path)
    if isinstance(parent, list):
        parent[int(key)] = value
    else:
        parent[key] = value


def layers_to_tile_variant(entry: dict[str, Any]) -> dict[str, Any]:
    """Map variant_set layers → flat tile_variant_bake variant block."""
    layers = dict(entry.get("layers") or {})
    lighting = dict(layers.get("lighting") or {})
    damage = dict(layers.get("damage") or {})
    fill = dict(layers.get("fill") or {})
    out: dict[str, Any] = {
        "variant_key": str(entry.get("variant_key") or ""),
        "state": str(damage.get("state") or "clean"),
        "damage": float(damage.get("damage") or 0.0),
        "power": str(lighting.get("power") or "off"),
        "fill": str(fill.get("fill") or "empty"),
        "lighting": str(lighting.get("lighting") or "day"),
    }
    material = layers.get("material")
    if material:
        out["material"] = material
    return out


def _find_variant(data: dict[str, Any], variant_key: str) -> tuple[int, dict[str, Any]]:
    for i, entry in enumerate(data.get("variants") or []):
        if str(entry.get("variant_key")) == variant_key:
            return i, dict(entry)
    raise KeyError(f"variant_key not found: {variant_key}")


def _assembly_snapshot_path(assembly_id: str) -> Path:
    p = assembly.default_snapshot_path(assembly_id)
    if not p.is_file():
        raise FileNotFoundError(f"assembly snapshot missing: {p}")
    return p


def _ensure_assembly_blend(assembly_id: str) -> str:
    snap_path = _assembly_snapshot_path(assembly_id)
    blend_rel = f"assets/staging/assemblies/{assembly_id}.blend"
    blend_path = paths.repo_root() / blend_rel
    if not blend_path.is_file():
        result = assembly_build_run(snap_path)
        if not result.get("ok"):
            raise RuntimeError(f"assembly_build failed: {result}")
    return blend_rel


def variant_bake(
    variant_set_path: str | Path,
    variant_key: str,
    *,
    seed: int | None = None,
    write_status: bool = True,
) -> dict[str, Any]:
    """Bake one variant_key — PNG + bake.status on variant row."""
    p = Path(variant_set_path).resolve()
    data = load_variant_set(p)
    idx, entry = _find_variant(data, variant_key)
    assembly_id = str(data["assembly_id"])
    render_seed = int(seed if seed is not None else data.get("seed") or 42)

    assembly_blend = _ensure_assembly_blend(assembly_id)
    tile_variant = layers_to_tile_variant(entry)
    vkey = _variant_key(tile_variant)

    png_rel = f"assets/staging/tiles/{assembly_id}/{vkey}.png"
    job_id = f"tile_{assembly_id}_{vkey}"[:120]

    light_path = light_setup_blend_path()
    try:
        light_rel = str(light_path.relative_to(paths.repo_root())).replace("\\", "/")
    except ValueError:
        light_rel = str(light_path)

    job = {
        "schema_version": 1,
        "job_id": job_id,
        "operation": "tile_variant_bake",
        "mode": "assembly",
        "variant": tile_variant,
        "render": {
            "method": "blender_orthographic_iso",
            "isometric": True,
            "seed": render_seed,
            "tile_size_px": 128,
            "camera_elevation_deg": 35.264,
        },
        "light_blend": light_rel,
        "assembly_blend": assembly_blend,
        "output": {"png": png_rel},
    }
    job_path = paths.jobs_root() / f"{job_id}.json"
    job_path.write_text(json.dumps(job, indent=2) + "\n", encoding="utf-8")

    if write_status:
        data["variants"][idx]["bake"] = {"status": "running", "last_job_id": job_id}
        save_variant_set(data, p)

    result = _run_tile_job_path(job_path)
    ok = result.status == "done"
    bake_status = {
        "status": "done" if ok else "failed",
        "png": png_rel if ok else None,
        "last_job_id": job_id,
    }
    if write_status:
        data["variants"][idx]["bake"] = bake_status
        save_variant_set(data, p)

    return {
        "ok": ok,
        "variant_key": vkey,
        "assembly_id": assembly_id,
        "job_id": job_id,
        "png": png_rel if ok else None,
        "outputs": result.outputs,
        "error": result.error,
        "dry_run": tile_dry_run_enabled(),
        "variant_set_path": str(p),
    }


def variant_agent_request(body: dict[str, Any], *, write: bool = True) -> dict[str, Any]:
    """Stub — returns deterministic patch proposal; no LLM in repo."""
    assembly_id = str(body.get("assembly_id") or "")
    variant_key = str(body.get("variant_key") or "")
    intent = str(body.get("intent") or "").lower()
    current_layers = dict(body.get("current_layers") or {})

    patch: list[dict[str, Any]] = []
    note = "stub template — review before variant_set_patch"

    if "night" in intent and "light" in intent:
        patch.append(
            {
                "op": "replace",
                "path": "/variants/0/layers/lighting",
                "value": {
                    **dict(current_layers.get("lighting") or {}),
                    "lighting": "night_on",
                    "power": "on",
                    "night_lights": True,
                    "emissive_strength": 0.8,
                },
            }
        )
        note = "suggested warm window lights (night_on + emissive)"
    elif "damage" in intent:
        patch.append(
            {
                "op": "replace",
                "path": "/variants/0/layers/damage",
                "value": {"state": "damaged", "damage": 0.45},
            }
        )
        note = "suggested damage layer"
    elif "material" in intent or "brick" in intent:
        patch.append(
            {
                "op": "replace",
                "path": "/variants/0/layers/material",
                "value": {"wall_material": "brick_red_01"},
            }
        )
        note = "suggested material swap"
    else:
        patch.append(
            {
                "op": "add",
                "path": "/variants/0/tags/-",
                "value": f"agent_request_{intent or 'review'}",
            }
        )
        note = "tagged for agent review — refine intent string for layer patch"

    response = {
        "schema_version": 1,
        "assembly_id": assembly_id,
        "variant_key": variant_key,
        "intent": body.get("intent"),
        "constraints": body.get("constraints") or [],
        "reference_tags": body.get("reference_tags") or [],
        "patch": patch,
        "note": note,
    }

    written_path: str | None = None
    if write:
        out_dir = paths.repo_root() / AGENT_REQUEST_DIR
        out_dir.mkdir(parents=True, exist_ok=True)
        out_path = out_dir / "variant_agent_request.json"
        payload = {
            **body,
            "generated_at": datetime.now(timezone.utc).isoformat(),
            "proposal": response,
        }
        out_path.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
        written_path = str(out_path.relative_to(paths.repo_root())).replace("\\", "/")
        response["written_path"] = written_path

    return response


def expand_variant_set_to_tile_batch(
    data: dict[str, Any],
    *,
    batch_id: str | None = None,
) -> dict[str, Any]:
    """Build tile_batch_v1 JSON body from variant_set (for Bake variants / Atlas flow)."""
    assembly_id = str(data["assembly_id"])
    snap_path = assembly.default_snapshot_path(assembly_id)
    bid = batch_id or f"tile_{data['variant_set_id']}"
    variants = [layers_to_tile_variant(v) for v in data.get("variants") or []]
    return {
        "schema_version": 1,
        "batch_id": bid,
        "tile_id": data.get("variant_set_id") or assembly_id,
        "rules_applied": ["variant_set_v1", "deterministic_output"],
        "render": {
            "method": "blender_orthographic_iso",
            "isometric": True,
            "seed": int(data.get("seed") or 42),
            "tile_size_px": 128,
            "camera_elevation_deg": 35.264,
        },
        "assembly_ref": {
            "assembly_snapshot": str(snap_path.relative_to(paths.repo_root())).replace("\\", "/"),
            "style_pack_id": data.get("style_pack_id") or "style_victorian",
        },
        "variants": variants,
        "atlas": {
            "atlas_id": bid,
            "columns": 4,
            "rows": max(1, (len(variants) + 3) // 4),
            "tile_px": 128,
            "padding_px": 2,
            "output_png": f"assets/textures/tiles/{bid}_atlas.png",
            "meta_json": f"assets/staging/tiles/{bid}/atlas_meta.json",
        },
    }


def _ron_to_dict(text: str) -> dict[str, Any]:
    """Minimal RON loader — falls back to JSON5-ish rewrite for variant_set drafts."""
    stripped = text.strip()
    if stripped.startswith("{"):
        return json.loads(stripped)
    # Quote bare keys: word: → "word":
    converted = re.sub(r"(\w+):", r'"\1":', stripped)
    converted = converted.replace("(", "{").replace(")", "}")
    converted = re.sub(r",\s*}", "}", converted)
    converted = re.sub(r",\s*]", "]", converted)
    try:
        return json.loads(converted)
    except json.JSONDecodeError as exc:
        raise ValueError(
            f"RON parse failed — save as .json or fix syntax: {exc}"
        ) from exc


def _dict_to_ron(data: dict[str, Any]) -> str:
    """Pretty RON-like export (JSON-compatible subset)."""
    lines = ["("]
    lines.append(f"    schema_version: {data.get('schema_version', 1)},")
    lines.append(f'    variant_set_id: "{data.get("variant_set_id")}",')
    lines.append(f'    assembly_id: "{data.get("assembly_id")}",')
    if data.get("style_pack_id"):
        lines.append(f'    style_pack_id: "{data.get("style_pack_id")}",')
    if data.get("seed") is not None:
        lines.append(f"    seed: {int(data['seed'])},")
    lines.append("    variants: [")
    for entry in data.get("variants") or []:
        lines.append("        (")
        lines.append(f'            variant_key: "{entry.get("variant_key")}",')
        tags = entry.get("tags") or []
        if tags:
            tag_str = ", ".join(f'"{t}"' for t in tags)
            lines.append(f"            tags: [{tag_str}],")
        lines.append(f"            layers: {json.dumps(entry.get('layers') or {})},")
        lines.append("        ),")
    lines.append("    ],")
    lines.append(")")
    return "\n".join(lines)


def example_variant_set_path() -> Path:
    return paths.schemas_dir() / "examples" / "variant_set_rowhouse_victorian_v1.json"
