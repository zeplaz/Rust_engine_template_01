//! **CLN-P0-T4-001** — intentional scaffold trait for future production rate aggregation.

pub trait RateCalculatable {
    fn calculate_total_rate(&self) -> f32;
}

