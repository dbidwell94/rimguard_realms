pub mod components;
mod systems;

use bevy::prelude::*;

pub struct PlaceablePlugin;

impl Plugin for PlaceablePlugin {
    fn build(&self, app: &mut App) {}
}
