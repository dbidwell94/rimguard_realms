pub mod components;
pub mod systems;
mod utils;

use self::components::{PathfindAnswer, PathfindRequest, ToggleNavmeshDebug};
use bevy::prelude::*;
pub use components::Navmesh;

pub mod prelude {
    pub use super::components::*;
    pub use super::utils::get_pathing;
}

#[derive(SystemSet, Hash, Debug, Clone, Eq, PartialEq)]
pub enum NavmeshSystemSet {
    First,
    Update,
    Last,
}

pub struct NavmeshPlugin;

impl Plugin for NavmeshPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Navmesh>()
            .init_resource::<ToggleNavmeshDebug>()
            .configure_sets(
                Update,
                (
                    NavmeshSystemSet::First,
                    NavmeshSystemSet::Update,
                    NavmeshSystemSet::Last,
                )
                    .chain(),
            )
            .add_systems(
                Update,
                (
                    systems::debug_navmesh,
                    systems::listen_for_pathfinding_requests,
                )
                    .in_set(NavmeshSystemSet::Update),
            )
            .add_event::<PathfindRequest>()
            .add_event::<PathfindAnswer>();
    }
}
