"""APS-VALIDATOR-PLAIN-002 — P0 validator codes → artist sentences."""

from __future__ import annotations

from dataclasses import dataclass
from typing import Any

from .validators.report import ValidationReport

APS_VALIDATOR_PLAIN_WITNESS = "debug_runs/aps_validator_plain_002_live.json"


@dataclass(frozen=True)
class PlainEntry:
    signature: str
    kind: str
    sentence: str
    fix_hint: str


# Authoritative map from docs/archive/2026-06-src-dev/plans/aps_validator_plain_language_v1.md
PLAIN_ENTRIES: tuple[PlainEntry, ...] = (
    # production
    PlainEntry(
        "assembly_production_tier",
        "TierMismatch",
        "This snapshot is not marked **production** — bake/ship needs production modules.",
        "Set tier to **production** when generating, or regenerate from Assembly.",
    ),
    PlainEntry(
        "assembly_production_min_placements",
        "ModuleCount",
        "The building has too few pieces — it won't read as a structure.",
        "Generate again or widen footprint until you have at least 4 placements.",
    ),
    PlainEntry(
        "assembly_production_unique_modules",
        "ModuleCount",
        "Not enough different module types — looks like a repeated strip.",
        "Check style pack / grammar; need more wall/roof/door variety.",
    ),
    PlainEntry(
        "assembly_production_missing_glb",
        "MissingField",
        "A placement is missing its **3D file path** — the worker can't load it.",
        "Regenerate snapshot or fix module index / promote GLB.",
    ),
    PlainEntry(
        "assembly_production_glb_path",
        "NonProductionGlb",
        "A wall/roof module is still a **greybox or lod0** file — not production art.",
        "Promote module in Catalog (production_run GLB).",
    ),
    PlainEntry(
        "assembly_production_lod0_rejected",
        "Lod0Module",
        "Greybox modules are in the building shell — swap to production GLBs.",
        "Run module promotion batch; refresh Catalog.",
    ),
    PlainEntry(
        "assembly_production_glb_missing",
        "MissingFile",
        "One or more **GLB files are missing on disk**.",
        "Promote or reindex modules; verify path in Catalog.",
    ),
    PlainEntry(
        "assembly_graph_material_profile",
        "MissingField",
        "Some placements have **no material** — assign in APS Material library.",
        "Select each cell → pick profile → Save snapshot.",
    ),
    PlainEntry(
        "assembly_production_snapshot_missing",
        "MissingFile",
        "Snapshot file not found.",
        "Load or Save a valid assembly JSON.",
    ),
    # grammar
    PlainEntry(
        "grammar_verify_footprint_min",
        "FootprintTooSmall",
        "Footprint is **too small** to read as a building.",
        "Increase W×D (minimum 3×3).",
    ),
    PlainEntry(
        "grammar_verify_warehouse_footprint",
        "WarehouseFootprintThin",
        "Warehouse footprint is **too narrow** — looks like a fence, not a hall.",
        "Use at least **4×3** for Industrial Warehouse.",
    ),
    PlainEntry(
        "grammar_verify_perimeter_count",
        "PerimeterIncomplete",
        "Building **shell is incomplete** — missing wall or roof ring cells.",
        "Regenerate with grammar on; check massing strategy.",
    ),
    PlainEntry(
        "grammar_verify_missing_wall",
        "MissingWallModule",
        "No **wall modules** in this assembly.",
        "Check grammar / style pack wall slot.",
    ),
    PlainEntry(
        "grammar_verify_missing_roof",
        "MissingRoofModule",
        "No **roof modules** — open-top stack.",
        "Check roof slot in grammar / district.",
    ),
    PlainEntry(
        "grammar_verify_grammar_chain",
        "GrammarChainMissing",
        "Snapshot is missing **grammar history** (how it was generated).",
        "Regenerate with **Use building grammar** checked.",
    ),
    PlainEntry(
        "grammar_verify_style_pack_drift",
        "StylePackDrift",
        "Some modules belong to a **different style pack** than the snapshot.",
        "Regenerate or fix style pack / module index rows.",
    ),
    PlainEntry(
        "grammar_verify_snapshot_missing",
        "MissingFile",
        "Snapshot file not found.",
        "Load valid JSON.",
    ),
    PlainEntry(
        "grammar_integration_preset_pair",
        "GrammarPresetPair",
        "DNA preset does not match pilot catalog or grammar file.",
        "Pick preset from catalog; run grammar-preset-pair-validate.",
    ),
    PlainEntry(
        "grammar_integration_preset_missing",
        "GrammarPresetPair",
        "Grammar snapshot has no linked **arch DNA preset**.",
        "Regenerate from a catalog pilot or set arch_dna_preset_id.",
    ),
    PlainEntry(
        "grammar_integration_site_missing",
        "SiteComposition",
        "Pilot **site JSON** is missing for this DNA preset.",
        "Add site file under assets/configs/buildings/pilots/.",
    ),
    # materials
    PlainEntry(
        "material_profiles_placement_missing",
        "MissingMaterialProfile",
        "A placement has **no material profile**.",
        "Assembly tab → select cell → Materials library → Save.",
    ),
    PlainEntry(
        "material_profiles_unknown_id",
        "UnknownMaterialProfile",
        "Material **not in registry** — worker may fall back to grey.",
        "Materials tab → Add/Generate profile → Register.",
    ),
    PlainEntry(
        "material_profiles_missing_albedo",
        "MissingTexture",
        "Material is missing **color texture (albedo)**.",
        "Materials tab → Generate or drop `albedo.png` in profile folder.",
    ),
    PlainEntry(
        "material_profiles_missing_normal_roughness",
        "MissingTexture",
        "Material is missing **normal or roughness** maps (ship warning).",
        "Add maps or accept pilot albedo-only for preview.",
    ),
    PlainEntry(
        "material_profiles_snapshot_missing",
        "MissingFile",
        "Snapshot file not found.",
        "Load valid JSON.",
    ),
)

