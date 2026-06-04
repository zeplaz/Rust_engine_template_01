"""APS-TAGS-002 — categorized semantic tags (taxonomy load, flat ↔ semantic sync)."""

from __future__ import annotations

from functools import lru_cache
from typing import Any

from .paths import schemas_dir
from .schemas import load_json_file, validate_aps_tag_taxonomy

CATEGORY_ORDER = ("location", "architectural", "detail", "condition")


@lru_cache(maxsize=1)
def load_taxonomy() -> dict[str, Any]:
    path = schemas_dir() / "examples" / "aps_tag_taxonomy_v1.json"
    data = load_json_file(path)
    validate_aps_tag_taxonomy(data)
    return data


def category_labels() -> dict[str, str]:
    tax = load_taxonomy()
    out: dict[str, str] = {}
    for cat, body in (tax.get("categories") or {}).items():
        out[str(cat)] = str(body.get("label") or cat)
    return out


def tags_for_category(category: str) -> list[dict[str, Any]]:
    tax = load_taxonomy()
    body = (tax.get("categories") or {}).get(category) or {}
    return list(body.get("tags") or [])


def legacy_flat_map() -> dict[str, dict[str, str]]:
    tax = load_taxonomy()
    raw = tax.get("legacy_flat_map") or {}
    return {str(k): dict(v) for k, v in raw.items()}


def semantic_tags_from_flat(flat_tags: list[str] | None) -> dict[str, list[str]]:
    """Map legacy flat APS checkboxes → categorized semantic_tags."""
    mapping = legacy_flat_map()
    out: dict[str, list[str]] = {cat: [] for cat in CATEGORY_ORDER}
    for raw in flat_tags or []:
        key = str(raw).strip()
        if not key:
            continue
        if key in mapping:
            row = mapping[key]
            cat = str(row.get("category") or "architectural")
            tag_id = str(row.get("tag_id") or key)
        else:
            cat, tag_id = _guess_category(key)
        bucket = out.setdefault(cat, [])
        if tag_id not in bucket:
            bucket.append(tag_id)
    return {k: v for k, v in out.items() if v}


def flatten_semantic_tags(semantic: dict[str, Any] | None) -> list[str]:
    """Deduped flat list for validators / legacy placement_tags field."""
    if not semantic:
        return []
    reverse = _reverse_legacy_map()
    flat: list[str] = []
    seen: set[str] = set()
    for cat in CATEGORY_ORDER:
        for tag_id in semantic.get(cat) or []:
            tid = str(tag_id)
            legacy = reverse.get((cat, tid))
            token = legacy or tid
            if token not in seen:
                seen.add(token)
                flat.append(token)
    return flat


def sync_placement_tags(placement: dict[str, Any]) -> dict[str, Any]:
    """Ensure placement has both semantic_tags and placement_tags (idempotent)."""
    out = dict(placement)
    semantic = out.get("semantic_tags")
    flat = out.get("placement_tags")
    if semantic and not flat:
        out["placement_tags"] = flatten_semantic_tags(semantic)
    elif flat and not semantic:
        out["semantic_tags"] = semantic_tags_from_flat(list(flat))
    elif semantic and flat:
        out["placement_tags"] = flatten_semantic_tags(semantic)
    return out


def grammar_tags_for_category(category: str) -> list[str]:
    """Tag ids in taxonomy for grammar-driven defaults (facade → location+architectural)."""
    return [str(t.get("id")) for t in tags_for_category(category)]


def _reverse_legacy_map() -> dict[tuple[str, str], str]:
    out: dict[tuple[str, str], str] = {}
    for flat, row in legacy_flat_map().items():
        out[(str(row.get("category")), str(row.get("tag_id")))] = flat
    return out


def _guess_category(tag_id: str) -> tuple[str, str]:
    for cat in CATEGORY_ORDER:
        for row in tags_for_category(cat):
            if str(row.get("id")) == tag_id:
                return cat, tag_id
    if tag_id in {"clean", "weathered", "damaged", "abandoned", "construction", "fire"}:
        return "condition", tag_id
    if tag_id in {"industrial", "commercial", "residential"}:
        return "architectural", tag_id
    return "detail", tag_id
