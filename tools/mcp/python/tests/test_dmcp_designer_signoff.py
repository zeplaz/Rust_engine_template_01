"""DMCP designer sign-off witnesses."""

from rust_engine_mcp.dmcp_designer_signoff import (
    refresh_dmcp_e0_artist_reverdict_witness,
    refresh_dmcp_e4_matrix_witness,
    verify_e0_artist_reverdict,
    verify_e4_matrix_charter,
)


def test_e4_matrix_charter_green() -> None:
    body = verify_e4_matrix_charter()
    assert body.get("green") is True
    assert body.get("verdict") == "PASS"
    assert len(body.get("variant_keys") or []) == 16


def test_e0_reverdict_stamps_signoff() -> None:
    witness = refresh_dmcp_e0_artist_reverdict_witness()
    assert witness.get("green") is True
    assert witness.get("verdict") == "PASS_WITH_NOTES"
    assert witness.get("e0_stamp", {}).get("designer_mcp_signoff") == "signed"
    e0 = verify_e0_artist_reverdict()
    assert e0.get("checks", {}).get("e0_witness_green") is True


def test_e4_witness_written() -> None:
    body = refresh_dmcp_e4_matrix_witness()
    assert body.get("written")
