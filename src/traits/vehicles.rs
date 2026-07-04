//! **CLN-P0-T4-001** — live trait; impls in `systems/navigation/road_vehicles_motion.rs`.

pub trait LoadBasedSpeedModifier {
    fn speed_modifier(&self) -> f32;
}
