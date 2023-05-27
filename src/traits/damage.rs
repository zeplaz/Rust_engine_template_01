use crate::systems::damage::DamageState;

pub trait DamageInfoProvider {
    type DamageInfo;
    fn get_damage_info(&self) -> &Self::DamageInfo;
}

pub trait TakesDamage {
    fn apply_damage(&mut self, amount: f32);
    fn repair(&mut self, amount: f32);
    fn get_structural_integrity(&self) -> f32;
    fn damage_state(&self) -> &DamageState;
}

pub trait DealsDamage {
    // DealsDamage trait methods
}
