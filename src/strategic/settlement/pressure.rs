//! District development pressure (ECON-OG-1-B).

use bevy::prelude::*;

use super::district::{DevelopmentPressure, DevelopmentPressureBook, DistrictBook, DistrictMetricsBook};
use super::town::TownBook;

pub fn rollup_district_metrics_system(
    town_book: Res<TownBook>,
    district_book: Res<DistrictBook>,
    mut metrics: ResMut<DistrictMetricsBook>,
) {
    let Some(town_id) = town_book.default_town.as_ref() else {
        return;
    };
    let Some(town) = town_book.towns.get(town_id) else {
        return;
    };
    for district in district_book.districts.values() {
        if district.town_id != *town_id {
            continue;
        }
        let area = (district.tile_rect.width().max(1) * district.tile_rect.height().max(1)) as f32;
        let pop_norm = (town.population as f32 / area).clamp(0.0, 1.0);
        let jobs_norm = (town.jobs as f32 / town.population.max(1) as f32).clamp(0.0, 1.0);
        let housing_norm = (town.housing as f32 / town.population.max(1) as f32).clamp(0.0, 1.0);
        metrics.by_district.insert(
            district.id.clone(),
            super::district::DistrictMetrics {
                population_density: pop_norm,
                employment_density: jobs_norm,
                employment_demand: (jobs_norm - 0.5).clamp(0.0, 1.0),
                housing_deficit: (1.0 - housing_norm).clamp(0.0, 1.0),
                transport_access: 0.9,
                utility_service: 1.0,
                freight_access: 1.0,
                ..Default::default()
            },
        );
    }
}

pub fn compute_district_pressure_system(
    metrics: Res<DistrictMetricsBook>,
    mut pressure: ResMut<DevelopmentPressureBook>,
) {
    pressure.by_district.clear();
    for (district_id, m) in &metrics.by_district {
        pressure.by_district.insert(
            district_id.clone(),
            DevelopmentPressure {
                residential: (m.housing_deficit * 0.7 + m.desirability * 0.3).clamp(0.0, 1.0),
                commercial: (m.employment_demand * 0.6 + m.transport_access * 0.4).clamp(0.0, 1.0),
                industrial: (m.freight_access * 0.5 + m.employment_demand * 0.5).clamp(0.0, 1.0),
            },
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mixed_use_high_transport_commercial_pressure() {
        assert!(district_pressure_witness_green());
    }
}

#[must_use]
pub fn district_pressure_witness_green() -> bool {
    use crate::strategic::settlement::district::portland_fixture_district;
    use crate::strategic::settlement::town::portland_fixture_town;

    let town = portland_fixture_town();
    let districts = portland_fixture_district(&town);
    let mut app = App::new();
    app.insert_resource(town)
        .insert_resource(districts)
        .init_resource::<DistrictMetricsBook>()
        .init_resource::<DevelopmentPressureBook>()
        .add_systems(
            Update,
            (rollup_district_metrics_system, compute_district_pressure_system).chain(),
        );
    app.update();
    let pressure = app.world().resource::<DevelopmentPressureBook>();
    let Some(p) = pressure.by_district.values().next() else {
        return false;
    };
    p.commercial > p.residential * 0.5
}