_BY_SIGNATURE: dict[str, PlainEntry] = {e.signature: e for e in PLAIN_ENTRIES}
_BY_KIND: dict[str, PlainEntry] = {e.kind: e for e in PLAIN_ENTRIES}


def lookup_plain(signature: str = "", kind: str = "") -> PlainEntry | None:
    if signature and signature in _BY_SIGNATURE:
        return _BY_SIGNATURE[signature]
    if kind and kind in _BY_KIND:
        return _BY_KIND[kind]
    return None


def plain_sentence(signature: str = "", kind: str = "", *, fallback: str = "") -> str:
    entry = lookup_plain(signature, kind)
    if entry:
        return entry.sentence.replace("**", "")
    return fallback or kind or signature or "Validation failed."


def fix_hint(signature: str = "", kind: str = "", *, fallback: str = "") -> str:
    entry = lookup_plain(signature, kind)
    if entry:
        return entry.fix_hint.replace("**", "")
    return fallback


def format_p0_issue_line(issue: Any) -> str:
    """One artist-facing block: bullet sentence + arrow fix hint."""
    sig = str(getattr(issue, "signature", "") or "")
    kind = str(getattr(issue, "kind", "") or "")
    hint = str(getattr(issue, "hint", "") or "")
    sentence = plain_sentence(sig, kind, fallback=hint or kind)
    arrow = fix_hint(sig, kind, fallback=hint)
    if arrow and arrow != sentence:
        return f"● {sentence}\n  → {arrow}"
    return f"● {sentence}"


def format_p0_display(report: ValidationReport, *, limit: int = 12) -> str:
    if report.status == "passed":
        return "All checks passed — safe to Save and continue toward variants/atlas."
    blocks: list[str] = []
    for issue in report.errors:
        if issue.severity != "error":
            continue
        blocks.append(format_p0_issue_line(issue))
        if len(blocks) >= limit:
            break
    if not blocks:
        return report.summary or "Validation failed."
    return "\n\n".join(blocks)


def plain_validation_lines(report: ValidationReport, *, limit: int = 12) -> list[str]:
    """Backward-compatible single-line list (sentence only)."""
    if report.status == "passed":
        return ["All checks passed — safe to Save and continue toward variants/atlas."]
    lines: list[str] = []
    for issue in report.errors:
        if issue.severity != "error":
            continue
        sig = issue.signature or issue.kind
        line = plain_sentence(issue.signature or "", issue.kind or "", fallback=issue.hint or issue.kind)
        if issue.field:
            line = f"{line} ({issue.field})"
        lines.append(line)
        if len(lines) >= limit:
            break
    if not lines:
        lines.append(report.summary or "Validation failed.")
    return lines


def refresh_aps_validator_plain_witness() -> bool:
    from .paths import repo_root

    signatures = {e.signature for e in PLAIN_ENTRIES}
    wired = len(signatures) >= 22
    panel_src = (repo_root() / "tools/mcp/art_pipeline_suite/assembly_panel.py").read_text(
        encoding="utf-8"
    )
    ui_uses_plain = "aps_validator_plain" in panel_src or "format_p0_display" in panel_src
    payload = {
        "program_id": "APS-VALIDATOR-PLAIN-002",
        "gate": "APS-VALIDATOR-PLAIN-002",
        "green": wired and ui_uses_plain,
        "code_count": len(PLAIN_ENTRIES),
        "signatures": sorted(signatures),
        "assembly_panel_wired": ui_uses_plain,
    }
    path = repo_root() / APS_VALIDATOR_PLAIN_WITNESS
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(__import__("json").dumps(payload, indent=2) + "\n", encoding="utf-8")
    return bool(payload["green"])
