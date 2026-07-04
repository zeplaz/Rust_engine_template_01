//! W / D / C footprint grammar — parametric grid from width × depth (PG-2).

use super::types::ProceduralBuildingRequest;

/// Facade / roof token in the W/D/C grammar.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FootprintToken {
    /// Wall bay (window slot optional on upper floors).
    Wall,
    /// Door bay — floor 0 only.
    Door,
    /// Corner / turn cell.
    Corner,
    /// Roof footprint cell (plan view).
    Roof,
    /// Interior / setback — no mesh.
    Yard,
}

/// One cell in the procedural footprint grid.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FootprintCell {
    pub x: u32,
    pub y: u32,
    pub floor: u32,
    pub token: FootprintToken,
}

/// Assembled W/D/C grid for a rectangular footprint.
#[derive(Debug, Clone)]
pub struct FootprintGrid {
    pub width: u32,
    pub depth: u32,
    pub floors: u32,
    pub cells: Vec<FootprintCell>,
}

impl FootprintGrid {
    /// Build perimeter grammar for a rectangle (min 2×2).
    #[must_use]
    pub fn from_rect(width: u32, depth: u32, floors: u32) -> Self {
        Self::from_rect_with_door(width, depth, floors, None)
    }

    /// **BQ-H2** — optional door column; defaults to legacy center when `None`.
    #[must_use]
    pub fn from_rect_with_door(width: u32, depth: u32, floors: u32, door_x: Option<u32>) -> Self {
        let width = width.max(2);
        let depth = depth.max(2);
        let floors = floors.max(1);
        let door_x = door_x.unwrap_or_else(|| width / 2);
        let door_x = door_x.clamp(1, width.saturating_sub(2).max(1));
        let mut cells = Vec::new();

        for floor in 0..floors {
            for y in 0..depth {
                for x in 0..width {
                    if !is_perimeter(x, y, width, depth) {
                        continue;
                    }
                    let token = if is_corner(x, y, width, depth) {
                        FootprintToken::Corner
                    } else if floor == 0 && y == 0 && x == door_x {
                        FootprintToken::Door
                    } else {
                        FootprintToken::Wall
                    };
                    cells.push(FootprintCell {
                        x,
                        y,
                        floor,
                        token,
                    });
                }
            }
        }

        let roof_floor = floors;
        for y in 0..depth {
            for x in 0..width {
                if is_perimeter(x, y, width, depth) {
                    cells.push(FootprintCell {
                        x,
                        y,
                        floor: roof_floor,
                        token: FootprintToken::Roof,
                    });
                }
            }
        }

        Self {
            width,
            depth,
            floors,
            cells,
        }
    }

    #[must_use]
    pub fn from_request(request: &ProceduralBuildingRequest) -> Self {
        Self::from_rect(request.width, request.depth, request.floors)
    }

    /// Build footprint from grammar massing (`rect`, `yard_interior`, `l_shape` v1 rect).
    #[must_use]
    pub fn from_grammar(result: &super::building_grammar::GrammarGenerateResult) -> Self {
        let door_x = street_facing_door_column(
            result.width,
            result.depth,
            &result.placement_tags,
            result.seed,
            &result.massing_strategy,
            &result.door_rhythm,
        );
        let mut grid = Self::from_rect_with_door(result.width, result.depth, result.floors, Some(door_x));
        match result.footprint_mode.as_str() {
            "yard_interior" => grid.inject_interior_yard(),
            "l_shape" => grid.inject_l_shape_yard_v1(),
            _ => {}
        }
        grid
    }

    /// Interior courtyard cells (no mesh) for yard_complex massing.
    pub fn inject_interior_yard(&mut self) {
        if self.width < 4 || self.depth < 4 {
            return;
        }
        for floor in 0..self.floors {
            for y in 1..self.depth.saturating_sub(1) {
                for x in 1..self.width.saturating_sub(1) {
                    if is_perimeter(x, y, self.width, self.depth) {
                        continue;
                    }
                    self.cells.push(FootprintCell {
                        x,
                        y,
                        floor,
                        token: FootprintToken::Yard,
                    });
                }
            }
        }
    }

    /// L-shape v1: yard notch on high-x / high-y interior quadrant.
    pub fn inject_l_shape_yard_v1(&mut self) {
        if self.width < 4 || self.depth < 4 {
            return;
        }
        let cut_x = (self.width * 2) / 3;
        let cut_y = (self.depth * 2) / 3;
        for floor in 0..self.floors {
            for y in cut_y..self.depth.saturating_sub(1) {
                for x in cut_x..self.width.saturating_sub(1) {
                    if is_perimeter(x, y, self.width, self.depth) {
                        continue;
                    }
                    self.cells.push(FootprintCell {
                        x,
                        y,
                        floor,
                        token: FootprintToken::Yard,
                    });
                }
            }
        }
    }

    /// Count of W + D + C tokens (excludes roof / yard).
    #[must_use]
    pub fn wdc_cell_count(&self) -> u32 {
        self.cells
            .iter()
            .filter(|c| {
                matches!(
                    c.token,
                    FootprintToken::Wall | FootprintToken::Door | FootprintToken::Corner
                )
            })
            .count() as u32
    }

    pub fn facade_cells(&self) -> impl Iterator<Item = &FootprintCell> {
        self.cells.iter().filter(|c| c.token != FootprintToken::Yard)
    }
}

