"""Production tier policy — TIER-001..006 for validate_asset_report."""

from __future__ import annotations

import json
import re
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any

from rust_engine_mcp.paths import repo_root
from rust_engine_mcp.schemas import load_json_file

from .module_inventory import CANONICAL_MODULE_IDS

DevelopmentTier = str  # smoke | lod0 | production
BOX_VERTEX_CEILING = 32

BOX_OK_AT_LOD0: dict[str, frozenset[str]] = {
    "module_wall": frozenset({"flat", "panel", ""}),
    "module_door": frozenset({"flat", ""}),
    "module_prop": frozenset({"box", ""}),
    "module_roof": frozenset({"flat", ""}),
    "module_window": frozenset({""}),
}

NON_BOX_PROFILES: dict[str, frozenset[str]] = {
    "module_wall": frozenset({"recess", "brick", "masonry", "panel_recess", "inset"}),
    "module_roof": frozenset({"pitched", "pitched_gable", "gable", "shed", "sawtooth"}),
    "module_window": frozenset({"mullion", "frame_mullion", "single", "arched", "curtain", "strip"}),
    "module_door": frozenset({"residential", "frame", "shop", "lod0"}),
    "module_prop": frozenset({"l_corner", "corner", "corner_l", "chimney", "prop_chimney", "vent", "ac"}),
}

ASSET_ID_PROFILE_HINTS: list[tuple[re.Pattern[str], str, str]] = [
    (re.compile(r"wall_brick", re.I), "module_wall", "brick"),
    (re.compile(r"pitched|gable", re.I), "module_roof", "pitched"),
    (re.compile(r"sawtooth", re.I), "module_roof", "sawtooth"),
    (re.compile(r"shed", re.I), "module_roof", "shed"),
    (re.compile(r"win_arched|arched", re.I), "module_window", "arched"),
    (re.compile(r"win_single|win_house", re.I), "module_window", "mullion"),
    (re.compile(r"^win_", re.I), "module_window", "mullion"),
    (re.compile(r"door_residential", re.I), "module_door", "residential"),
    (re.compile(r"^door_", re.I), "module_door", "frame"),
    (re.compile(r"prop_chimney", re.I), "module_prop", "chimney"),
    (re.compile(r"corner_l", re.I), "module_prop", "corner_l"),
    (re.compile(r"roof_pitched", re.I), "module_roof", "pitched"),
]

# Approved tileable / Material Maker set ids for production promote (MCP-PROD-PBR-PILOT).
PRODUCTION_TILEABLE_SET_IDS: frozenset[str] = frozenset(
    {
        "brick_red_01",
        "concrete_grey_01",
        "steel_panel_01",
        "steel_corner_01",
        "steel_door_warehouse_01",
        "wood_plank_01",
        "stucco_cream_01",
        "glass_panel_01",
        "roof_tile_01",
        "roof_metal_01",
        "metal_roof_01",
    }
)


@dataclass
class TierIssue:
    rule_id: str
    kind: str
    severity: str
    hint: str
    signature: str


@dataclass
class AssetValidationContext:
    glb_path: Path
    vertex_count: int | None = None
    archetype: str = ""
    profile: str = ""
    development_tier: str = ""
    pbr_status: str = ""
    batch_id: str = ""
    module_id: str = ""
    material_profile: str = ""
    tileable_set_id: str = ""
    references: list[str] = field(default_factory=list)
    spec_path: str = ""
    job_path: str = ""

    def effective_tier(self) -> str:
        if self.development_tier in ("smoke", "lod0", "production"):
            return self.development_tier
        return infer_development_tier(
            {"references": self.references, "batch_id": self.batch_id},
            self.batch_id,
            default="smoke",
        )

    def expects_non_box_silhouette(self) -> bool:
        arch = self.archetype or ""
        profile = (self.profile or "").lower()
        if profile and profile in NON_BOX_PROFILES.get(arch, frozenset()):
            return True
        for pattern, hint_arch, hint_profile in ASSET_ID_PROFILE_HINTS:
            if hint_arch == arch and pattern.search(self.module_id):
                if hint_profile in NON_BOX_PROFILES.get(arch, frozenset()):
                    return True
        return False

    def box_silhouette_allowed(self) -> bool:
        if self.effective_tier() == "smoke":
            return True
        arch = self.archetype or ""
        profile = (self.profile or "").lower()
        if self.expects_non_box_silhouette():
            return False
        allowed = BOX_OK_AT_LOD0.get(arch, frozenset())
        return profile in allowed or profile == ""


