"""BUILDING-LOOK-V2 — dual-face corners + full-footprint roof seat."""

from __future__ import annotations

import math

from rust_engine_mcp.assembly import (
    exterior_faces,
    footprint_grid,
    full_footprint_roof_scale,
    generate_assembly_snapshot,
    outward_yaw_for_face,
)


def test_exterior_faces_corners_are_dual() -> None:
    assert exterior_faces(0, 0, 4, 2) == ["S", "W"]
    assert exterior_faces(3, 1, 4, 2) == ["N", "E"]
    assert exterior_faces(1, 0, 4, 2) == ["S"]


def test_footprint_dual_face_and_single_roof() -> None:
    cells = footprint_grid(4, 2, 1)
    walls = [c for c in cells if c["token"] != "R"]
    roofs = [c for c in cells if c["token"] == "R"]
    assert len(roofs) == 1
    assert roofs[0].get("roof_mode") == "full_footprint"
    # 4×2 perimeter: 4 corners×2 + 4 mid-edge = 12 wall faces
    assert len(walls) == 12
    assert outward_yaw_for_face("S") == 0.0
    assert abs(outward_yaw_for_face("N") - math.pi) < 1e-9


def test_full_footprint_roof_scale_beyond_4x2() -> None:
    assert full_footprint_roof_scale(4, 2) == [1.0, 1.0, 1.0]
    assert full_footprint_roof_scale(6, 3) == [1.5, 1.0, 1.5]
    assert full_footprint_roof_scale(5, 3) == [1.5, 1.0, 1.25]


def test_rural_uses_pitched_gable_not_flat_tile() -> None:
    snap = generate_assembly_snapshot(
        style_pack_id="style_rural",
        width=4,
        depth=2,
        floors=1,
        seed=3,
        write=False,
    )
    roofs = [p for p in snap["module_placements"] if p["token"] == "R"]
    assert len(roofs) == 1
    assert roofs[0]["module_id"] == "roof_pitched_gable"
    assert "roof_tile" not in {p["module_id"] for p in snap["module_placements"]}


def test_industrial_6x3_emits_roof_scale() -> None:
    snap = generate_assembly_snapshot(
        style_pack_id="style_industrial_west",
        width=6,
        depth=3,
        floors=1,
        seed=5,
        write=False,
    )
    roofs = [p for p in snap["module_placements"] if p["token"] == "R"]
    assert len(roofs) == 1
    assert roofs[0].get("scale") == [1.5, 1.0, 1.5]