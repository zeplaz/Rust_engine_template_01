//! **Deterministic randomness for AI decisions** (FAI-2). No `thread_rng`, no clock: every roll is a
//! pure function of `(session seed, faction slot, sim tick, salt)`, so the same session replays the
//! same decisions.

use bevy::prelude::*;

use crate::terrain::generation::world_generator_enhanced::WorldGenParams;

/// Optional explicit session seed for AI rolls. `0` means "derive from [`WorldGenParams::seed`]".
#[derive(Resource, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AiSeed(pub u64);

/// Salt for the economy brain's placement jitter.
pub const SALT_ECONOMY: u64 = 0x0EC0_0001;
/// Salt for the military brain.
pub const SALT_MILITARY: u64 = 0x0A11_0002;
/// Salt for the world-intent resolver (`hybrid_resolve_and_feedback_system`).
pub const SALT_HYBRID: u64 = 0x4B1D_0003;
/// Faction key used by rolls that belong to no faction.
pub const NO_FACTION: u8 = u8::MAX;

/// Session seed: explicit [`AiSeed`] when non-zero, else the world-gen seed, else 0.
#[must_use]
pub fn session_seed(ai: Option<&AiSeed>, world: Option<&WorldGenParams>) -> u64 {
    ai.map(|s| s.0)
        .filter(|s| *s != 0)
        .or_else(|| world.map(|w| w.seed))
        .unwrap_or(0)
}

/// SplitMix64 finalizer — a well-mixed bijection on `u64`.
#[inline]
#[must_use]
pub const fn mix64(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9E37_79B9_7F4A_7C15);
    x = (x ^ (x >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    x ^ (x >> 31)
}

/// Seed for one decision of one faction at one tick.
#[inline]
#[must_use]
pub const fn decision_seed(session: u64, slot: u8, tick: u64, salt: u64) -> u64 {
    mix64(mix64(mix64(session ^ salt) ^ (slot as u64)) ^ tick)
}

/// Tiny deterministic generator (SplitMix64 stream).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DetRng(u64);

impl DetRng {
    #[must_use]
    pub const fn new(seed: u64) -> Self {
        Self(seed)
    }

    /// Generator for one decision of one faction.
    #[must_use]
    pub const fn for_decision(session: u64, slot: u8, tick: u64, salt: u64) -> Self {
        Self(decision_seed(session, slot, tick, salt))
    }

    pub fn next_u64(&mut self) -> u64 {
        let out = mix64(self.0);
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        out
    }

    /// Uniform in `[0, 1)`.
    pub fn next_f32(&mut self) -> f32 {
        ((self.next_u64() >> 40) as f32) / (1u64 << 24) as f32
    }

    /// Uniform in `[0, n)`; `0` when `n == 0`.
    pub fn next_below(&mut self, n: u32) -> u32 {
        if n == 0 {
            0
        } else {
            (self.next_u64() % u64::from(n)) as u32
        }
    }
}

/// The world-intent resolver's roll for one tick (replaces `rand::thread_rng().gen()`).
#[must_use]
pub fn hybrid_roll(session: u64, tick: u64) -> f32 {
    DetRng::for_decision(session, NO_FACTION, tick, SALT_HYBRID).next_f32()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rolls_replay_for_the_same_inputs() {
        let a = DetRng::for_decision(7, 1, 100, SALT_ECONOMY).next_u64();
        let b = DetRng::for_decision(7, 1, 100, SALT_ECONOMY).next_u64();
        assert_eq!(a, b);
    }

    #[test]
    fn rolls_differ_by_seed_slot_tick_and_salt() {
        let base = decision_seed(7, 1, 100, SALT_ECONOMY);
        assert_ne!(base, decision_seed(8, 1, 100, SALT_ECONOMY));
        assert_ne!(base, decision_seed(7, 2, 100, SALT_ECONOMY));
        assert_ne!(base, decision_seed(7, 1, 101, SALT_ECONOMY));
        assert_ne!(base, decision_seed(7, 1, 100, SALT_MILITARY));
    }

    #[test]
    fn unit_rolls_stay_in_range_and_spread() {
        let mut rng = DetRng::new(42);
        let xs: Vec<f32> = (0..1000).map(|_| rng.next_f32()).collect();
        assert!(xs.iter().all(|x| (0.0..1.0).contains(x)));
        let mean = xs.iter().sum::<f32>() / xs.len() as f32;
        assert!((mean - 0.5).abs() < 0.05, "mean {mean}");
        assert_eq!(DetRng::new(1).next_below(0), 0);
    }

    #[test]
    fn hybrid_resolve_is_seeded() {
        assert_eq!(hybrid_roll(11, 5), hybrid_roll(11, 5));
        assert_ne!(hybrid_roll(11, 5), hybrid_roll(11, 6));
        assert_ne!(hybrid_roll(11, 5), hybrid_roll(12, 5));
    }

    #[test]
    fn explicit_seed_wins_then_world_seed() {
        assert_eq!(session_seed(Some(&AiSeed(9)), None), 9);
        assert_eq!(session_seed(Some(&AiSeed(0)), None), 0);
        assert_eq!(session_seed(None, None), 0);
    }
}
