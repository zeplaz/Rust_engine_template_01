//! STRUCT-ROLE — footprint tile load vs facade tags (cascade consumer later).

use bevy::prelude::*;

use super::demolish::DestructibleDamage;
use crate::strategic::{ConstructionSite, SiteArchetype, SiteFootprint};

/// Load-bearing vs cosmetic role for one footprint tile (parallel to [`crate::strategic::SiteFootprint::tiles`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Reflect, Default)]
#[reflect(Default)]
pub enum StructuralRole {
    #[default]
    Foundation,
    Column,
    Facade,
    Trim,
}

impl StructuralRole {
    #[must_use]
    pub fn is_load_bearing(self) -> bool {
        matches!(self, Self::Foundation | Self::Column)
    }
}

/// Per-tile roles for a construction site — same order as footprint tiles.
#[derive(Component, Clone, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct SiteStructuralRoles {
    pub roles: Vec<StructuralRole>,
}

impl SiteStructuralRoles {
    /// Resize / fill roles for `n` footprint tiles. Dam-like spans: ends Foundation, mid Column.
    pub fn ensure_dam_span(&mut self, n: usize) {
        if self.roles.len() == n && n > 0 {
            return;
        }
        self.roles = (0..n)
            .map(|i| {
                if n <= 2 || i == 0 || i + 1 == n {
                    StructuralRole::Foundation
                } else {
                    StructuralRole::Column
                }
            })
            .collect();
    }

    /// Default roles for an ordinary building footprint (D-2), from tile adjacency inside the
    /// footprint: a corner / end tile (≤ 2 footprint neighbours) is the load-bearing
    /// `Foundation`, a remaining perimeter tile (3 neighbours) is a load wall (`Column`), an
    /// interior tile (4 neighbours) is the non-structural floor (`Facade`).
    pub fn ensure_building_default(&mut self, tiles: &[IVec2]) {
        if self.roles.len() == tiles.len() && !tiles.is_empty() {
            return;
        }
        let set: std::collections::HashSet<IVec2> = tiles.iter().copied().collect();
        self.roles = tiles
            .iter()
            .map(|t| {
                let neighbours = [IVec2::X, IVec2::NEG_X, IVec2::Y, IVec2::NEG_Y]
                    .iter()
                    .filter(|d| set.contains(&(*t + **d)))
                    .count();
                match neighbours {
                    0..=2 => StructuralRole::Foundation,
                    3 => StructuralRole::Column,
                    _ => StructuralRole::Facade,
                }
            })
            .collect();
    }

    pub fn ensure_uniform(&mut self, n: usize, role: StructuralRole) {
        if self.roles.len() == n {
            return;
        }
        self.roles = vec![role; n];
    }
}

/// D-2: every committed non-dam site gets default roles (+ a damage ladder) once its footprint
/// exists, so the flood proxy and the cascade reach ordinary buildings. Dams keep their own
/// span roles (`attach_dam_body_on_commit_system`).
pub fn ensure_default_structural_roles_system(
    mut commands: Commands,
    q: Query<
        (Entity, &ConstructionSite, &SiteFootprint, Has<DestructibleDamage>),
        Without<SiteStructuralRoles>,
    >,
) {
    for (e, site, fp, has_damage) in &q {
        if site.archetype == SiteArchetype::Dam || fp.tiles.is_empty() {
            continue;
        }
        let mut roles = SiteStructuralRoles::default();
        roles.ensure_building_default(&fp.tiles);
        let mut ec = commands.entity(e);
        ec.insert(roles);
        if !has_damage {
            ec.insert(DestructibleDamage::default());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn struct_role_building_default_corner_edge_interior() {
        let tiles: Vec<IVec2> = (0..3)
            .flat_map(|y| (0..3).map(move |x| IVec2::new(x, y)))
            .collect();
        let mut roles = SiteStructuralRoles::default();
        roles.ensure_building_default(&tiles);
        let at = |x: i32, y: i32| {
            roles.roles[tiles.iter().position(|t| *t == IVec2::new(x, y)).unwrap()]
        };
        assert_eq!(at(0, 0), StructuralRole::Foundation);
        assert_eq!(at(1, 0), StructuralRole::Column);
        assert_eq!(at(1, 1), StructuralRole::Facade);
        let mut one = SiteStructuralRoles::default();
        one.ensure_building_default(&[IVec2::new(5, 5)]);
        assert_eq!(one.roles, vec![StructuralRole::Foundation]);
    }

    #[test]
    fn ordinary_site_gets_roles_dam_does_not() {
        use crate::strategic::{ConstructionSite, LayerType, SiteConstructionPhase};
        let mut app = App::new();
        app.add_systems(Update, ensure_default_structural_roles_system);
        let mk = |archetype| {
            (
                ConstructionSite {
                    site_id: 1,
                    owner: Entity::PLACEHOLDER,
                    archetype,
                    phase: SiteConstructionPhase::default(),
                    operational_readiness: 1.0,
                },
                SiteFootprint {
                    tiles: vec![IVec2::new(0, 0), IVec2::new(1, 0)],
                    layer: LayerType::Surface,
                },
            )
        };
        let house = app.world_mut().spawn(mk(SiteArchetype::Factory)).id();
        let dam = app.world_mut().spawn(mk(SiteArchetype::Dam)).id();
        app.update();
        let w = app.world();
        assert_eq!(w.get::<SiteStructuralRoles>(house).unwrap().roles.len(), 2);
        assert!(w.get::<DestructibleDamage>(house).is_some());
        assert!(w.get::<SiteStructuralRoles>(dam).is_none());
    }

    #[test]
    fn struct_role_dam_span_ends_are_foundation() {
        let mut roles = SiteStructuralRoles::default();
        roles.ensure_dam_span(5);
        assert_eq!(roles.roles.len(), 5);
        assert_eq!(roles.roles[0], StructuralRole::Foundation);
        assert_eq!(roles.roles[4], StructuralRole::Foundation);
        assert_eq!(roles.roles[2], StructuralRole::Column);
        assert!(StructuralRole::Column.is_load_bearing());
        assert!(!StructuralRole::Facade.is_load_bearing());
    }
}
