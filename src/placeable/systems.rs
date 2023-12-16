use super::components::NowPlacing;
use crate::utils::*;
use bevy::prelude::*;

const PLACING_Z_INDEX: f32 = 2.;

pub fn show_placeable_item(
    mut commands: Commands,
    placeable_item: Res<super::CurrentPlaceableItem>,
    mouse_pos: Res<crate::CursorPosition>,
    q_all_placeable: Query<Entity, With<NowPlacing>>,
) {
    let Some(ref item) = placeable_item.0 else {
        return;
    };

    let Some(mouse_pos) = mouse_pos.0 else {
        return;
    };

    for entity in &q_all_placeable {
        commands.entity(entity).despawn_recursive();
    }

    let mut to_place = item.clone_bundle_dyn();
    to_place.sprite_bundle.transform.translation =
        mouse_pos.tile_pos_to_world().extend(PLACING_Z_INDEX);

    commands.spawn(to_place).insert(NowPlacing);
}

pub fn change_placeable_item_position(
    mut q_placing: Query<&mut Transform, With<NowPlacing>>,
    cursor_pos: Res<crate::CursorPosition>,
) {
    let Some(cursor_pos) = cursor_pos.0 else {
        return;
    };

    // convert tile pos to world
    let coords = cursor_pos.tile_pos_to_world();

    for mut transform in &mut q_placing {
        transform.translation = coords.extend(PLACING_Z_INDEX);
    }
}

pub fn remove_placing_if_no_longer_placing(
    mut commands: Commands,
    q_placing: Query<Entity, With<NowPlacing>>,
) {
    for entity in &q_placing {
        commands.entity(entity).despawn_recursive();
    }
}

// place item in a walkable location (walkable means the tile is empty and can be placed there)
pub fn place_item_at_location() {}
