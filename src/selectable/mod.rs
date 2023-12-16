mod components;
mod systems;

use bevy::prelude::*;
pub use components::*;

use crate::WorldInteraction;

pub struct SelectablePlugin;

impl Plugin for SelectablePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            systems::select_selectables_in_bounds.run_if(in_state(WorldInteraction::Selecting)),
        );
    }
}
