pub mod components;
mod systems;

use bevy::prelude::*;

use crate::{CursorPosition, WorldInteraction};

use self::components::{PlaceableBundle, PlaceableItem};

pub struct PlaceablePlugin;

impl Plugin for PlaceablePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CurrentPlaceableItem>()
            .add_systems(
                Update,
                systems::show_placeable_item.run_if(resource_changed::<CurrentPlaceableItem>()),
            )
            .add_systems(
                Update,
                systems::change_placeable_item_position
                    .run_if(resource_changed::<CursorPosition>()),
            )
            .add_systems(
                OnExit(WorldInteraction::Placing),
                systems::remove_placing_if_no_longer_placing,
            );
    }
}

#[derive(Resource, Default)]
pub struct CurrentPlaceableItem(pub Option<PlaceableBundle<dyn PlaceableItem>>);
