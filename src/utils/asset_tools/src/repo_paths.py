"""
Canonical paths into the game repository for desktop content tools.

Artists and designers should edit JSON / Tiled sources under `assets/`; the Bevy app loads
those files. This module finds the repo root regardless of the current working directory
when running `python run.py` from `src/utils/asset_tools/`.
"""

from __future__ import annotations

from pathlib import Path
from typing import Final, Optional

__all__ = [
    "repo_root",
    "REPO_ROOT",
    "assets_dir",
    "plant_definitions_json",
    "buildings_configs_dir",
    "vehicles_configs_dir",
    "tiled_assets_dir",
    "production_config_data",
    "game_entities_consts_file",
    "roads_configs_dir",
    "rails_configs_dir",
    "building_types_index_json",
    "world_gen_tuning_json",
    "world_gen_tuning_example_json",
    "terrain_config_dir",
    "material_registry_json",
    "material_registry_example_json",
    "tag_registry_json",
    "tag_registry_example_json",
    "material_rules_ron",
    "material_rules_example_ron",
]


def repo_root(start: Optional[Path] = None) -> Path:
    """Walk up from ``start`` (or this file) until ``Cargo.toml`` and ``assets/`` exist."""
    p = (start or Path(__file__)).resolve()
    for _ in range(10):
        if (p / "Cargo.toml").is_file() and (p / "assets").is_dir():
            return p
        parent = p.parent
        if parent == p:
            break
        p = parent
    raise RuntimeError(
        "Could not locate repo root (expected Cargo.toml and assets/ directory). "
        "Run the asset editor from the game repository tree."
    )


REPO_ROOT: Final[Path] = repo_root()
assets_dir: Final[Path] = REPO_ROOT / "assets"

# Gameplay / economy JSON (engine `include_str!` or AssetServer)
plant_definitions_json: Final[Path] = assets_dir / "config" / "power" / "plant_definitions.json"

# Building & vehicle blueprints (tooling + future loaders)
buildings_configs_dir: Final[Path] = assets_dir / "configs" / "buildings"
vehicles_configs_dir: Final[Path] = assets_dir / "configs" / "vehicles"

# Isometric / tile authoring (Tiled)
tiled_assets_dir: Final[Path] = assets_dir / "tiled"

# Legacy / misc production data
production_config_data: Final[Path] = assets_dir / "configs" / "production"

# Text key/value consts (voltage tuples, etc.) — see integration/const_game_entities.py
game_entities_consts_file: Final[Path] = assets_dir / "game_entities" / "consts_for_entities.const"

# Surface transport blueprint examples (editor + future loaders)
roads_configs_dir: Final[Path] = assets_dir / "configs" / "roads"
rails_configs_dir: Final[Path] = assets_dir / "configs" / "rails"

# Index of BuildingType / examples — see dynamic_building_page + s_flagz.rs
building_types_index_json: Final[Path] = buildings_configs_dir / "_building_types_index.json"

# World generation overlay — matches `WORLD_GEN_TUNING_JSON_PATH` in `world_generator_enhanced.rs`
world_gen_tuning_json: Final[Path] = assets_dir / "config" / "world_gen_tuning.json"
world_gen_tuning_example_json: Final[Path] = assets_dir / "config" / "world_gen_tuning.example.json"

# Terrain material / tag / rule configs — see `material_unification_matrix_v1.md`
terrain_config_dir: Final[Path] = assets_dir / "config" / "terrain"
material_registry_json: Final[Path] = terrain_config_dir / "material_registry.json"
material_registry_example_json: Final[Path] = terrain_config_dir / "material_registry.example.json"
tag_registry_json: Final[Path] = terrain_config_dir / "tag_registry.json"
tag_registry_example_json: Final[Path] = terrain_config_dir / "tag_registry.example.json"
material_rules_ron: Final[Path] = terrain_config_dir / "material_rules.ron"
material_rules_example_ron: Final[Path] = terrain_config_dir / "material_rules.example.ron"
