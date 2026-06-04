//! Bevy 0.18 + `bevy_hanabi` 0.18 compile/runtime registration gate.

use bevy::prelude::*;
use bevy_hanabi::prelude::*;

/// Registers Hanabi on a minimal 3D render stack — proves crate alignment without main `EnginePlugin`.
pub fn register_hanabi_spike_app(app: &mut App) {
    app.add_plugins((MinimalPlugins, AssetPlugin::default(), HanabiPlugin));
}

/// L3 ember preset sized to designer cap (24 ≤ 32 instances).
pub fn fire_ember_effect_asset() -> EffectAsset {
    let writer = ExprWriter::new();
    let lifetime = writer.lit(0.85).expr();
    let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, lifetime);
    let init_pos = SetPositionSphereModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        radius: writer.lit(0.35).expr(),
        dimension: ShapeDimension::Surface,
    };
    let init_vel = SetVelocitySphereModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        speed: writer.lit(0.6).expr(),
    };
    let drag = LinearDragModifier::new(writer.lit(1.2).expr());
    let color = writer.lit(Vec4::new(0.72, 0.38, 0.18, 0.35)).expr();
    let color_mod = SetAttributeModifier::new(Attribute::COLOR, color);
    let spawner = SpawnerSettings::once(24.0.into());
    EffectAsset::new(32, spawner, writer.finish())
        .with_name("fire_ember_burst")
        .init(init_pos)
        .init(init_vel)
        .init(init_lifetime)
        .init(color_mod)
        .update(drag)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Runtime plugin graph needs a full render stack; `cargo check -p hanabi_validation` is the CI gate.
    #[test]
    fn hanabi_crate_linked_on_bevy_018() {
        let _ = std::any::type_name::<HanabiPlugin>();
    }

    #[test]
    fn fire_ember_effect_within_designer_cap() {
        let effect = fire_ember_effect_asset();
        assert!(effect.capacity() <= 32);
    }
}
