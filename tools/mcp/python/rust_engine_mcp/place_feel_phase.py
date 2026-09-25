"""Place-feel honesty for kit phase — no rebake, no site-phase alias.

`development_tier` (smoke | lod0 | production) is how finished a mesh is.
It is not a construction site phase (Planned … Operational), not the ghost
FSM (Preview / Adjust), and not deployable stock (in transit / staged / ready).

Optional `place_feel_claim` on a geometry job or AssetSpec must be legal for
that kit phase. A module must never claim the 32px picker/unit icon cell.
"""

from __future__ import annotations

from typing import Any

SCHEMA = "place_feel_phase_note_v1"
RULE_ID = "TIER-PLACE-001"

KIT_PHASES = ("smoke", "lod0", "production")
PLACE_FEEL_CLAIMS = (
    "harness_only",
    "preview_ghost_stand_in",
    "preview_ghost_same_mesh",
    "operational_envelope",
    "picker_icon",
)

# picker_icon is never legal on a 4 m module. The click icon is a 32px atlas cell.
ALLOWED: dict[str, frozenset[str]] = {
    "smoke": frozenset({"harness_only"}),
    "lod0": frozenset({"preview_ghost_stand_in"}),
    "production": frozenset({"preview_ghost_same_mesh", "operational_envelope"}),
}

_HINTS: dict[tuple[str, str], str] = {
    ("smoke", "preview_ghost_stand_in"): "Smoke harness is not a cursor ghost.",
    ("smoke", "preview_ghost_same_mesh"): "Smoke cannot be the mesh that stays after place.",
    ("smoke", "operational_envelope"): "Smoke cannot stand in for an Operational site.",
    ("smoke", "picker_icon"): "A smoke GLB is not a 32px click icon.",
    ("lod0", "harness_only"): "lod0 is a labeled stand-in, not a smoke harness.",
    ("lod0", "preview_ghost_same_mesh"): (
        "lod0 claiming to be the placed mesh is a lie — ghost and Operational must share production."
    ),
    ("lod0", "operational_envelope"): (
        "lod0 on an Operational/Built read is the place-feel lie. "
        "Coverage that counts any GLB stays green while the click lands a stand-in."
    ),
    ("lod0", "picker_icon"): "An lod0 module is not the picker icon.",
    ("production", "harness_only"): "A production mesh is not a smoke harness.",
    ("production", "preview_ghost_stand_in"): (
        "A production ghost is the same mesh as the placed envelope, not a cheaper stand-in."
    ),
    ("production", "picker_icon"): (
        "A production GLB is not the 32px atlas cell. "
        "Building place-targets and unit movers already share that sheet — do not collapse them into the module."
    ),
}


def allowed_claims(kit_phase: str) -> list[str]:
    return sorted(ALLOWED.get(kit_phase, frozenset()))


def phase_note(kit_phase: str, claim: str | None = None) -> dict[str, Any]:
    """Structured note. Illegal claim sets ok=false. Missing claim is ok."""
    issues = claim_issue_rows(kit_phase, claim) if claim else []
    return {
        "schema": SCHEMA,
        "kit_phase": kit_phase,
        "not_a": [
            "site_phase",
            "ghost_fsm",
            "deployable_staging",
            "icon_atlas_cell",
        ],
        "allowed_claims": allowed_claims(kit_phase),
        "place_feel_claim": claim,
        "issues": issues,
        "ok": not issues and kit_phase in ALLOWED,
        "note": (
            "development_tier is kit phase only. "
            "Do not encode Planned, ghost Preview, or in-transit stock by swapping smoke/lod0/production."
        ),
    }


def claim_issue_rows(kit_phase: str, claim: str | None) -> list[dict[str, str]]:
    """Empty when claim is omitted. Errors when the pair is illegal or unknown."""
    if not claim:
        return []
    if claim not in PLACE_FEEL_CLAIMS:
        return [
            {
                "rule_id": RULE_ID,
                "kind": "UnknownPlaceFeelClaim",
                "severity": "error",
                "hint": f"place_feel_claim {claim!r} is not a known surface",
                "signature": "place_feel_unknown_claim",
            }
        ]
    if kit_phase not in ALLOWED:
        return [
            {
                "rule_id": RULE_ID,
                "kind": "UnknownKitPhase",
                "severity": "error",
                "hint": (
                    f"place_feel_claim {claim!r} requires development_tier "
                    "smoke|lod0|production (kit phase, not a site phase)"
                ),
                "signature": "place_feel_unknown_tier",
            }
        ]
    if claim in ALLOWED[kit_phase]:
        return []
    hint = _HINTS.get(
        (kit_phase, claim),
        f"place_feel_claim {claim!r} is illegal at development_tier {kit_phase!r}",
    )
    return [
        {
            "rule_id": RULE_ID,
            "kind": "PlaceFeelPhaseLie",
            "severity": "error",
            "hint": hint,
            "signature": f"place_feel_{kit_phase}_{claim}",
        }
    ]


def claim_issue_rows_for_record(record: dict[str, Any]) -> list[dict[str, str]]:
    """Lint only when the record asserts a place_feel_claim. Absent claim is silent."""
    if "place_feel_claim" not in record:
        return []
    claim = record.get("place_feel_claim")
    tier = str(record.get("development_tier") or "")
    if claim is None or claim == "":
        return [
            {
                "rule_id": RULE_ID,
                "kind": "MissingField",
                "severity": "error",
                "hint": "place_feel_claim is present but empty",
                "signature": "place_feel_empty_claim",
            }
        ]
    return claim_issue_rows(tier, str(claim))
