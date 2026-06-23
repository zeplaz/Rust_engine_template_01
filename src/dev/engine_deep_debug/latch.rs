//! Deep debug latch — feature build + env + CLI.

use std::sync::atomic::{AtomicBool, Ordering};

use bevy::prelude::*;

static CLI_ARMED: AtomicBool = AtomicBool::new(false);

#[derive(Resource, Clone, Debug)]
pub struct DeepDebugConfig {
    pub active: bool,
    pub minimap_trace: bool,
    pub gpu_render_trace: bool,
    pub schedule_trace: bool,
    pub entity_inventory: bool,
    pub flush_every_n_frames: u32,
    pub jsonl_frames: bool,
}

impl Default for DeepDebugConfig {
    fn default() -> Self {
        Self {
            active: deep_debug_active(),
            minimap_trace: true,
            gpu_render_trace: true,
            schedule_trace: true,
            entity_inventory: true,
            flush_every_n_frames: 30,
            jsonl_frames: env_on("RUST_ENGINE_DEEP_DEBUG_JSONL"),
        }
    }
}

#[must_use]
pub fn deep_debug_feature_compiled() -> bool {
    cfg!(feature = "engine_deep_debug")
}

pub fn arm_deep_debug_from_cli() {
    CLI_ARMED.store(true, Ordering::Relaxed);
}

fn env_on(key: &str) -> bool {
    std::env::var(key)
        .ok()
        .is_some_and(|v| v == "1" || v.eq_ignore_ascii_case("true") || v.eq_ignore_ascii_case("yes"))
}

fn env_off(key: &str) -> bool {
    std::env::var(key)
        .ok()
        .is_some_and(|v| v == "0" || v.eq_ignore_ascii_case("false") || v.eq_ignore_ascii_case("no"))
}

/// True when intrusive debug should run.
#[must_use]
pub fn deep_debug_active() -> bool {
    if env_off("RUST_ENGINE_DEEP_DEBUG") {
        return false;
    }
    if env_on("RUST_ENGINE_DEEP_DEBUG") || CLI_ARMED.load(Ordering::Relaxed) {
        return true;
    }
    if deep_debug_feature_compiled() {
        return true;
    }
    false
}

pub fn init_startup_config(mut commands: Commands) {
    let active = deep_debug_active();
    let mut cfg = DeepDebugConfig::default();
    cfg.active = active;
    if active {
        if env_on("RUST_ENGINE_DEEP_DEBUG_MINIMAP_ONLY") {
            cfg.gpu_render_trace = false;
            cfg.schedule_trace = false;
            cfg.entity_inventory = false;
        }
        if let Ok(v) = std::env::var("RUST_ENGINE_DEEP_DEBUG_FLUSH_EVERY") {
            if let Ok(n) = v.parse::<u32>() {
                if n > 0 {
                    cfg.flush_every_n_frames = n;
                }
            }
        }
        info!(
            target: "engine_deep_debug",
            feature = deep_debug_feature_compiled(),
            jsonl = cfg.jsonl_frames,
            flush_every = cfg.flush_every_n_frames,
            "ENGINE-DEEP-DEBUG armed — minimap/GPU/schedule witnesses active"
        );
        if !std::env::var("MINIMAP_GPU_DEBUG").is_ok() {
            std::env::set_var("MINIMAP_GPU_DEBUG", "1");
        }
        if !std::env::var("VIEW_RUNTIME_AUDIT").is_ok() {
            std::env::set_var("VIEW_RUNTIME_AUDIT", "1");
        }
    }
    commands.insert_resource(cfg);
}
