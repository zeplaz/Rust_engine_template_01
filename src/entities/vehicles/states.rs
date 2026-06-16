//! Road vehicle lifecycle states — reserved for motion FSM (not wired to runtime yet).

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RoadVehicleState {
    #[default]
    Idle,
    Moving,
    Loading,
}
