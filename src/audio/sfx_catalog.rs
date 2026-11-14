//! Tag → `AudioSource` handles. Cue ids map to N variant tags (AUD-03).

use std::collections::HashMap;

use bevy::prelude::*;

/// Maps variant tags and cue ids → handles / variant lists.
#[derive(Resource, Debug)]
pub struct SfxCatalog {
    pub by_tag: HashMap<&'static str, Handle<AudioSource>>,
    cues: HashMap<&'static str, &'static [&'static str]>,
}

impl Default for SfxCatalog {
    fn default() -> Self {
        Self {
            by_tag: HashMap::new(),
            cues: HashMap::new(),
        }
    }
}

impl SfxCatalog {
    #[must_use]
    pub fn get(&self, tag: &str) -> Option<&Handle<AudioSource>> {
        self.by_tag.get(tag)
    }

    pub fn insert(&mut self, tag: &'static str, handle: Handle<AudioSource>) {
        self.by_tag.insert(tag, handle);
    }

    /// Register one logical cue id behind `variant_tags` (≥ 1).
    pub fn register_cue(&mut self, cue_id: &'static str, variant_tags: &'static [&'static str]) {
        debug_assert!(!variant_tags.is_empty(), "cue {cue_id} needs ≥1 variant");
        self.cues.insert(cue_id, variant_tags);
    }

    #[must_use]
    pub fn cue_variants(&self, cue_id: &str) -> Option<&'static [&'static str]> {
        self.cues.get(cue_id).copied()
    }

    /// Resolve a mix `tag` (cue id or bare variant) to a concrete variant tag for playback.
    #[must_use]
    pub fn resolve_play_tag(
        &self,
        cue_or_variant: &'static str,
        picker: &mut AudioCuePicker,
    ) -> &'static str {
        let Some(variants) = self.cue_variants(cue_or_variant) else {
            return cue_or_variant;
        };
        let idx = picker.pick_variant_index(cue_or_variant, variants.len());
        variants[idx]
    }
}

/// Presentation-only RNG salt — never mixed with sim / AI decision seeds.
const AUDIO_LOCAL_RNG_SALT: u64 = 0xAD10_C0DE_5EED;

/// SplitMix64 step (local copy — audio must not depend on sim `ai::faction::rng`).
#[inline]
const fn mix64(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9E37_79B9_7F4A_7C15);
    x = (x ^ (x >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    x ^ (x >> 31)
}

/// Audio-local RNG stream (not tick- or faction-scoped).
#[derive(Clone, Debug)]
pub struct AudioLocalRng {
    state: u64,
}

impl AudioLocalRng {
    #[must_use]
    pub const fn new(presentation_seed: u64) -> Self {
        Self {
            state: mix64(presentation_seed ^ AUDIO_LOCAL_RNG_SALT),
        }
    }

    pub fn next_u64(&mut self) -> u64 {
        let out = mix64(self.state);
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        out
    }

    /// Uniform index in `[0, n)`; returns `0` when `n == 0`.
    pub fn next_below(&mut self, n: usize) -> usize {
        if n == 0 {
            0
        } else {
            (self.next_u64() as usize) % n
        }
    }
}

/// Per-cue last variant + audio-local RNG — sole writer: playback resolve path.
#[derive(Resource, Clone, Debug)]
pub struct AudioCuePicker {
    rng: AudioLocalRng,
    last_index: HashMap<&'static str, usize>,
}

impl AudioCuePicker {
    #[must_use]
    pub fn new(presentation_seed: u64) -> Self {
        Self {
            rng: AudioLocalRng::new(presentation_seed),
            last_index: HashMap::new(),
        }
    }

    /// Pick variant index for `cue_id`; never repeats the previous index when `n ≥ 2`.
    pub fn pick_variant_index(&mut self, cue_id: &'static str, n: usize) -> usize {
        if n <= 1 {
            let idx = 0;
            self.last_index.insert(cue_id, idx);
            return idx;
        }
        let mut idx = self.rng.next_below(n);
        if self.last_index.get(cue_id) == Some(&idx) {
            idx = (idx + 1) % n;
        }
        self.last_index.insert(cue_id, idx);
        idx
    }
}

/// Built-in tags the mix emits today — register assets under these keys.
pub mod tags {
    pub const AMBIENT_RAIN: &str = "ambient.rain";
    pub const AMBIENT_WIND: &str = "ambient.wind";
    pub const SPATIAL_NEAR: &str = "spatial.near_field";
    pub const UI_CLICK: &str = "ui.click";
    pub const UI_CONFIRM: &str = "ui.confirm";
    pub const UI_DENY: &str = "ui.deny";
    pub const SHOT: &str = "combat.shot";
    pub const EXPLOSION: &str = "combat.explosion";
    pub const MISSILE_LAUNCH: &str = "combat.missile_launch";
    pub const LAUNCH_WARNING: &str = "warning.launch";
    pub const WEATHER_ALERT: &str = "warning.weather";

    pub const SHOT_V0: &str = "combat.shot.v0";
    pub const SHOT_V1: &str = "combat.shot.v1";
    pub const SHOT_V2: &str = "combat.shot.v2";

    pub const EXPLOSION_V0: &str = "combat.explosion.v0";
    pub const EXPLOSION_V1: &str = "combat.explosion.v1";

    pub const UI_CLICK_V0: &str = "ui.click.v0";
    pub const UI_CLICK_V1: &str = "ui.click.v1";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn variant_never_immediate_repeat_for_multi_variant_cue() {
        let mut catalog = SfxCatalog::default();
        catalog.register_cue(tags::SHOT, &[tags::SHOT_V0, tags::SHOT_V1, tags::SHOT_V2]);
        let mut picker = AudioCuePicker::new(0xDEAD_BEEF);
        let mut prev: Option<&str> = None;
        for _ in 0..128 {
            let tag = catalog.resolve_play_tag(tags::SHOT, &mut picker);
            if let Some(p) = prev {
                assert_ne!(p, tag, "immediate repeat of variant {tag}");
            }
            prev = Some(tag);
        }
    }

    #[test]
    fn single_variant_cue_is_stable() {
        let mut catalog = SfxCatalog::default();
        catalog.register_cue(tags::UI_DENY, &[tags::UI_DENY]);
        let mut picker = AudioCuePicker::new(1);
        for _ in 0..8 {
            assert_eq!(catalog.resolve_play_tag(tags::UI_DENY, &mut picker), tags::UI_DENY);
        }
    }
}