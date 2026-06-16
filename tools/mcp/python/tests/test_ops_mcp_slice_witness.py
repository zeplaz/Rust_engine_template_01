"""OPS-006 / OPS-007 slice witnesses."""

from rust_engine_mcp.ops_mcp_slice_witness import (
    OPS_007_WITNESS_PATH,
    write_ops_007_warehouse_production_pause_witness,
)
from rust_engine_mcp.paths import repo_root


def test_ops_007_pause_witness() -> None:
    body = write_ops_007_warehouse_production_pause_witness()
    assert body["ops_id"] == "OPS-007"
    assert body["status"] == "paused"
    assert body["ok"] is True
    assert body["green"] is False
    assert body["variant_matrix_exists"] is True
    assert body["tile_batch_exists"] is True
    path = repo_root() / OPS_007_WITNESS_PATH
    assert path.is_file()
