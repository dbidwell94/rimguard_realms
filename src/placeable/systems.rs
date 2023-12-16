use super::{components::*, RequestPlacementEvent};
use crate::utils::*;
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

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

    commands
        .spawn(to_place)
        .insert((NowPlacing, TempPlaceholder));
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

pub fn update_zoop_location(
    input: Query<&ActionState<crate::Input>>,
    mut zoop_location: ResMut<super::ZoopStartLocation>,
    mouse_location: Res<crate::CursorPosition>,
) {
    let Ok(input) = input.get_single() else {
        return;
    };

    let Some(mouse_location) = mouse_location.0 else {
        return;
    };

    if input.just_pressed(crate::Input::Select) {
        zoop_location.0 = Some(mouse_location);
    }
}

// place item in a walkable location (walkable means the tile is empty and can be placed there)
// Also: place on top of wall if `<dyn PlaceableItem>.placeable_on_wall()` is true
pub fn place_item_at_location(
    mut commands: Commands,
    (cursor_pos, zoop_start_location, placeable_item, navmesh): (
        Res<crate::CursorPosition>,
        Res<super::ZoopStartLocation>,
        Res<super::CurrentPlaceableItem>,
        Res<crate::navmesh::Navmesh>,
    ),
    q_input: Query<&ActionState<crate::Input>>,
    q_walls: Query<&Placeable<Wall>>,
    q_temp_placeable: Query<Entity, (With<NowPlacing>, With<TempPlaceholder>)>,
    mut request_placement: EventWriter<RequestPlacementEvent>,
) {
    let Ok(input) = q_input.get_single() else {
        return;
    };
    let Some(cursor_pos) = cursor_pos.0 else {
        return;
    };
    let Some(zoop_start) = zoop_start_location.0 else {
        return;
    };
    let Some(ref item) = placeable_item.0 else {
        return;
    };

    // check if the item is tileable
    let is_tileable = item.placeable.0.is_tileable();
    let placeable_on_wall = item.placeable.0.placeable_on_wall();

    // get a straight line from the zoop start location to the cursor location, using the greater of x or y as the direction

    let diff_x = (cursor_pos.x - zoop_start.x).abs();
    let diff_y = (cursor_pos.y - zoop_start.y).abs();

    let (start, end) = if diff_x > diff_y {
        (zoop_start, Vec2::new(cursor_pos.x, zoop_start.y))
    } else {
        (zoop_start, Vec2::new(zoop_start.x, cursor_pos.y))
    };
}
