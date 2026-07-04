//! **BQ-A1-ADJ-001** — footprint edge-compatibility rules (constraint check, not WFC).

use super::footprint_grid::{FootprintGrid, FootprintToken};
use super::FootprintCell;

pub const BQ_A1_LIVE_JSON: &str = "debug_runs/bq_a1_adjacency_001_live.json";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdjacencyViolation {
    pub rule_id: &'static str,
    pub message: String,
}

impl FootprintGrid {
    #[must_use]
    pub fn cell_at(&self, x: u32, y: u32, floor: u32) -> Option<&FootprintCell> {
        self.cells
            .iter()
            .find(|c| c.x == x && c.y == y && c.floor == floor)
    }

    #[must_use]
    fn perimeter_cardinal_neighbors(&self, x: u32, y: u32) -> Vec<(u32, u32)> {
        let mut out = Vec::with_capacity(4);
        if x > 0 {
            out.push((x - 1, y));
        }
        if y > 0 {
            out.push((x, y - 1));
        }
        if x + 1 < self.width {
            out.push((x + 1, y));
        }
        if y + 1 < self.depth {
            out.push((x, y + 1));
        }
        out
            .into_iter()
            .filter(|(nx, ny)| is_perimeter(*nx, *ny, self.width, self.depth))
            .collect()
    }
}

#[must_use]
fn is_perimeter(x: u32, y: u32, width: u32, depth: u32) -> bool {
    x == 0 || y == 0 || x + 1 == width || y + 1 == depth
}

#[must_use]
fn is_corner(x: u32, y: u32, width: u32, depth: u32) -> bool {
    (x == 0 || x + 1 == width) && (y == 0 || y + 1 == depth)
}

fn is_facade_token(token: FootprintToken) -> bool {
    matches!(
        token,
        FootprintToken::Wall | FootprintToken::Door | FootprintToken::Corner
    )
}

/// Five v1 adjacency rules from `plan_building_quality_v1.md` § BQ-A1.
#[must_use]
pub fn check_footprint_adjacency(grid: &FootprintGrid) -> Vec<AdjacencyViolation> {
    let mut violations = Vec::new();

    // Rule 2 — door cells never at corners.
    for cell in &grid.cells {
        if cell.token == FootprintToken::Door
            && is_corner(cell.x, cell.y, grid.width, grid.depth)
        {
            violations.push(AdjacencyViolation {
                rule_id: "door_not_corner",
                message: format!("door at corner ({},{}) floor {}", cell.x, cell.y, cell.floor),
            });
        }
    }

    // Rule 1 — corner cells have facade neighbors on exactly two adjacent perimeter sides.
    for cell in &grid.cells {
        if cell.token != FootprintToken::Corner || cell.floor >= grid.floors {
            continue;
        }
        let neighbors = grid.perimeter_cardinal_neighbors(cell.x, cell.y);
        let facade_neighbors = neighbors
            .iter()
            .filter_map(|(nx, ny)| grid.cell_at(*nx, *ny, cell.floor))
            .filter(|n| is_facade_token(n.token))
            .count();
        if facade_neighbors != 2 {
            violations.push(AdjacencyViolation {
                rule_id: "corner_two_neighbors",
                message: format!(
                    "corner ({},{}) floor {} has {facade_neighbors} facade neighbors, expected 2",
                    cell.x, cell.y, cell.floor
                ),
            });
        }
    }

    // Rule 3 — at most one door per perimeter edge per floor.
    for floor in 0..grid.floors {
        for edge in ["north", "south", "east", "west"] {
            let doors = grid.cells.iter().filter(|c| {
                c.floor == floor
                    && c.token == FootprintToken::Door
                    && edge_door_match(c, edge, grid.width, grid.depth)
            });
            let count = doors.count();
            if count > 1 {
                violations.push(AdjacencyViolation {
                    rule_id: "one_door_per_edge",
                    message: format!("{edge} edge floor {floor} has {count} doors"),
                });
            }
        }
    }

    // Rule 4 — roof covers full perimeter (no gaps).
    let roof_floor = grid.floors;
    for y in 0..grid.depth {
        for x in 0..grid.width {
            if !is_perimeter(x, y, grid.width, grid.depth) {
                continue;
            }
            match grid.cell_at(x, y, roof_floor) {
                Some(c) if c.token == FootprintToken::Roof => {}
                _ => violations.push(AdjacencyViolation {
                    rule_id: "roof_perimeter_continuous",
                    message: format!("missing roof at ({x},{y})"),
                }),
            }
        }
    }

    // Rule 5 — no doors above ground; upper floors align as wall at door column.
    let door_columns: Vec<(u32, u32)> = grid
        .cells
        .iter()
        .filter(|c| c.floor == 0 && c.token == FootprintToken::Door)
        .map(|c| (c.x, c.y))
        .collect();
    for cell in &grid.cells {
        if cell.token == FootprintToken::Door && cell.floor > 0 {
            violations.push(AdjacencyViolation {
                rule_id: "door_ground_only",
                message: format!("door above ground at ({},{}) floor {}", cell.x, cell.y, cell.floor),
            });
        }
    }
    for (dx, dy) in door_columns {
        for floor in 1..grid.floors {
            match grid.cell_at(dx, dy, floor) {
                Some(c) if c.token == FootprintToken::Wall || c.token == FootprintToken::Corner => {}
                Some(c) => violations.push(AdjacencyViolation {
                    rule_id: "vertical_door_rhythm",
                    message: format!(
                        "floor {floor} at door column ({dx},{dy}) is {:?}, expected wall rhythm",
                        c.token
                    ),
                }),
                None => violations.push(AdjacencyViolation {
                    rule_id: "vertical_door_rhythm",
                    message: format!("missing cell at door column ({dx},{dy}) floor {floor}"),
                }),
            }
        }
    }

    violations
}

