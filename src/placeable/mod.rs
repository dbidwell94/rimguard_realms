pub mod components;
mod systems;

use self::components::PlaceableBundle;
use crate::{utils::GridPos, CursorPosition, WorldInteraction};
use bevy::{prelude::*, utils::HashSet};

pub mod prelude {
    pub use super::components::{PlaceableItemExt, PlaceableType};
}

pub struct PlaceablePlugin;

impl Plugin for PlaceablePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CurrentPlaceableItem>()
            .init_resource::<ZoopStartLocation>()
            .init_resource::<ItemGridPlacement>()
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
                    systems::populate_item_grid_placement_res_and_send_spawn_event,
                )
                    .chain()
                    .run_if(in_state(WorldInteraction::Placing)),
            )
            .add_systems(
                Update,
                (
                    systems::handle_built_added,
                    systems::handle_built_removed,
                    systems::add_unbuilt_to_navmesh,
                    systems::check_if_unbuilt_has_been_finished,
                ),
            );
    }
}

#[derive(Resource, Default)]
pub struct CurrentPlaceableItem(pub Option<PlaceableBundle>);

#[derive(Event)]
pub struct RequestPlacementEvent(pub Vec<PlaceableBundle>);

#[derive(Resource, Default)]
struct ZoopStartLocation(pub Option<Vec2>);

#[derive(Resource, Default)]
struct ItemGridPlacement(HashSet<GridPos>);
