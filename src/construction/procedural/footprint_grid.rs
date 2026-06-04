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
        let width = width.max(2);
        let depth = depth.max(2);
        let floors = floors.max(1);
        let door_x = width / 2;
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
        let mut grid = Self::from_rect(result.width, result.depth, result.floors);
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
}