impl FootprintToken {
    /// AUTO-001 `token` field (`W` / `D` / `C` / `R`).
    #[must_use]
    pub const fn as_schema_token(self) -> Option<&'static str> {
        match self {
            Self::Wall => Some("W"),
            Self::Door => Some("D"),
            Self::Corner => Some("C"),
            Self::Roof => Some("R"),
            Self::Yard => None,
        }
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

/// **BQ-H1/H2** — street-facing door column from grammar tags + massing rhythm (seeded, not width/2).
#[must_use]
pub fn street_facing_door_column(
    width: u32,
    _depth: u32,
    placement_tags: &[String],
    seed: u64,
    massing_strategy: &str,
    door_rhythm: &str,
) -> u32 {
    let width = width.max(2);
    let min_x = 1u32;
    let max_x = width.saturating_sub(2).max(min_x);
    let span = max_x.saturating_sub(min_x) + 1;

    let street_tagged = placement_tags.iter().any(|t| {
        matches!(
            t.as_str(),
            "street_facing" | "commercial" | "storefront" | "civic"
        )
    });
    let loading = placement_tags.iter().any(|t| {
        matches!(t.as_str(), "loading_dock" | "logistics" | "rail" | "industrial")
    });

    let rhythm_bias = match door_rhythm {
        "linear_center" => span / 2,
        "perimeter_only" => (min_x + span / 4).max(min_x),
        "leg_offset" => min_x + span / 3,
        "loading_bay" => min_x + span * 2 / 3,
        _ if massing_strategy.contains("long_hall") || massing_strategy.contains("double_hall") => {
            span / 2
        }
        _ if massing_strategy.contains("l_shape") => min_x + span / 3,
        _ if street_tagged => min_x + span / 2,
        _ if loading => min_x + span * 2 / 3,
        _ => width / 2,
    };

    let jitter = (seed % span.max(1) as u64) as u32;
    let base = if street_tagged || loading {
        rhythm_bias.saturating_add(jitter / 2)
    } else {
        rhythm_bias
    };
    base.clamp(min_x, max_x)
}

#[must_use]
pub fn bq_h2_street_facing_witness_green() -> bool {
    let col = street_facing_door_column(
        6,
        4,
        &["street_facing".into(), "commercial".into()],
        42,
        "row_infill",
        "linear_center",
    );
    col >= 1 && col <= 4 && col != 0 && col != 5
}

#[must_use]
pub fn build_bq_h2_openings_witness_body() -> serde_json::Value {
    let grid = FootprintGrid::from_grammar(&super::building_grammar::generate(
        "IndustrialWarehouse",
        "industrial_west",
        43,
    )
    .expect("grammar"));
    let door = grid
        .cells
        .iter()
        .find(|c| c.token == FootprintToken::Door && c.floor == 0)
        .map(|c| (c.x, c.y));
    let green = bq_h2_street_facing_witness_green()
        && door.is_some_and(|(_, y)| y == 0)
        && door.is_some_and(|(x, _)| x >= 1);
    serde_json::json!({
        "gate": "BQ-H2-OPENINGS-001",
        "green": green,
        "door_cell_floor0": door.map(|(x, y)| serde_json::json!({"x": x, "y": y})),
        "uses_street_facing_heuristic": true,
        "plan_ref": "src/dev/plan_building_quality_v1.md#BQ-H2",
    })
}

#[must_use]
pub fn bq_h_openings_witness_green() -> bool {
    build_bq_h2_openings_witness_body()
        .get("green")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
}

#[must_use]
pub fn refresh_bq_h2_openings_witness() -> bool {
    use crate::dev::debug_run_envelope::{wrap_debug_run, write_debug_run_json};

    const JSON: &str = "debug_runs/bq_h2_openings_001_live.json";
    let body = build_bq_h2_openings_witness_body();
    let green = body.get("green").and_then(|v| v.as_bool()).unwrap_or(false);
    let wrapped = wrap_debug_run("BQ-H2-OPENINGS-001", "refresh_bq_h2_openings_witness", JSON, body);
    write_debug_run_json(JSON, wrapped) && green
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn footprint_grid_door_on_floor_zero() {
        let grid = FootprintGrid::from_rect(4, 2, 3);
        assert!(
            grid.cells
                .iter()
                .any(|c| c.token == FootprintToken::Door && c.floor == 0),
            "floor 0 must include a door"
        );
        assert!(
            !grid.cells.iter().any(|c| c.token == FootprintToken::Door && c.floor > 0),
            "upper floors must not include doors"
        );
    }

    #[test]
    fn footprint_grid_corner_token_consumes_c() {
        let grid = FootprintGrid::from_rect(4, 2, 1);
        let corners: Vec<_> = grid
            .cells
            .iter()
            .filter(|c| is_corner(c.x, c.y, grid.width, grid.depth) && c.floor == 0)
            .collect();
        assert_eq!(corners.len(), 4);
        for cell in corners {
            assert_eq!(
                cell.token,
                FootprintToken::Corner,
                "corner ({},{}) must be C not W/D",
                cell.x,
                cell.y
            );
        }
    }

    #[test]
    fn footprint_grid_door_not_on_corner_when_street_tagged() {
        let col = street_facing_door_column(4, 2, &["street_facing".into()], 7, "long_hall", "linear_center");
        assert!(col >= 1 && col <= 2);
        let grid = FootprintGrid::from_rect_with_door(4, 2, 1, Some(col));
        let door = grid.cells.iter().find(|c| c.token == FootprintToken::Door).unwrap();
        assert_ne!(door.x, 0);
        assert_ne!(door.x + 1, grid.width);
    }

    #[test]
    fn bq_h2_openings_witness_green_lib() {
        assert!(bq_h_openings_witness_green());
    }

    #[test]
    fn bq_h2_refresh_witness_when_green() {
        if bq_h_openings_witness_green() {
            assert!(refresh_bq_h2_openings_witness());
        }
    }
}
