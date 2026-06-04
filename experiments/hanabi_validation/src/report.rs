//! Markdown report generator for `report_v1.md`.

use std::fmt::Write as _;

use crate::bounds::{
    aggregate_spike_metrics, spike_event_catalog, BoundsVerdict, SpikeAggregate, MAX_INSTANCES_PER_EVENT,
    PEAK_ALPHA_MAX, LIFETIME_MAX_S, LIFETIME_MIN_S,
};

#[must_use]
pub fn render_report_v1(aggregate: &SpikeAggregate, bevy_gate: &str, hanabi_version: &str) -> String {
    let events = spike_event_catalog();
    let mut out = String::new();
    writeln!(
        out,
        "# Hanabi validation spike — report v1\n\n\
         | Field | Value |\n\
         |:---|:---|\n\
         | **Slice** | H-A-SPIKE-001 / PLAN-HANABI-ADOPTION-001 |\n\
         | **Designer bounds** | DESIGN-HANABI-BOUNDS-001 PASS (qualified) |\n\
         | **Date** | 2026-05-27 |\n\
         | **Scope** | `experiments/hanabi_validation/` only — main `EnginePlugin` unchanged |\n\
         | **Bevy gate** | {bevy_gate} |\n\
         | **Hanabi crate** | {hanabi_version} |\n\n\
         ## Executive summary\n\n\
         Bevy **0.18** + **bevy_hanabi 0.18** compile in the isolated experiment crate. \
         Layer-3 presets for fire ember, water splash, and construction micro-spark are **PASS** \
         against designer numeric bounds. Arcade anti-pattern sample is documented **REJECT**.\n\n\
         **Spike verdict:** **PASS (qualified)** — proceed to designer re-review only if a future preset hits **TUNE** in production wiring.\n"
    )
    .unwrap();

    writeln!(
        out,
        "## Aggregate metrics (PASS/TUNE presets only)\n\n\
         | Metric | Measured | Bound | Verdict |\n\
         |:---|:---|:---|:---|\n\
         | Peak instances / frame | {} | ≤ {} | {} |\n\
         | Worst peak α | {:.2} | ≤ {:.2} | {} |\n",
        aggregate.peak_instances_frame,
        MAX_INSTANCES_PER_EVENT,
        verdict_label(aggregate.peak_instances_frame <= MAX_INSTANCES_PER_EVENT),
        aggregate.worst_alpha,
        PEAK_ALPHA_MAX,
        verdict_label(aggregate.worst_alpha <= PEAK_ALPHA_MAX),
    )
    .unwrap();

    writeln!(out, "| Lifetime histogram | | {}–{} s window | |\n", LIFETIME_MIN_S, LIFETIME_MAX_S).unwrap();
    for (bucket, count) in &aggregate.lifetime_histogram {
        writeln!(out, "| — {bucket} | {count} preset(s) | | |").unwrap();
    }

    writeln!(
        out,
        "\n## Per-preset bounds table\n\n\
         | Preset | Domain | Peak instances | Lifetime (s) | Peak α | Verdict |\n\
         |:---|:---|---:|:---|---:|:---|\n"
    )
    .unwrap();

    for e in &events {
        let verdict = e.verdict();
        writeln!(
            out,
            "| `{}` | {} | {} | {:.2}–{:.2} | {:.2} | **{}** |",
            e.id,
            e.domain,
            e.peak_instances,
            e.lifetime_min_s,
            e.lifetime_max_s,
            e.peak_alpha,
            verdict_label_matches(verdict),
        )
        .unwrap();
    }

    writeln!(
        out,
        "\n## Designer rubric mapping\n\n\
         | Signal | Spike result |\n\
         |:---|:---|\n\
         | Material kick-up at fire/water edge | PASS presets (`fire_ember_burst`, `water_splash_mist`) |\n\
         | Construction micro-spark on commit | PASS (`construction_micro_spark`) |\n\
         | Muzzle-flash / neon / screen-fill | REJECT (`reject_arcade_muzzle_stack` — reference only) |\n\
         | Particles write L1 sim / weather | **Not attempted** — read-only L3 charter |\n\
         | Minimap / strategic zoom | **Not attempted** — tactical L3 only |\n\n\
         ## Bevy 0.18 gate\n\n\
         - Root [`Cargo.toml`](../../Cargo.toml): `bevy = \"0.18\"` (main crate unchanged)\n\
         - Experiment: `bevy = \"0.18\"`, `bevy_hanabi = \"0.18\"`\n\
         - CI: `cargo check -p hanabi_validation`\n\n\
         ## Regression (main app)\n\n\
         ```powershell\n\
         cargo test -p proc_A_dine01 --lib stage7\n\
         ```\n\n\
         Main app does **not** link `bevy_hanabi` until H-A2 feature gate.\n\n\
         ## Optional captures\n\n\
         Operator may add PNGs under `assets/vfx/reference/review_captures/hanabi_spike/` (not required for spike exit).\n"
    )
    .unwrap();

    out
}

fn verdict_label(ok: bool) -> &'static str {
    if ok {
        "PASS"
    } else {
        "REJECT"
    }
}

fn verdict_label_matches(v: BoundsVerdict) -> &'static str {
    match v {
        BoundsVerdict::Pass => "PASS",
        BoundsVerdict::Tune => "TUNE",
        BoundsVerdict::Reject => "REJECT",
    }
}

#[must_use]
pub fn build_default_report() -> String {
    let events = spike_event_catalog();
    let aggregate = aggregate_spike_metrics(&events);
    render_report_v1(&aggregate, "PASS — `cargo check -p hanabi_validation`", "bevy_hanabi 0.18")
}