def infer_development_tier(
    spec: dict[str, Any],
    batch_id: str,
    *,
    default: str = "smoke",
) -> str:
    explicit = spec.get("development_tier")
    if explicit in ("smoke", "lod0", "production"):
        return str(explicit)
    refs = spec.get("references") or []
    if any(str(r).startswith("greybox:") for r in refs):
        return "smoke"
    bid = batch_id or str(spec.get("batch_id") or "")
    if bid.startswith("kit_greybox") or bid.startswith("kit_smoke"):
        return "smoke"
    if bid.startswith("kit_lod0"):
        return "lod0"
    if bid.startswith("kit_production"):
        return "production"
    return default


def infer_stylepack_visible(tier: str) -> bool:
    return tier != "smoke"


def infer_pbr_status(spec: dict[str, Any], tier: str) -> str:
    explicit = spec.get("pbr_status")
    if explicit in ("none", "deferred", "shipped"):
        return str(explicit)
    if tier == "smoke":
        return "none"
    if tier == "lod0":
        return "deferred"
    return "none"


def infer_profile_from_context(ctx: AssetValidationContext) -> str:
    if ctx.profile:
        return ctx.profile.lower()
    for pattern, hint_arch, hint_profile in ASSET_ID_PROFILE_HINTS:
        if hint_arch == ctx.archetype and pattern.search(ctx.module_id):
            return hint_profile
    if ctx.archetype == "module_roof" and "flat" in ctx.module_id:
        return "flat"
    return ""


def _find_job_json(job_id: str) -> Path | None:
    candidates = [
        repo_root() / "tools" / "mcp" / "schemas" / "examples" / f"{job_id}.json",
        repo_root() / "tools" / "mcp" / "jobs" / f"{job_id}.json",
    ]
    status_path = repo_root() / "tools" / "mcp" / "jobs" / f"{job_id}.status.json"
    if status_path.is_file():
        try:
            status = json.loads(status_path.read_text(encoding="utf-8"))
            jp = status.get("job_path")
            if jp:
                p = Path(str(jp))
                if not p.is_absolute():
                    p = repo_root() / p
                if p.is_file():
                    return p
        except (json.JSONDecodeError, OSError):
            pass
    for p in candidates:
        if p.is_file():
            return p
    return None


def resolve_asset_context(glb_path: Path) -> AssetValidationContext:
    glb_path = glb_path.resolve()
    ctx = AssetValidationContext(glb_path=glb_path)

    if "models" in glb_path.parts and "modules" in glb_path.parts:
        job_dir = glb_path.parent
        manifest = job_dir / "manifest.json"
        if manifest.is_file():
            try:
                man = json.loads(manifest.read_text(encoding="utf-8"))
                ctx.batch_id = str(man.get("batch_id") or "")
                ctx.vertex_count = man.get("vertex_count")
            except (json.JSONDecodeError, OSError):
                pass
        sidecars = list(job_dir.glob("*.module.json"))
        if sidecars:
            try:
                spec = json.loads(sidecars[0].read_text(encoding="utf-8"))
                _apply_spec(ctx, spec)
                ctx.spec_path = str(sidecars[0])
            except (json.JSONDecodeError, OSError):
                ctx.module_id = job_dir.name
        else:
            ctx.module_id = job_dir.name
        job_json = _find_job_json(job_dir.name)
        if job_json:
            _apply_job(ctx, load_json_file(job_json))
            ctx.job_path = str(job_json)

    elif "staging" in glb_path.parts:
        job_id = glb_path.parent.name
        ctx.module_id = job_id
        job_json = _find_job_json(job_id)
        if job_json:
            job = load_json_file(job_json)
            _apply_job(ctx, job)
            ctx.job_path = str(job_json)
            spec_ref = job.get("spec_ref")
            if spec_ref:
                spec_path = Path(str(spec_ref))
                if not spec_path.is_absolute():
                    spec_path = repo_root() / spec_path
                if spec_path.is_file():
                    _apply_spec(ctx, load_json_file(spec_path))
                    ctx.spec_path = str(spec_path)

    ctx.profile = infer_profile_from_context(ctx)
    if not ctx.development_tier:
        ctx.development_tier = infer_development_tier(
            {"references": ctx.references, "batch_id": ctx.batch_id},
            ctx.batch_id,
        )
    if not ctx.pbr_status:
        ctx.pbr_status = infer_pbr_status({}, ctx.effective_tier())
    return ctx


def _apply_spec(ctx: AssetValidationContext, spec: dict[str, Any]) -> None:
    ctx.module_id = str(spec.get("asset_id") or ctx.module_id)
    ctx.archetype = str(spec.get("archetype") or ctx.archetype)
    if spec.get("development_tier"):
        ctx.development_tier = str(spec["development_tier"])
    if spec.get("pbr_status"):
        ctx.pbr_status = str(spec["pbr_status"])
    ctx.references = [str(r) for r in (spec.get("references") or [])]
    if spec.get("batch_id"):
        ctx.batch_id = str(spec["batch_id"])
    if spec.get("material_profile"):
        ctx.material_profile = str(spec["material_profile"])
    elif spec.get("material_id"):
        ctx.material_profile = str(spec["material_id"])
    if spec.get("tileable_set_id"):
        ctx.tileable_set_id = str(spec["tileable_set_id"])


