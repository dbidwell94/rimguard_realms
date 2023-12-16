use super::{components::*, RequestPlacementEvent};
use crate::utils::*;
use bevy::{prelude::*, utils::hashbrown::HashMap};
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
    (cursor_pos, mut zoop_start_location, placeable_item, navmesh): (
        Res<crate::CursorPosition>,
        ResMut<super::ZoopStartLocation>,
        Res<super::CurrentPlaceableItem>,
        Res<crate::navmesh::Navmesh>,
    ),
    q_input: Query<&ActionState<crate::Input>>,
    q_walls: Query<&Placeable<Wall>>,
    q_temp_placeable: Query<(Entity, &Transform), (With<NowPlacing>, With<TempPlaceholder>)>,
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

    // get a straight line from the zoop start location to the cursor location, using the greater of x or y as the direction

    let diff_x = (cursor_pos.x - zoop_start.x).abs();
    let diff_y = (cursor_pos.y - zoop_start.y).abs();

    let (start, end) = if diff_x > diff_y {
        (zoop_start, Vec2::new(cursor_pos.x, zoop_start.y))
    } else {
        (zoop_start, Vec2::new(zoop_start.x, cursor_pos.y))
    };

    let placement_direction = (end - start).normalize_or_zero();

    let mut vectors_to_place = if is_tileable {
        // get all the vectors between the start and end
        let mut vectors_to_place = HashMap::new();

        let mut current = start;
        while current != end {
            vectors_to_place.insert(GridPos::from_tile_pos_vec(current), false);
            current += placement_direction;
        }
        vectors_to_place.insert(GridPos::from_tile_pos_vec(end), false);
        vectors_to_place
    } else {
        let mut vectors_to_place = HashMap::new();
        vectors_to_place.insert(GridPos::from_tile_pos_vec(end), false);
        vectors_to_place
    };

    // If we release the mouse button, we want to place the item(s) using the event, and discontinue all below logic
    if input.just_released(crate::Input::Select) {
        zoop_start_location.0 = None;

        // convert the hashmap above into a Vec of bundles, with the correct transforms applied to them
        let mut bundles = Vec::new();
        for (tile_pos, _) in vectors_to_place {
            let tile_pos_vec = tile_pos.to_vec2();
            let tile_pos_world = tile_pos_vec.tile_pos_to_world();

            let mut bundle = item.clone_bundle_dyn();

            // Change transform to be at the tile_pos
            bundle.sprite_bundle.transform.translation = tile_pos_world.extend(PLACING_Z_INDEX);
            bundles.push(bundle);
        }

        request_placement.send(RequestPlacementEvent(bundles));

        return;
    }

    // loop though q_temp_placeable and check if any of the entities are in the vectors_to_place
    for (entity, transform) in &q_temp_placeable {
        // if they are not, despawn
        let tile_pos = GridPos::from_world_pos_vec(transform.translation.truncate());
        if !vectors_to_place.contains_key(&tile_pos) {
            commands.entity(entity).despawn_recursive();
        } else {
            // Not worried about `unwrap()` here because we already checked to see if it exists above
            *(vectors_to_place.get_mut(&tile_pos).unwrap()) = true;
        }
    }

    // loop through the vectors_to_place and spawn any that are not already spawned
    for (tile_pos, is_spawned) in vectors_to_place {
        if is_spawned {
            continue;
        }
        let tile_pos_vec = tile_pos.to_vec2();
        let tile_pos_world = tile_pos_vec.tile_pos_to_world();

        let mut bundle = item.clone_bundle_dyn();

        // Change the sprite color to be more transparent (to indicate that we are placing)
        bundle.sprite_bundle.sprite.color = Color::rgba(1., 1., 1., 0.85);

        // Change transform to be at the tile_pos
        bundle.sprite_bundle.transform.translation = tile_pos_world.extend(PLACING_Z_INDEX);

        // TODO: Check if the tile is walkable

        commands.spawn(bundle).insert((NowPlacing, TempPlaceholder));
    }
}
