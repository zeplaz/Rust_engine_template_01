//! Engine UI primitives replacing egui Color32/Pos2/Rect/Stroke.
use bevy::prelude::{Color, Rect, Vec2};
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct UiRgba { pub r: u8, pub g: u8, pub b: u8, pub a: u8 }
impl UiRgba {
    pub const TRANSPARENT: Self = Self { r: 0, g: 0, b: 0, a: 0 };
    pub const WHITE: Self = Self { r: 255, g: 255, b: 255, a: 255 };
    pub const BLACK: Self = Self { r: 0, g: 0, b: 0, a: 255 };
    pub const RED: Self = Self { r: 255, g: 0, b: 0, a: 255 };
    pub const GREEN: Self = Self { r: 0, g: 255, b: 0, a: 255 };
    pub const YELLOW: Self = Self { r: 255, g: 255, b: 0, a: 255 };
    pub const GRAY: Self = Self { r: 128, g: 128, b: 128, a: 255 };
    pub const LIGHT_RED: Self = Self { r: 255, g: 128, b: 128, a: 255 };
    pub const fn from_rgb(r: u8, g: u8, b: u8) -> Self { Self { r, g, b, a: 255 } }
    pub const fn from_rgba_unmultiplied(r: u8, g: u8, b: u8, a: u8) -> Self { Self { r, g, b, a } }
    pub fn to_bevy_color(self) -> Color {
        Color::srgba(self.r as f32 / 255.0, self.g as f32 / 255.0, self.b as f32 / 255.0, self.a as f32 / 255.0)
    }
    pub fn gamma_multiply(self, factor: f32) -> Self {
        let f = factor.clamp(0.0, 1.0);
        Self { r: (self.r as f32 * f) as u8, g: (self.g as f32 * f) as u8, b: (self.b as f32 * f) as u8, a: self.a }
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UiStroke { pub width: f32, pub color: UiRgba }
impl UiStroke {
    pub const fn new(width: f32, color: UiRgba) -> Self { Self { width, color } }
}
pub type UiPos2 = Vec2;
pub type UiRect = Rect;
pub fn ui_pos2(x: f32, y: f32) -> UiPos2 { Vec2::new(x, y) }