def _apply_job(ctx: AssetValidationContext, job: dict[str, Any]) -> None:
    ctx.archetype = str(job.get("operation") or ctx.archetype)
    if job.get("batch_id"):
        ctx.batch_id = str(job["batch_id"])
    if job.get("development_tier"):
        ctx.development_tier = str(job["development_tier"])
    params = job.get("params") or {}
    if params.get("profile"):
        ctx.profile = str(params["profile"]).lower()
    elif params.get("prop_kind"):
        ctx.profile = str(params["prop_kind"]).lower()


def tier_issues_for_asset(
    ctx: AssetValidationContext,
    *,
    vertex_count: int | None,
) -> list[TierIssue]:
    issues: list[TierIssue] = []
    tier = ctx.effective_tier()
    verts = vertex_count if vertex_count is not None else ctx.vertex_count
    batch = ctx.batch_id or ""
    explicit_tier = ctx.development_tier in ("smoke", "lod0", "production")

    # TIER-001: development_tier missing on kit_production_*
    if batch.startswith("kit_production_") and not explicit_tier:
        issues.append(
            TierIssue(
                "TIER-001",
                "MissingField",
                "error",
                "development_tier required for kit_production_* batches",
                "tier_missing_production",
            )
        )
    if batch.startswith("kit_production_") and tier != "production":
        issues.append(
            TierIssue(
                "TIER-001",
                "MissingField",
                "error",
                "kit_production_* batch requires development_tier: production",
                "tier_missing_production",
            )
        )

    # TIER-002: single-box silhouette at lod0/production for non-box profiles
    if tier in ("lod0", "production") and verts is not None and verts <= BOX_VERTEX_CEILING:
        if not ctx.box_silhouette_allowed():
            issues.append(
                TierIssue(
                    "TIER-002",
                    "SilhouetteInsufficient",
                    "error",
                    (
                        f"vertex_count={verts} is single-box greybox for "
                        f"{ctx.archetype} profile={ctx.profile or 'inferred'} at tier={tier}"
                    ),
                    "tier_silhouette_insufficient",
                )
            )

    # TIER-003: greybox refs with tier != smoke
    greybox_refs = [r for r in ctx.references if str(r).startswith("greybox:")]
    if greybox_refs and tier != "smoke":
        issues.append(
            TierIssue(
                "TIER-003",
                "SmokeAsProduction",
                "error",
                f"references {greybox_refs} require development_tier: smoke",
                "tier_smoke_stylepack",
            )
        )

    # TIER-004: production requires pbr_status shipped + tileable material id
    if tier == "production" and ctx.pbr_status != "shipped":
        issues.append(
            TierIssue(
                "TIER-004",
                "PbrNotShipped",
                "error",
                f"tier=production requires pbr_status: shipped (got {ctx.pbr_status!r})",
                "tier_pbr_not_shipped",
            )
        )
    tileable = (ctx.tileable_set_id or ctx.material_profile or "").strip()
    if tier == "production" and not tileable:
        issues.append(
            TierIssue(
                "TIER-004",
                "MissingField",
                "error",
                "production assets require material_profile or tileable_set_id (PBR-pilot)",
                "tier_missing_tileable_set",
            )
        )
    elif tier == "production" and tileable not in PRODUCTION_TILEABLE_SET_IDS:
        issues.append(
            TierIssue(
                "TIER-004",
                "UnknownTileableSet",
                "error",
                f"tileable id {tileable!r} not in production pilot allowlist",
                "tier_unknown_tileable_set",
            )
        )

    # TIER-005: module_id canonical inventory (lod0 warning, production error)
    if tier in ("lod0", "production") and ctx.module_id:
        if ctx.module_id not in CANONICAL_MODULE_IDS:
            sev = "error" if tier == "production" else "warning"
            issues.append(
                TierIssue(
                    "TIER-005",
                    "UnknownModuleId",
                    sev,
                    f"module_id {ctx.module_id!r} not in canonical kit inventory ({len(CANONICAL_MODULE_IDS)} IDs)",
                    "tier_unknown_module_id",
                )
            )

    # TIER-006: legacy kit_greybox_* harness (warn on existing assets; error on new jobs in tier_issues_for_job)
    if batch.startswith("kit_greybox_"):
        issues.append(
            TierIssue(
                "TIER-006",
                "BatchRetired",
                "warning",
                f"batch_id {batch} is legacy smoke harness — not StylePack art",
                "tier_batch_retired",
            )
        )

    return issues


