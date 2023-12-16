pub mod components;
mod systems;

use self::components::{PlaceableBundle, PlaceableItem};
use crate::{CursorPosition, WorldInteraction};
use bevy::prelude::*;

pub mod prelude {
    pub use super::components::{
        ClonePlaceableItem, PlaceableItem, PlaceableItemExt, PlaceableType,
    };
}

pub struct PlaceablePlugin;

impl Plugin for PlaceablePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CurrentPlaceableItem>()
            .init_resource::<ZoopStartLocation>()
            .add_event::<RequestPlacementEvent>()
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
            )
            .add_systems(
                Update,
                (
                    systems::update_zoop_location,
                    systems::place_item_at_location,
                )
                    .chain()
                    .run_if(in_state(WorldInteraction::Placing)),
            );
    }
}

#[derive(Resource, Default)]
pub struct CurrentPlaceableItem(pub Option<PlaceableBundle<dyn PlaceableItem>>);

#[derive(Event)]
pub struct RequestPlacementEvent(pub Vec<PlaceableBundle<dyn PlaceableItem>>);

#[derive(Resource, Default)]
struct ZoopStartLocation(pub Option<Vec2>);
