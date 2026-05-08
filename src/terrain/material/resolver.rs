use super::registry::{MaterialId, MaterialRegistry};
use super::rules::RuleSet;
use super::tags::{TagRegistry, TagSet};
use crate::terrain::biome::BiomeWeights;
use crate::terrain::family::TerrainFamilyId;

/// Pick a [`MaterialId`] from `(family, tags, rules, registries)` — deterministic (rules pre-sorted).
///
/// `_weights` is reserved for future `weight_predicate` / scoring (`§48`); pass [`BiomeWeights::default()`] until then.
pub fn resolve_material(
    family: TerrainFamilyId,
    weights: &BiomeWeights,
    tags: TagSet,
    rules: &RuleSet,
    registry: &MaterialRegistry,
    tag_registry: &TagRegistry,
) -> MaterialId {
    resolve_material_with_rule_index(family, weights, tags, rules, registry, tag_registry).0
}

/// `rule_index` from the winning rule file order or `u32::MAX` when the family default was used.
pub fn resolve_material_with_rule_index(
    family: TerrainFamilyId,
    _weights: &BiomeWeights,
    tags: TagSet,
    rules: &RuleSet,
    registry: &MaterialRegistry,
    tag_registry: &TagRegistry,
) -> (MaterialId, u32) {
    for rule in &rules.rules {
        if let Some(f) = rule.family_filter {
            if f != family {
                continue;
            }
        }
        if !rule_matches(&tags, rule, tag_registry) {
            continue;
        }
        let Some(id) = registry.name_to_id.get(&rule.result_name).copied() else {
            panic!(
                "resolve_material_with_rule_index: rule references unknown material {:?}",
                rule.result_name
            );
        };
        return (id, rule.rule_index);
    }

    let id = family_default(family, registry)
        .unwrap_or_else(|| panic!("resolve_material_with_rule_index: no materials for family {family:?}"));
    (id, u32::MAX)
}

fn rule_matches(tags: &TagSet, rule: &super::rules::MaterialRule, tag_registry: &TagRegistry) -> bool {
    for name in &rule.required {
        let Some(id) = tag_registry.tag_id(name) else {
            return false;
        };
        if !tags.contains(id) {
            return false;
        }
    }
    for name in &rule.forbidden {
        if let Some(id) = tag_registry.tag_id(name) {
            if tags.contains(id) {
                return false;
            }
        }
    }
    true
}

fn family_default(family: TerrainFamilyId, registry: &MaterialRegistry) -> Option<MaterialId> {
    registry
        .materials
        .iter()
        .enumerate()
        .find(|(_, m)| m.family == family)
        .map(|(i, _)| MaterialId(i as u16))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::terrain::family::TerrainFamilyRegistry;
    use crate::terrain::material::rules::MaterialRule;

    fn example_paths() -> (std::path::PathBuf, std::path::PathBuf, std::path::PathBuf) {
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        (
            root.join("assets/config/terrain/material_registry.example.json"),
            root.join("assets/config/terrain/tag_registry.example.json"),
            root.join("assets/config/terrain/material_rules.example.ron"),
        )
    }

    #[test]
    fn material_resolver_priority_tiebreak_deterministic() {
        let (mp, tp, _) = example_paths();
        let registry = MaterialRegistry::load_from_json(mp.to_str().unwrap()).unwrap();
        let tag_registry = TagRegistry::load_from_json(tp.to_str().unwrap()).unwrap();

        let fam = TerrainFamilyRegistry::load_from_json(
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("assets/config/terrain/terrain_family_registry.example.json")
                .to_str()
                .unwrap(),
        )
        .unwrap();
        let grass = fam.id("Grassland").unwrap();

        let mut tags = TagSet::default();
        for name in ["wet", "lowland", "fertile"] {
            tags.insert(tag_registry.tag_id(name).unwrap());
        }

        let mut rule_vec = vec![
            MaterialRule {
                required: vec!["wet".into(), "lowland".into()],
                forbidden: vec![],
                family_filter: None,
                result_name: "basalt_dense".into(),
                priority: 10,
                rule_index: 1,
            },
            MaterialRule {
                required: vec!["wet".into(), "lowland".into(), "fertile".into()],
                forbidden: vec!["rock".into()],
                family_filter: None,
                result_name: "loam_wet".into(),
                priority: 10,
                rule_index: 0,
            },
        ];
        rule_vec.sort_by(|a, b| {
            b.priority
                .cmp(&a.priority)
                .then_with(|| a.rule_index.cmp(&b.rule_index))
        });
        let rules = RuleSet {
            schema_version: 1,
            rules: rule_vec,
        };

        let id = resolve_material(
            grass,
            &BiomeWeights::default(),
            tags,
            &rules,
            &registry,
            &tag_registry,
        );
        assert_eq!(id, *registry.name_to_id.get("loam_wet").unwrap());
    }

    #[test]
    fn material_resolver_falls_back_to_family_default() {
        let (mp, tp, _) = example_paths();
        let registry = MaterialRegistry::load_from_json(mp.to_str().unwrap()).unwrap();
        let tag_registry = TagRegistry::load_from_json(tp.to_str().unwrap()).unwrap();
        let rules = RuleSet {
            schema_version: 1,
            rules: vec![],
        };

        let fam = TerrainFamilyRegistry::load_from_json(
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("assets/config/terrain/terrain_family_registry.example.json")
                .to_str()
                .unwrap(),
        )
        .unwrap();
        let grass = fam.id("Grassland").unwrap();

        let id = resolve_material(
            grass,
            &BiomeWeights::default(),
            TagSet::default(),
            &rules,
            &registry,
            &tag_registry,
        );
        assert_eq!(id, *registry.name_to_id.get("loam_wet").unwrap());
    }
}