def tier_issues_for_job(job: dict[str, Any], job_path: Path) -> list[TierIssue]:
    issues: list[TierIssue] = []
    batch = str(job.get("batch_id") or "")
    tier = str(job.get("development_tier") or "")

    if batch.startswith("kit_greybox_"):
        issues.append(
            TierIssue(
                "TIER-006",
                "BatchRetired",
                "error",
                f"{job_path.name}: kit_greybox_* batches frozen — use kit_lod0_* or kit_production_*",
                "tier_batch_retired",
            )
        )

    if batch.startswith("kit_production_") and tier != "production":
        issues.append(
            TierIssue(
                "TIER-001",
                "MissingField",
                "error",
                "development_tier: production required for kit_production_* jobs",
                "tier_missing_production",
            )
        )

    if batch.startswith("kit_lod0_") and not tier:
        issues.append(
            TierIssue(
                "TIER-001",
                "MissingField",
                "warning",
                "development_tier should be lod0 for kit_lod0_* batches",
                "tier_missing_lod0",
            )
        )

    params = job.get("params") or {}
    op = str(job.get("operation") or "")
    profile = str(params.get("profile") or "").lower()
    if tier in ("lod0", "production") and op == "module_roof" and not profile:
        if batch.startswith("kit_lod0"):
            issues.append(
                TierIssue(
                    "TIER-002",
                    "MissingField",
                    "warning",
                    "roof jobs at lod0+ should set params.profile (flat|pitched|shed|sawtooth)",
                    "tier_missing_roof_profile",
                )
            )

    return issues


def tier_issues_for_spec(spec: dict[str, Any], spec_path: Path) -> list[TierIssue]:
    issues: list[TierIssue] = []
    batch = str(spec.get("batch_id") or "")
    tier = infer_development_tier(spec, batch, default="")
    refs = spec.get("references") or []
    module_id = str(spec.get("asset_id") or "")

    if any(str(r).startswith("greybox:") for r in refs) and tier not in ("", "smoke"):
        issues.append(
            TierIssue(
                "TIER-003",
                "SmokeAsProduction",
                "error",
                "greybox: references force development_tier: smoke",
                "tier_smoke_stylepack",
            )
        )

    if batch.startswith("kit_production_") and tier != "production":
        issues.append(
            TierIssue(
                "TIER-001",
                "MissingField",
                "error",
                "development_tier: production required for kit_production_* specs",
                "tier_missing_production",
            )
        )

    if tier in ("lod0", "production") and not spec.get("pbr_status"):
        issues.append(
            TierIssue(
                "TIER-004",
                "MissingField",
                "warning",
                "pbr_status required when development_tier is lod0 or production",
                "tier_missing_pbr_status",
            )
        )

    if tier in ("lod0", "production") and module_id and module_id not in CANONICAL_MODULE_IDS:
        sev = "error" if tier == "production" else "warning"
        issues.append(
            TierIssue(
                "TIER-005",
                "UnknownModuleId",
                sev,
                f"asset_id {module_id!r} not in canonical kit inventory",
                "tier_unknown_module_id",
            )
        )

    return issues


MCP_PROD_B2_WITNESS = "debug_runs/mcp_prod_b2_live.json"


def _b2_silhouette_case(archetype: str, profile: str, module_id: str) -> bool:
    ctx = AssetValidationContext(
        glb_path=repo_root() / "x.glb",
        vertex_count=24,
        archetype=archetype,
        profile=profile,
        development_tier="production",
        batch_id="kit_production_001",
        module_id=module_id,
        pbr_status="shipped",
        material_profile="brick_red_01",
    )
    issues = tier_issues_for_asset(ctx, vertex_count=24)
    return any(i.rule_id == "TIER-002" and i.severity == "error" for i in issues)


def refresh_mcp_prod_b2_witness() -> bool:
    """MCP-PROD-B2 — 24-vert cube fails pitched/sawtooth/arched at production tier."""
    import json

    cases = [
        ("module_roof", "pitched", "roof_pitched_gable"),
        ("module_roof", "sawtooth", "roof_sawtooth"),
        ("module_window", "arched", "win_arched_1u"),
    ]
    silhouette_hits = {
        f"{a}:{p}": _b2_silhouette_case(a, p, mid) for a, p, mid in cases
    }
    green = all(silhouette_hits.values())
    payload = {
        "gate_id": "MCP-PROD-B2",
        "ok": green,
        "green": green,
        "phase": "B2",
        "acceptance": "24-vert cube fails pitched/sawtooth/arched at production",
        "silhouette_cases": silhouette_hits,
        "tier_rules": ["TIER-002"],
    }
    out = repo_root() / MCP_PROD_B2_WITNESS
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
    return green
