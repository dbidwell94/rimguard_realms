use bevy::{ecs::system::EntityCommands, prelude::*};

pub enum SelectableType {
    Pawn(crate::pawn::components::Pawn),
}

#[derive(Component)]
pub struct Selectable;

#[derive(Component)]
pub struct Selected;

#[derive(Bundle)]
pub struct SelectedBundle {
    pub selected: Selected,
    pub aabb_gizmo: AabbGizmo,
}

impl Default for SelectedBundle {
    fn default() -> Self {
        Self {
            selected: Selected,
            aabb_gizmo: AabbGizmo {
                color: Some(Color::WHITE),
            },
        }
    }
}

pub trait SelectEntity {
    fn select(&mut self) -> &mut Self;
    fn deselect(&mut self) -> &mut Self;
}

impl SelectEntity for EntityCommands<'_, '_, '_> {
    fn select(&mut self) -> &mut Self {
        self.try_insert(SelectedBundle::default())
    }

    fn deselect(&mut self) -> &mut Self {
        self.remove::<SelectedBundle>()
    }
}
