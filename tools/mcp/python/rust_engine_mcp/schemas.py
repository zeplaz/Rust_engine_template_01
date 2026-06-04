"""JSON schema validation for AssetSpec and GeometryJob."""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any

import jsonschema

from .paths import schemas_dir


def _load_schema(name: str) -> dict[str, Any]:
    path = schemas_dir() / name
    return json.loads(path.read_text(encoding="utf-8"))


def validate_asset_spec(data: dict[str, Any]) -> None:
    jsonschema.validate(instance=data, schema=_load_schema("asset_spec_v1.schema.json"))


def validate_geometry_job(data: dict[str, Any]) -> None:
    jsonschema.validate(instance=data, schema=_load_schema("geometry_job_v1.schema.json"))


def validate_building_grammar(data: dict[str, Any]) -> None:
    jsonschema.validate(instance=data, schema=_load_schema("building_grammar_v1.schema.json"))


def validate_assembly_snapshot(data: dict[str, Any]) -> None:
    snap = _load_schema("assembly_snapshot_v1.schema.json")
    node = _load_schema("assembly_graph_node_v1.schema.json")
    tags = _load_schema("aps_tag_taxonomy_v1.schema.json")
    base = schemas_dir().as_uri() + "/"
    store = {
        snap.get("$id", ""): snap,
        node.get("$id", ""): node,
        tags.get("$id", ""): tags,
        f"{base}assembly_graph_node_v1.schema.json": node,
        f"{base}aps_tag_taxonomy_v1.schema.json": tags,
    }
    resolver = jsonschema.RefResolver(base_uri=base, referrer=snap, store=store)
    jsonschema.validate(instance=data, schema=snap, resolver=resolver)


def validate_aps_tag_taxonomy(data: dict[str, Any]) -> None:
    jsonschema.validate(instance=data, schema=_load_schema("aps_tag_taxonomy_v1.schema.json"))


def validate_assembly_build_job(data: dict[str, Any]) -> None:
    jsonschema.validate(instance=data, schema=_load_schema("assembly_build_job_v1.schema.json"))


def validate_tile_variant_bake_job(data: dict[str, Any]) -> None:
    jsonschema.validate(instance=data, schema=_load_schema("tile_variant_bake_job_v1.schema.json"))


def validate_variant_set(data: dict[str, Any]) -> None:
    jsonschema.validate(instance=data, schema=_load_schema("variant_set_v1.schema.json"))


def validate_variant_catalog(data: dict[str, Any]) -> None:
    jsonschema.validate(instance=data, schema=_load_schema("variant_catalog_v1.schema.json"))


def validate_atlas_meta_v2(data: dict[str, Any]) -> None:
    jsonschema.validate(instance=data, schema=_load_schema("atlas_meta_v2.schema.json"))


def validate_visual_config(data: dict[str, Any]) -> None:
    jsonschema.validate(instance=data, schema=_load_schema("visual_config_v1.schema.json"))


def load_json_file(path: Path) -> dict[str, Any]:
    return json.loads(path.read_text(encoding="utf-8"))
