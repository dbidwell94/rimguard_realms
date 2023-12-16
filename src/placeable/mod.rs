pub mod components;
mod systems;

use bevy::prelude::*;

use self::components::{PlaceableItem, PlaceableBundle};

pub struct PlaceablePlugin;

impl Plugin for PlaceablePlugin {
    fn build(&self, app: &mut App) {}
}

pub struct CurrentPlaceableItem(pub Option<PlaceableBundle<dyn PlaceableItem>>);