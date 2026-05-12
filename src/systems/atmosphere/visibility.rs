//! Line-of-sight style sampling on [`super::field::AtmosphereField`] (`base_fire2_smoke.md` §8).

use bevy::prelude::*;

use super::field::AtmosphereField;

/// Sample `visibility` along segment `(a,b)` in **field tile space** (same space as [`AtmosphereField::chunk_to_tile`] origin).
/// Returns multiplicative factor in `[0, 1]` — multiply with geometric clear-LOS as needed.
pub fn visibility_between(a: Vec2, b: Vec2, field: &AtmosphereField) -> f32 {
    let dx = b.x - a.x;
    let dy = b.y - a.y;
    let dist = (dx * dx + dy * dy).sqrt();
    if dist <= 1e-5 {
        return sample_visibility_at(field, a.x, a.y);
    }
    let steps = (dist.ceil() as i32).clamp(3, 48);
    let mut acc = 1.0f32;
    for i in 0..=steps {
        let t = i as f32 / steps as f32;
        let x = a.x + dx * t;
        let y = a.y + dy * t;
        acc *= sample_visibility_at(field, x, y);
    }
    acc.clamp(0.0, 1.0)
}

fn sample_visibility_at(field: &AtmosphereField, wx: f32, wy: f32) -> f32 {
    let xi = wx.floor() as i32;
    let yi = wy.floor() as i32;
    if xi < 0 || yi < 0 {
        return 1.0;
    }
    let xu = xi as u32;
    let yu = yi as u32;
    let Some(i) = field.cell_index(xu, yu) else {
        return 1.0;
    };
    field.cells[i].visibility.clamp(0.05, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::field::{AtmosphereCell, AtmosphereField};

    #[test]
    fn uniform_clear_field_high_visibility() {
        let mut f = AtmosphereField::default();
        for c in &mut f.cells {
            c.visibility = 1.0;
        }
        let v = visibility_between(Vec2::ZERO, Vec2::new(10.0, 0.0), &f);
        assert!(v > 0.99);
    }

    #[test]
    fn opaque_band_reduces_visibility() {
        let mut f = AtmosphereField::default();
        for c in &mut f.cells {
            c.visibility = 1.0;
        }
        for x in 4..8 {
            let i = f.idx(x, 0);
            f.cells[i] = AtmosphereCell {
                visibility: 0.2,
                ..Default::default()
            };
        }
        let v = visibility_between(Vec2::new(2.0, 0.5), Vec2::new(10.0, 0.5), &f);
        assert!(v < 0.95);
    }
}
