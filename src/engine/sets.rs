use bevy::prelude::SystemSet;

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub enum Nav_Sets {
    MotionCalculation,
    DamageSpeedAdjustment,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
enum GameSystemSet {
    TerrainGeneration,
    RegionGeneration,
    AgentGeneration,
    AgentRelationships,
    Simulation,
}

/*
dont forget to add these to systems!

.add_system(
    vehicle_damage_speed_system
        .system()
        .in_set(CustomSystemSets::DamageSpeedAdjustment),
)
.configure_sets(|app| {
    app.configure_set(
        CustomSystemSets::MotionCalculation,
        |set| set.after(CustomSystemSets::DamageSpeedAdjustment),
    );
})
*/
/*// remeber that sets are to be configured: can can be complex orders::
app.configure_sets(
    (
        MySet::BeforeRound,
        MySystem::ComputeForces,
        MySystem::FindCollisions,
        MySet::AfterRound,
    ).chain()
);

app.add_system(my_system.in_base_set(MySet::BeforeRound))
   .add_system(another_system.in_base_set(MySet::AfterRound));
*/
