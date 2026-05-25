use crate::models::SubsystemTag;

pub fn trace_subsystem(file: &str) -> SubsystemTag {
    let norm = file.replace('\\', "/");
    let parts: Vec<&str> = norm.split('/').collect();

    if norm.contains("sim_view_sync_debug") {
        return SubsystemTag {
            domain: "GUI",
            system: "HUD",
            feature: "VIEWPORT_SYNC_DEBUG",
        };
    }
    if norm.contains("viewport_layout_solver") || norm.contains("authoritative_viewport") {
        return SubsystemTag {
            domain: "GUI",
            system: "VIEWPORT_AUTHORITY",
            feature: "SEMANTIC_SOLVER",
        };
    }
    if norm.contains("viewport_authority") {
        return SubsystemTag {
            domain: "GUI",
            system: "VIEWPORT_AUTHORITY",
            feature: "AUTHORITY_COMMIT",
        };
    }
    if norm.contains("map_view/") {
        return SubsystemTag {
            domain: "GUI",
            system: "MAP_VIEW",
            feature: "PRESENTATION_SPINE",
        };
    }
    if norm.starts_with("src/gui/hud/") {
        return SubsystemTag {
            domain: "GUI",
            system: "HUD",
            feature: infer_hud_feature(&norm),
        };
    }
    if norm.starts_with("src/gui/") {
        return SubsystemTag {
            domain: "GUI",
            system: "SHELL",
            feature: infer_gui_feature(&norm),
        };
    }
    if norm.starts_with("src/render/") {
        return SubsystemTag {
            domain: "RENDER",
            system: infer_render_system(&norm),
            feature: "PIPELINE",
        };
    }
    if norm.starts_with("src/compute/") {
        return SubsystemTag {
            domain: "COMPUTE",
            system: "DISPATCH",
            feature: "GPU_OFFLOAD",
        };
    }
    if norm.starts_with("src/engine/") {
        return SubsystemTag {
            domain: "ENGINE",
            system: "RUNTIME",
            feature: infer_engine_feature(&norm),
        };
    }
    if norm.starts_with("src/dev/") {
        return SubsystemTag {
            domain: "DEV",
            system: "STAGE5",
            feature: "READINESS",
        };
    }

    let _top = parts.get(1).copied().unwrap_or("unknown");
    SubsystemTag {
        domain: "SRC",
        system: "CORE",
        feature: "GENERAL",
    }
}

fn infer_hud_feature(path: &str) -> &'static str {
    if path.contains("minimap") {
        "MINIMAP"
    } else if path.contains("viewport") {
        "VIEWPORT"
    } else {
        "PRODUCT_SHELL"
    }
}

fn infer_gui_feature(path: &str) -> &'static str {
    if path.contains("editor/world_preview") {
        "WORLD_PREVIEW"
    } else if path.contains("build/") {
        "CONSTRUCTION"
    } else if path.contains("map_view") {
        "MAP_VIEW"
    } else {
        "LAYOUT"
    }
}

fn infer_render_system(path: &str) -> &'static str {
    if path.contains("extraction") {
        "EXTRACTION"
    } else if path.contains("viewport") || path.contains("ResolvedViewport") {
        "VIEWPORT_RESOLVE"
    } else {
        "FRAME_GRAPH"
    }
}

fn infer_engine_feature(path: &str) -> &'static str {
    if path.contains("worldgen") {
        "WORLDGEN"
    } else if path.contains("test_harness") {
        "TEST_HARNESS"
    } else {
        "APP_SHELL"
    }
}

pub fn subsystem_graph_markdown() -> String {
    r#"## Viewport authority spine

```text
viewport authority
    ├── semantic viewport (viewport_layout_solver)
    ├── ui measured rect (authoritative_viewport)
    ├── layout solver (commit_authority_from_semantic)
    ├── camera viewport (map_camera / MainWorldCamera)
    ├── minimap shell (MinimapShellState)
    ├── world preview (map_view / world_preview)
    ├── render sync (ResolvedViewports)
    └── debug tracing (sim_view_sync_debug, viewport_authority_debug)
```

## Map presentation spine

```text
map_view
    ├── backend / texture cache
    ├── presentation state
    ├── projection / resolved frames
    ├── minimap consumer
    └── world_preview consumer
```

## Render extraction spine

```text
render
    ├── RenderProjectionGraph
    ├── fire visual extract
    └── viewport resolve sets
```
"#
    .to_string()
}

pub fn related_systems_for_tag(tag: &SubsystemTag) -> Vec<String> {
    match (tag.domain, tag.system, tag.feature) {
        ("GUI", "VIEWPORT_AUTHORITY", _) | ("GUI", "HUD", "VIEWPORT_SYNC_DEBUG") => vec![
            "semantic_viewport".into(),
            "ui_measured_rect".into(),
            "camera_viewport".into(),
            "minimap_shell".into(),
            "world_preview".into(),
            "render_diagnostics".into(),
            "drift_detection".into(),
        ],
        ("GUI", "MAP_VIEW", _) => vec![
            "map_texture_cache".into(),
            "resolved_map_view_frames".into(),
            "minimap".into(),
            "world_preview".into(),
        ],
        ("RENDER", "VIEWPORT_RESOLVE", _) | ("RENDER", _, "PIPELINE") => {
            vec!["ResolvedViewports".into(), "camera_scissor".into()]
        }
        _ => Vec::new(),
    }
}

pub fn tag_path(file: &str) -> String {
    trace_subsystem(file).display_path()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sim_view_sync_maps_to_viewport_debug_pipeline() {
        let tag = trace_subsystem("src/gui/hud/sim_view_sync_debug.rs");
        assert_eq!(tag.domain, "GUI");
        assert_eq!(tag.feature, "VIEWPORT_SYNC_DEBUG");
    }
}