fn edge_door_match(cell: &FootprintCell, edge: &str, width: u32, depth: u32) -> bool {
    match edge {
        "north" => cell.y == 0,
        "south" => cell.y + 1 == depth,
        "west" => cell.x == 0,
        "east" => cell.x + 1 == width,
        _ => false,
    }
}

#[must_use]
pub fn bq_a1_adjacency_witness_green() -> bool {
    use super::types::{ProceduralBuildingRequest, StylePackId};

    let grid = FootprintGrid::from_request(&ProceduralBuildingRequest {
        archetype_id: "rect_perimeter".into(),
        width: 4,
        depth: 2,
        floors: 2,
        style: StylePackId("style_victorian".into()),
        seed: 1,
        arch_dna_preset_id: None,
    });
    check_footprint_adjacency(&grid).is_empty()
}

#[must_use]
pub fn build_bq_a1_adjacency_witness_body() -> serde_json::Value {
    use super::types::{ProceduralBuildingRequest, StylePackId};

    let grid = FootprintGrid::from_request(&ProceduralBuildingRequest {
        archetype_id: "rect_perimeter".into(),
        width: 4,
        depth: 2,
        floors: 2,
        style: StylePackId("style_victorian".into()),
        seed: 1,
        arch_dna_preset_id: None,
    });
    let violations = check_footprint_adjacency(&grid);
    let green = violations.is_empty();
    serde_json::json!({
        "gate": "BQ-A1-ADJ-001",
        "green": green,
        "violation_count": violations.len(),
        "rules": ["corner_two_neighbors", "door_not_corner", "one_door_per_edge", "roof_perimeter_continuous", "vertical_door_rhythm"],
        "violations": violations.iter().map(|v| serde_json::json!({
            "rule_id": v.rule_id,
            "message": v.message,
        })).collect::<Vec<_>>(),
        "plan_ref": "src/dev/plan_building_quality_v1.md#BQ-A1",
    })
}

#[must_use]
pub fn refresh_bq_a1_adjacency_witness() -> bool {
    use crate::dev::debug_run_envelope::{wrap_debug_run, write_debug_run_json};

    let body = build_bq_a1_adjacency_witness_body();
    let green = body.get("green").and_then(|v| v.as_bool()).unwrap_or(false);
    let wrapped = wrap_debug_run(
        "BQ-A1-ADJ-001",
        "refresh_bq_a1_adjacency_witness",
        BQ_A1_LIVE_JSON,
        body,
    );
    write_debug_run_json(BQ_A1_LIVE_JSON, wrapped) && green
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::construction::procedural::types::{ProceduralBuildingRequest, StylePackId};

    #[test]
    fn bq_a1_rect_perimeter_passes_five_rules() {
        let grid = FootprintGrid::from_request(&ProceduralBuildingRequest {
            archetype_id: "rect_perimeter".into(),
            width: 4,
            depth: 2,
            floors: 2,
            style: StylePackId("style_victorian".into()),
            seed: 1,
            arch_dna_preset_id: None,
        });
        assert!(
            check_footprint_adjacency(&grid).is_empty(),
            "{:?}",
            check_footprint_adjacency(&grid)
        );
    }

    #[test]
    fn bq_a1_witness_green_lib() {
        assert!(bq_a1_adjacency_witness_green());
    }

    #[test]
    fn bq_a1_refresh_witness_when_green() {
        if !bq_a1_adjacency_witness_green() {
            return;
        }
        assert!(refresh_bq_a1_adjacency_witness());
    }
}
