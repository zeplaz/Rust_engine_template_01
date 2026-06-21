"""APS studio close + VEG-F02 burn atlas witnesses."""

from rust_engine_mcp.aps_studio_close import refresh_aps_studio_close_witness
from rust_engine_mcp.veg_f02_burn_atlas import refresh_veg_f02_burn_atlas_witness


def test_refresh_aps_studio_close_witness_green() -> None:
    body = refresh_aps_studio_close_witness()
    assert body.get("green") is True


def test_refresh_veg_f02_burn_atlas_witness_green() -> None:
    body = refresh_veg_f02_burn_atlas_witness()
    assert body.get("green") is True
