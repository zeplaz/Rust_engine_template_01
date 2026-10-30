//! Plain-language report of one site or plant for the info panel (presentation reads, never writes).
//!
//! Building sites report stage progress, the material bill and why they are held. Operational
//! plants report production rate, workforce and power (with the town / grid they come from), input
//! and output buffers, and every reason output is below capacity.

use crate::economy::facility_inputs::{is_supply_input_tag, FacilityInputs, PowerState};
use crate::economy::resource_flow::{
    resource_type_from_tag, FacilityFlowState, ResourceFlowNode, INPUT_BUFFER_TARGET, OUTPUT_BUFFER_CAP,
};
use crate::construction::SiteStageProgress;
use crate::strategic::{ConstructionSite, SiteMaterialStarved, SiteResourceManifest};
use crate::strategic::SiteConstructionPhase;

/// Units of `tag` on the node (typed resources live in `inventory`, designer tags in `buffer_by_tag`).
#[must_use]
pub fn node_amount(node: &ResourceFlowNode, tag: &str) -> f32 {
    match resource_type_from_tag(tag) {
        Some(rt) => node.inventory.get(&rt).copied().unwrap_or(0.0),
        None => node.buffer_by_tag.get(tag).copied().unwrap_or(0.0),
    }
}

/// Units produced per sim tick right now: throughput cap scaled by workforce / power and by input
/// starvation, zero while every output buffer is full.
#[must_use]
pub fn production_rate_per_tick(
    node: &ResourceFlowNode,
    state: &FacilityFlowState,
    inputs: Option<&FacilityInputs>,
) -> f32 {
    if state.output_full {
        return 0.0;
    }
    let supply = inputs.map_or(1.0, FacilityInputs::factor);
    let nominal: f32 = node.production.iter().map(|r| r.rate_per_tick).sum();
    nominal.min(node.throughput_limit * state.output_scale * supply)
}

fn pct(v: f32) -> u32 {
    (v.clamp(0.0, 1.0) * 100.0).round() as u32
}

/// Everything wrong with a plant's output, most blocking first.
#[must_use]
pub fn stall_reasons(
    node: &ResourceFlowNode,
    state: &FacilityFlowState,
    inputs: Option<&FacilityInputs>,
) -> Vec<String> {
    let mut out = Vec::new();
    if state.output_full {
        out.push("output full: logistics must haul product away (production paused)".to_string());
    }
    if let Some(i) = inputs {
        match i.power_state {
            PowerState::Disconnected => out.push("no power: not connected to a power plant".to_string()),
            PowerState::Overloaded => out.push(format!("grid overloaded: power at {}%", pct(i.power))),
            _ => {}
        }
        if i.workforce <= 0.0 {
            out.push(match &i.town {
                Some((town, _)) => format!("no workers available from {town}"),
                None => "no workers: no town within commute range".to_string(),
            });
        } else if i.workforce < 1.0 {
            out.push(format!("short of workers: {}% staffed", pct(i.workforce)));
        }
    }
    let short: Vec<&str> = node
        .consumption
        .iter()
        .filter(|r| !is_supply_input_tag(&r.tag))
        .filter(|r| node_amount(node, &r.tag) < r.rate_per_tick)
        .map(|r| r.tag.as_str())
        .collect();
    if !short.is_empty() {
        out.push(format!("inputs short: {} (request a supply line)", short.join(", ")));
    }
    out
}

/// Panel lines for one site entity. Returns an empty list for a site with nothing to report.
#[must_use]
#[allow(clippy::too_many_arguments)]
pub fn facility_report_lines(
    catalog_id: &str,
    site: &ConstructionSite,
    stage: Option<&SiteStageProgress>,
    manifest: Option<&SiteResourceManifest>,
    starved: Option<&SiteMaterialStarved>,
    node: Option<&ResourceFlowNode>,
    state: Option<&FacilityFlowState>,
    inputs: Option<&FacilityInputs>,
) -> Vec<String> {
    let mut lines = Vec::new();
    match (site.phase, node, state) {
        (SiteConstructionPhase::Operational, Some(node), Some(state)) => {
            lines.push(format!("{catalog_id} - operational"));
            let rate = production_rate_per_tick(node, state, inputs);
            let outputs: Vec<&str> = node.production.iter().map(|r| r.tag.as_str()).collect();
            if !outputs.is_empty() {
                let nominal: f32 = node.production.iter().map(|r| r.rate_per_tick).sum();
                let of = if nominal > 0.0 { rate / nominal } else { 0.0 };
                lines.push(format!("Output: {rate:.2} /tick of {} ({}% of nominal)", outputs.join(", "), pct(of)));
            }
            if let Some(i) = inputs {
                lines.push(match &i.town {
                    Some((town, metres)) => {
                        format!("Workforce: {}% from {town} ({:.1} km)", pct(i.workforce), metres / 1000.0)
                    }
                    None if i.workforce >= 1.0 => "Workforce: not modelled".to_string(),
                    None => format!("Workforce: {}% (no town in commute range)", pct(i.workforce)),
                });
                lines.push(match i.power_state {
                    PowerState::Unmodelled => "Power: not modelled".to_string(),
                    PowerState::Disconnected => "Power: 0% (not connected to a power plant)".to_string(),
                    PowerState::Connected => "Power: 100% (grid connected)".to_string(),
                    PowerState::Overloaded => format!("Power: {}% (grid overloaded)", pct(i.power)),
                });
            }
            let ins: Vec<String> = node
                .consumption
                .iter()
                .filter(|r| !is_supply_input_tag(&r.tag))
                .map(|r| format!("{} {:.0}/{:.0}", r.tag, node_amount(node, &r.tag), INPUT_BUFFER_TARGET))
                .collect();
            if !ins.is_empty() {
                lines.push(format!("Inputs: {}", ins.join(", ")));
            }
            let outs: Vec<String> = node
                .production
                .iter()
                .map(|r| {
                    let have = node_amount(node, &r.tag);
                    let full = if have >= OUTPUT_BUFFER_CAP - f32::EPSILON { " FULL" } else { "" };
                    format!("{} {have:.0}/{OUTPUT_BUFFER_CAP:.0}{full}", r.tag)
                })
                .collect();
            if !outs.is_empty() {
                lines.push(format!("Stored output: {}", outs.join(", ")));
            }
            let reasons = stall_reasons(node, state, inputs);
            if !reasons.is_empty() {
                lines.push(format!("Held back: {}", reasons.join("; ")));
            }
        }
        _ => {
            let progress = stage.map_or(0.0, |s| s.progress);
            lines.push(format!("{catalog_id} - {:?} {}%", site.phase, pct(progress)));
            if let Some(m) = manifest {
                let bill: Vec<String> = [
                    ("Concrete", m.concrete_required, m.delivered_concrete),
                    ("Steel", m.steel_required, m.delivered_steel),
                    ("Fuel", m.fuel_required, m.delivered_fuel),
                    ("Machinery", m.machinery_required, m.delivered_machinery),
                ]
                .into_iter()
                .filter(|(_, required, _)| *required > 0.0)
                .map(|(tag, required, delivered)| format!("{tag} {delivered:.0}/{required:.0}"))
                .collect();
                if !bill.is_empty() {
                    lines.push(format!("Materials delivered: {}", bill.join(", ")));
                }
            }
            if let Some(s) = starved {
                let missing: Vec<String> = s.missing.iter().map(|(tag, n)| format!("{tag} x{n}")).collect();
                lines.push(format!("STALLED: waiting for {}", missing.join(", ")));
            }
        }
    }
    lines
}
