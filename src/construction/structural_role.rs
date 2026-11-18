//! STRUCT-ROLE — footprint tile load vs facade tags (cascade consumer later).

use bevy::prelude::*;

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

    pub fn ensure_uniform(&mut self, n: usize, role: StructuralRole) {
        if self.roles.len() == n {
            return;
        }
        self.roles = vec![role; n];
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
