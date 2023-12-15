mod components;
mod systems;

use bevy::prelude::*;
pub use components::*;

pub struct SelectablePlugin;

impl Plugin for SelectablePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, systems::select_selectables_in_bounds);
    }
}
