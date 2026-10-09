//! Per-cell fuel, ignition admission, neighbor spread, and a fixed burned-tick hold.
//!
//! Sole callers: [`super::chunk_fire_overlay::chunk_fire_overlay_tick`] writes the overlay.
//! Deterministic: no RNG. Rain does not refill fuel.

use super::combustion::{fuel_ignition_gate, MIN_CELL_FUEL_FOR_SPREAD};

/// Ticks a just-depleted cell stays at residual heat.
pub const BURNED_HOLD_TICKS: u16 = 6;

/// Heat floor while [`BURNED_HOLD_TICKS`] remains. Below the neighbor-spread source gate.
pub const BURNED_RESIDUAL_HEAT: f32 = 0.05;

/// Source heat required before cardinal spread.
pub const SPREAD_HEAT_MIN: f32 = 0.08;

/// Heat added to an eligible neighbor (one cardinal step, snapshot order).
pub const NEIGHBOR_HEAT_GAIN: f32 = 0.04;

/// Fuel drawn from that neighbor. Does not refill the source.
pub const NEIGHBOR_FUEL_DRAW: f32 = 0.02;

/// Snow at or above this blocks spread and clears a burned hold.
pub const SNOW_BLOCKS_SPREAD: f32 = 0.35;

/// Cell moisture at or above this refuses a neighbor deposit.
pub const MOISTURE_BLOCKS_SPREAD: f32 = 0.72;

