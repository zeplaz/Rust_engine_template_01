//! Dam footprint helpers for the DamFlood scenario (channel-centered span).

use crate::strategic::{BuildSiteTile, FootprintTiles};

/// Origin + footprint so a dam **centers** on the channel and spans both banks.
///
/// Footprints expand +X/+Z from origin — placing at the narrows center only covers
/// the east/south half of the river (the janky playcheck dam).
#[must_use]
pub fn dam_channel_span_footprint(
    narrows_xz: (i32, i32),
    span_tiles: u32,
    depth_tiles: u32,
) -> (BuildSiteTile, FootprintTiles) {
    let span = span_tiles.max(4);
    let depth = depth_tiles.max(1);
    let half_x = (span / 2) as i32;
    let half_z = (depth / 2) as i32;
    let ox = (narrows_xz.0 - half_x).max(0) as u32;
    let oz = (narrows_xz.1 - half_z).max(0) as u32;
    (
        BuildSiteTile { x: ox, z: oz },
        FootprintTiles {
            width: span,
            depth,
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dam_span_centers_on_narrows() {
        let (origin, fp) = dam_channel_span_footprint((160, 160), 12, 2);
        assert_eq!(origin.x, 154);
        assert_eq!(origin.z, 159);
        assert_eq!(fp.width, 12);
        assert_eq!(fp.depth, 2);
        // Covers channel x=160 and both banks.
        assert!(origin.x <= 160 && origin.x + fp.width > 160);
    }
}
