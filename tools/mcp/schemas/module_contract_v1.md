# Module geometric contract v1 (BQ-C1)

Authoritative constants for MCP module-kit bakes and Rust `module_index` resolution.

| Field | Value | Notes |
|-------|-------|-------|
| `grid_unit_m` | **4.0** | One horizontal grid cell in meters |
| `floor_height_m` | **3.0** | One vertical floor band in meters |
| `pivot_convention` | **bottom_center** | GLB origin at footprint center, bottom at y=0 |
| `edge_socket_names` | left, right, top, bottom | BQ-A1 adjacency profiles reference these edges |

Machine-readable instance: `tools/mcp/schemas/module_contract_v1.json`  
JSON Schema: `tools/mcp/schemas/module_contract_v1.schema.json`

## Family height rules

- **Wall** — `height_m = floor_height_m` (3.0); width = `n × grid_unit_m`.
- **Door** — opening may be shorter than the wall; frame/top must align to `target_wall_height_m` (BQ-C3 seam check).
- **Window** — sill/head measured from bottom pivot; must fit within `target_wall_height_m`.
- **Roof** — seat plane at export min y ≥ 0 (see BQ-F1 bake witness).
- **Corner** — matches wall `height_m` per style pack.

Python mirror: `rust_engine_mcp.module_contract`  
Rust mirror: `crate::construction::procedural::module_contract`