/// Chunk rain at or above this stops cardinal deposits. It does not refill fuel.
pub const RAIN_BLOCKS_SPREAD: f32 = 0.85;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FireCell {
    pub heat: f32,
    pub fuel: f32,
    pub burned_hold: u16,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FuelStep {
    pub fuel: f32,
    /// Fuel crossed from spread-capable to below [`MIN_CELL_FUEL_FOR_SPREAD`].
    pub crossed_below_spread: bool,
    /// Crossed while hot enough to arm the burned window.
    pub arm_hold: bool,
}

/// Ambient spark is zero when wildland mass is under the gate or the cell cannot spread.
#[must_use]
pub fn admit_spark(wildland_mass: f32, cell_fuel: f32, raw_spark: f32) -> f32 {
    if fuel_ignition_gate(wildland_mass) <= 0.0 || cell_fuel < MIN_CELL_FUEL_FOR_SPREAD {
        0.0
    } else {
        raw_spark.max(0.0)
    }
}

/// Consume fuel with heat. `rain_intensity` is ignored — rain never refills the reservoir.
#[must_use]
pub fn step_cell_fuel(fuel: f32, heat: f32, dt: f32, rain_intensity: f32) -> FuelStep {
    if dt <= 0.0 {
        return FuelStep {
            fuel: fuel.max(0.0),
            crossed_below_spread: false,
            arm_hold: false,
        };
    }
    let _rain = rain_intensity;
    let burn = heat.max(0.0) * 0.012 * dt.max(0.0);
    let next = (fuel - burn).max(0.0);
    let crossed = fuel >= MIN_CELL_FUEL_FOR_SPREAD && next < MIN_CELL_FUEL_FOR_SPREAD;
    FuelStep {
        fuel: next,
        crossed_below_spread: crossed,
        arm_hold: crossed && heat >= SPREAD_HEAT_MIN,
    }
}

/// Extra fuel wind removes after the still-air step. Still air (`wind_speed` ≤ 0) adds none.
#[must_use]
pub fn wind_fuel_delta(heat: f32, dt: f32, wind_speed: f32) -> f32 {
    if dt <= 0.0 {
        return 0.0;
    }
    heat.max(0.0) * 0.012 * dt * wind_speed.clamp(0.0, 1.0) * 0.5
}

/// Wind speeds fuel use. Still air matches [`step_cell_fuel`]. Rain still does not refill.
#[must_use]
pub fn fuel_step_in_wind(
    fuel: f32,
    heat: f32,
    dt: f32,
    rain_intensity: f32,
    wind_speed: f32,
) -> FuelStep {
    let still = step_cell_fuel(fuel, heat, dt, rain_intensity);
    let extra = wind_fuel_delta(heat, dt, wind_speed);
    if extra <= 0.0 {
        return still;
    }
    let next = (still.fuel - extra).max(0.0);
    let crossed = fuel >= MIN_CELL_FUEL_FOR_SPREAD && next < MIN_CELL_FUEL_FOR_SPREAD;
    FuelStep {
        fuel: next,
        crossed_below_spread: crossed,
        arm_hold: crossed && heat >= SPREAD_HEAT_MIN,
    }
}

/// One tick of the burned window. Forces residual heat and decrements. Does not restore fuel.
pub fn step_burn_hold(cell: &mut FireCell) {
    if cell.burned_hold == 0 {
        return;
    }
    cell.heat = cell.heat.max(BURNED_RESIDUAL_HEAT).min(1.0);
    cell.fuel = cell.fuel.max(0.0);
    cell.burned_hold -= 1;
}

/// Deposit heat on a fueled neighbor and draw its fuel.
/// A source that is only residual-burn (empty fuel or still inside the hold) does not spread.
/// A neighbor already inside its burned window does not accept the deposit.
#[cfg(test)]
#[must_use]
pub fn spread_onto_neighbor(src: &FireCell, dst: &mut FireCell) -> bool {
    if src.burned_hold > 0 || src.fuel < MIN_CELL_FUEL_FOR_SPREAD || src.heat < SPREAD_HEAT_MIN {
        return false;
    }
    if dst.burned_hold > 0 || dst.fuel < MIN_CELL_FUEL_FOR_SPREAD {
        return false;
    }
    spread_with_gain(src, dst, NEIGHBOR_HEAT_GAIN)
}

/// Heat deposited on a neighbor. Fog at 1.0 cuts the gain in half. Fuel draw is unchanged.
#[must_use]
pub fn scaled_neighbor_heat_gain(fog_density: f32) -> f32 {
    NEIGHBOR_HEAT_GAIN * (1.0 - fog_density.clamp(0.0, 1.0) * 0.5)
}

/// Same gates as [`spread_onto_neighbor`], with an explicit heat gain.
#[must_use]
pub fn spread_with_gain(src: &FireCell, dst: &mut FireCell, heat_gain: f32) -> bool {
    if heat_gain <= 0.0 {
        return false;
    }
    if src.burned_hold > 0 || src.fuel < MIN_CELL_FUEL_FOR_SPREAD || src.heat < SPREAD_HEAT_MIN {
        return false;
    }
    if dst.burned_hold > 0 || dst.fuel < MIN_CELL_FUEL_FOR_SPREAD || dst.heat >= 1.0 {
        return false;
    }
    dst.heat = (dst.heat + heat_gain).min(1.0);
    dst.fuel = (dst.fuel - NEIGHBOR_FUEL_DRAW).max(0.0);
    true
}

/// Lightning is an extra spark only when the wildland gate and the cell fuel floor already allow it.
#[must_use]
pub fn lightning_spark(wildland_mass: f32, cell_fuel: f32, lightning_risk: f32) -> f32 {
    let risk = lightning_risk.clamp(0.0, 1.0);
    if risk <= 0.0 {
        return 0.0;
    }
    admit_spark(wildland_mass, cell_fuel, risk * 0.05)
}

/// A hold arms only when fuel just crossed down and snow is not already extinguishing the cell.
#[must_use]
pub fn should_arm_burn_hold(arm_hold: bool, snow_depth: f32) -> bool {
    arm_hold && snow_depth < SNOW_BLOCKS_SPREAD
}

/// Snow cover or a wet destination cell blocks the cardinal deposit.
#[must_use]
pub fn spread_allowed(snow_depth: f32, dest_moisture: f32) -> bool {
    snow_depth < SNOW_BLOCKS_SPREAD && dest_moisture < MOISTURE_BLOCKS_SPREAD
}

/// Heavy rain stops cardinal spread. Light rain does not. Fuel is untouched.
#[must_use]
pub fn rain_blocks_neighbor_spread(rain_intensity: f32) -> bool {
    rain_intensity >= RAIN_BLOCKS_SPREAD
}

/// Spark below this does not ignite a cell sitting on the fuel floor.
pub const EMBER_FLOOR_SPARK_MIN: f32 = 0.03;

/// An ember may heat a cell only while that cell still has spread fuel.
/// A weak spark does not ignite a cell that is exactly on the floor.
#[must_use]
pub fn ember_heat_deposit(cell_fuel: f32, spark: f32) -> f32 {
    if cell_fuel < MIN_CELL_FUEL_FOR_SPREAD || spark <= 0.0 {
        0.0
    } else if cell_fuel <= MIN_CELL_FUEL_FOR_SPREAD + 1e-6 && spark < EMBER_FLOOR_SPARK_MIN {
        0.0
    } else {
        spark
    }
}

/// One neighbor accepts one cardinal deposit per tick. A second neighbor does not stack.
#[must_use]
pub fn take_neighbor_deposit(
    already_taken: bool,
    src: &FireCell,
    dst: &mut FireCell,
    heat_gain: f32,
) -> bool {
    if already_taken {
        return false;
    }
    spread_with_gain(src, dst, heat_gain)
}

/// Deep snow ends the burned window and drops residual heat on an empty cell.
pub fn apply_snow_to_hold(cell: &mut FireCell, snow_depth: f32) {
    if snow_depth < SNOW_BLOCKS_SPREAD {
        return;
    }
    cell.burned_hold = 0;
    if cell.fuel < MIN_CELL_FUEL_FOR_SPREAD {
        cell.heat = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::systems::fire::combustion::MIN_WILDLAND_FUEL_MASS;

    #[test]
    fn sweep_spark_below_wildland_mass_is_zero() {
        let below = MIN_WILDLAND_FUEL_MASS - 0.01;
        assert_eq!(admit_spark(below, 0.4, 0.05), 0.0);
        assert_eq!(fuel_ignition_gate(below), 0.0);
        let above = MIN_WILDLAND_FUEL_MASS + 0.2;
        assert!(admit_spark(above, 0.4, 0.05) > 0.0);
    }

    #[test]
    fn sweep_spark_below_cell_fuel_is_zero() {
        let mass = MIN_WILDLAND_FUEL_MASS + 0.4;
        assert_eq!(
            admit_spark(mass, MIN_CELL_FUEL_FOR_SPREAD - 0.01, 0.07),
            0.0
        );
        assert!(admit_spark(mass, MIN_CELL_FUEL_FOR_SPREAD, 0.07) > 0.0);
    }

    #[test]
    fn sweep_spread_deposits_heat_and_draws_neighbor_fuel() {
        let src = FireCell {
            heat: 0.4,
            fuel: 0.3,
            burned_hold: 0,
        };
        let mut dst = FireCell {
            heat: 0.0,
            fuel: 0.2,
            burned_hold: 0,
        };
        assert!(spread_onto_neighbor(&src, &mut dst));
        assert!((dst.heat - NEIGHBOR_HEAT_GAIN).abs() < 1e-6);
        assert!((dst.fuel - (0.2 - NEIGHBOR_FUEL_DRAW)).abs() < 1e-6);
        assert!((src.fuel - 0.3).abs() < 1e-6);
    }

    #[test]
    fn sweep_burned_cell_holds_residual_for_fixed_tick_window() {
        let mut cell = FireCell {
            heat: 0.4,
            fuel: MIN_CELL_FUEL_FOR_SPREAD,
            burned_hold: 0,
        };
        let stepped = step_cell_fuel(cell.fuel, cell.heat, 10.0, 0.9);
        cell.fuel = stepped.fuel;
        assert!(stepped.arm_hold);
        assert!(cell.fuel < MIN_CELL_FUEL_FOR_SPREAD);
        cell.burned_hold = BURNED_HOLD_TICKS;
        for tick in 0..BURNED_HOLD_TICKS {
            cell.heat = 0.0;
            let fuel_before = cell.fuel;
            step_burn_hold(&mut cell);
            assert!(
                cell.heat + 1e-6 >= BURNED_RESIDUAL_HEAT,
                "tick {tick} dropped residual"
            );
            assert!((cell.fuel - fuel_before).abs() < 1e-6);
        }
        assert_eq!(cell.burned_hold, 0);
        cell.heat = 0.0;
        step_burn_hold(&mut cell);
        assert_eq!(cell.heat, 0.0);
        assert!(cell.fuel < MIN_CELL_FUEL_FOR_SPREAD);
    }

    #[test]
    fn sweep_rain_does_not_restore_consumed_fuel() {
        let dry = step_cell_fuel(0.2, 0.5, 1.0, 0.0);
        let soaked = step_cell_fuel(0.2, 0.5, 1.0, 1.0);
        assert_eq!(dry, soaked);
        assert!(soaked.fuel < 0.2);
    }

    #[test]
    fn sweep_spread_does_not_land_on_a_held_burn_cell() {
        let src = FireCell {
            heat: 0.5,
            fuel: 0.4,
            burned_hold: 0,
        };
        let mut held = FireCell {
            heat: BURNED_RESIDUAL_HEAT,
            fuel: 0.0,
            burned_hold: 3,
        };
        assert!(!spread_onto_neighbor(&src, &mut held));
        assert_eq!(held.burned_hold, 3);
        assert!((held.fuel).abs() < 1e-6);

        let mut charred_src = src;
        charred_src.fuel = 0.0;
        charred_src.burned_hold = 2;
        let mut open = FireCell {
            heat: 0.0,
            fuel: 0.3,
            burned_hold: 0,
        };
        assert!(!spread_onto_neighbor(&charred_src, &mut open));
        assert!((open.heat).abs() < 1e-6);
        assert!((open.fuel - 0.3).abs() < 1e-6);
    }

    #[test]
    fn drain_snow_blocks_neighbor_spread() {
        assert!(!spread_allowed(SNOW_BLOCKS_SPREAD, 0.1));
        assert!(spread_allowed(0.0, 0.1));
        let src = FireCell {
            heat: 0.5,
            fuel: 0.4,
            burned_hold: 0,
        };
        let mut dst = FireCell {
            heat: 0.0,
            fuel: 0.3,
            burned_hold: 0,
        };
        let before = dst;
        if spread_allowed(SNOW_BLOCKS_SPREAD, 0.1) {
            let _ = take_neighbor_deposit(false, &src, &mut dst, NEIGHBOR_HEAT_GAIN);
        }
        assert_eq!(dst, before);
    }

    #[test]
    fn drain_wet_neighbor_refuses_spread() {
        assert!(!spread_allowed(0.0, MOISTURE_BLOCKS_SPREAD));
        let src = FireCell {
            heat: 0.5,
            fuel: 0.4,
            burned_hold: 0,
        };
        let mut dst = FireCell {
            heat: 0.02,
            fuel: 0.25,
            burned_hold: 0,
        };
        let before = dst;
        if spread_allowed(0.0, MOISTURE_BLOCKS_SPREAD) {
            let _ = take_neighbor_deposit(false, &src, &mut dst, NEIGHBOR_HEAT_GAIN);
        }
        assert_eq!(dst, before);
    }

    #[test]
    fn drain_snow_clears_the_burn_hold() {
        let mut cell = FireCell {
            heat: BURNED_RESIDUAL_HEAT,
            fuel: 0.0,
            burned_hold: BURNED_HOLD_TICKS,
        };
        apply_snow_to_hold(&mut cell, SNOW_BLOCKS_SPREAD);
        assert_eq!(cell.burned_hold, 0);
        assert_eq!(cell.heat, 0.0);
        let mut dry = cell;
        dry.burned_hold = 3;
        dry.heat = BURNED_RESIDUAL_HEAT;
        apply_snow_to_hold(&mut dry, 0.1);
        assert_eq!(dry.burned_hold, 3);
        assert!((dry.heat - BURNED_RESIDUAL_HEAT).abs() < 1e-6);
    }

    #[test]
    fn live_lightning_spark_respects_the_fuel_gate() {
        let mass = MIN_WILDLAND_FUEL_MASS + 0.4;
        assert_eq!(lightning_spark(mass, 0.4, 0.0), 0.0);
        assert!(lightning_spark(mass, 0.4, 1.0) > 0.0);
        assert_eq!(
            lightning_spark(MIN_WILDLAND_FUEL_MASS - 0.05, 0.4, 1.0),
            0.0
        );
        assert_eq!(
            lightning_spark(mass, MIN_CELL_FUEL_FOR_SPREAD - 0.01, 1.0),
            0.0
        );
    }

    #[test]
    fn live_fog_halves_neighbor_heat_gain() {
        let clear = scaled_neighbor_heat_gain(0.0);
        let fogged = scaled_neighbor_heat_gain(1.0);
        assert!((clear - NEIGHBOR_HEAT_GAIN).abs() < 1e-6);
        assert!((fogged - NEIGHBOR_HEAT_GAIN * 0.5).abs() < 1e-6);
        let src = FireCell {
            heat: 0.4,
            fuel: 0.3,
            burned_hold: 0,
        };
        let mut dst = FireCell {
            heat: 0.0,
            fuel: 0.2,
            burned_hold: 0,
        };
        assert!(spread_with_gain(&src, &mut dst, fogged));
        assert!((dst.heat - fogged).abs() < 1e-6);
        assert!((dst.fuel - (0.2 - NEIGHBOR_FUEL_DRAW)).abs() < 1e-6);
    }

    #[test]
    fn live_snow_does_not_arm_a_burn_hold() {
        assert!(!should_arm_burn_hold(true, SNOW_BLOCKS_SPREAD));
        assert!(should_arm_burn_hold(true, 0.0));
        assert!(!should_arm_burn_hold(false, 0.0));
    }

    #[test]
    fn push_heavy_rain_blocks_neighbor_spread() {
        assert!(!rain_blocks_neighbor_spread(0.0));
        assert!(!rain_blocks_neighbor_spread(0.4));
        assert!(rain_blocks_neighbor_spread(RAIN_BLOCKS_SPREAD));
        let src = FireCell {
            heat: 0.4,
            fuel: 0.3,
            burned_hold: 0,
        };
        let mut dst = FireCell {
            heat: 0.0,
            fuel: 0.2,
            burned_hold: 0,
        };
        let before = dst;
        if !rain_blocks_neighbor_spread(RAIN_BLOCKS_SPREAD) {
            let _ = take_neighbor_deposit(false, &src, &mut dst, NEIGHBOR_HEAT_GAIN);
        }
        assert_eq!(dst, before);
    }

    #[test]
    fn push_ember_skips_an_empty_cell() {
        assert_eq!(ember_heat_deposit(MIN_CELL_FUEL_FOR_SPREAD - 0.01, 0.2), 0.0);
        assert!((ember_heat_deposit(0.4, 0.2) - 0.2).abs() < 1e-6);
        assert_eq!(ember_heat_deposit(0.4, 0.0), 0.0);
    }

    #[test]
    fn push_second_neighbor_does_not_stack_heat() {
        let src = FireCell {
            heat: 0.5,
            fuel: 0.4,
            burned_hold: 0,
        };
        let mut dst = FireCell {
            heat: 0.1,
            fuel: 0.3,
            burned_hold: 0,
        };
        assert!(take_neighbor_deposit(false, &src, &mut dst, 0.04));
        let once = dst.heat;
        assert!(!take_neighbor_deposit(true, &src, &mut dst, 0.04));
        assert!((dst.heat - once).abs() < 1e-6);
    }

    #[test]
    fn next_wind_burns_fuel_faster_than_still_air() {
        let still = step_cell_fuel(0.4, 0.6, 1.0, 0.0);
        let calm = fuel_step_in_wind(0.4, 0.6, 1.0, 1.0, 0.0);
        let blown = fuel_step_in_wind(0.4, 0.6, 1.0, 1.0, 1.0);
        assert_eq!(calm, still);
        assert!(blown.fuel < still.fuel);
        assert!(blown.fuel <= 0.4);
    }

    #[test]
    fn next_zero_timestep_does_not_consume_fuel() {
        let held = step_cell_fuel(0.2, 0.8, 0.0, 0.4);
        assert!((held.fuel - 0.2).abs() < 1e-6);
        assert!(!held.crossed_below_spread);
        assert!(!held.arm_hold);
        let negative = step_cell_fuel(0.2, 0.8, -1.0, 0.0);
        assert!((negative.fuel - 0.2).abs() < 1e-6);
    }

    #[test]
    fn cont_saturated_neighbor_keeps_its_fuel() {
        let src = FireCell {
            heat: 0.5,
            fuel: 0.4,
            burned_hold: 0,
        };
        let mut full = FireCell {
            heat: 1.0,
            fuel: 0.3,
            burned_hold: 0,
        };
        assert!(!spread_with_gain(&src, &mut full, NEIGHBOR_HEAT_GAIN));
        assert!((full.heat - 1.0).abs() < 1e-6);
        assert!((full.fuel - 0.3).abs() < 1e-6);
    }

    #[test]
    fn cont_zero_heat_gain_does_not_draw_fuel() {
        let src = FireCell {
            heat: 0.5,
            fuel: 0.4,
            burned_hold: 0,
        };
        let mut dst = FireCell {
            heat: 0.1,
            fuel: 0.3,
            burned_hold: 0,
        };
        assert!(!spread_with_gain(&src, &mut dst, 0.0));
        assert!(!spread_with_gain(&src, &mut dst, -0.04));
        assert!((dst.heat - 0.1).abs() < 1e-6);
        assert!((dst.fuel - 0.3).abs() < 1e-6);
    }

    #[test]
    fn fold_fuel_cannot_go_negative() {
        let burned = step_cell_fuel(0.001, 80.0, 40.0, 0.0);
        assert_eq!(burned.fuel, 0.0);
        let blown = fuel_step_in_wind(-0.4, 20.0, 5.0, 1.0, 1.0);
        assert_eq!(blown.fuel, 0.0);
        let idle = step_cell_fuel(-2.0, 1.0, 0.0, 0.0);
        assert_eq!(idle.fuel, 0.0);
    }

    #[test]
    fn fold_weak_ember_at_fuel_floor_is_zero() {
        let floor = MIN_CELL_FUEL_FOR_SPREAD;
        let weak = EMBER_FLOOR_SPARK_MIN * 0.5;
        assert_eq!(ember_heat_deposit(floor, weak), 0.0);
        assert!((ember_heat_deposit(floor, EMBER_FLOOR_SPARK_MIN) - EMBER_FLOOR_SPARK_MIN).abs() < 1e-6);
        assert!((ember_heat_deposit(floor + 0.02, weak) - weak).abs() < 1e-6);
    }

    #[test]
    fn fold_still_air_wind_delta_is_fixed() {
        let still = step_cell_fuel(0.5, 0.6, 2.0, 0.2);
        let blown = fuel_step_in_wind(0.5, 0.6, 2.0, 0.2, 0.5);
        let again = fuel_step_in_wind(0.5, 0.6, 2.0, 0.2, 0.5);
        let delta = wind_fuel_delta(0.6, 2.0, 0.5);
        assert_eq!(blown, again);
        assert!((still.fuel - blown.fuel - delta).abs() < 1e-5);
        assert!(delta > 0.0);
        assert_eq!(wind_fuel_delta(0.6, 2.0, 0.0), 0.0);
        assert!(blown.fuel >= 0.0);
    }
}
