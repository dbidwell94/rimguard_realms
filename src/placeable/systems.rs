use super::{components::*, RequestPlacementEvent};
use crate::{utils::*, TILE_SIZE};
use bevy::{prelude::*, utils::hashbrown::HashMap};
use leafwing_input_manager::prelude::*;

const PLACING_Z_INDEX: f32 = 2.;

#[derive(Component)]
pub struct Cursor;

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
        .insert((NowPlacing, TempPlaceholder, Cursor));
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
    q_temp_placeable: Query<
        (Entity, &GlobalTransform),
        (With<NowPlacing>, With<TempPlaceholder>, Without<Cursor>),
    >,
    mut request_placement: EventWriter<RequestPlacementEvent>,
    mut gizmos: Gizmos,
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
    let grid_direction: GridPos;

    let (start, end) = if diff_x > diff_y {
        // check left / right direction
        if cursor_pos.x > zoop_start.x {
            // right
            grid_direction = GridPos::new(1, 0);
        } else {
            // left
            grid_direction = GridPos::new(-1, 0);
        }
        (
            GridPos::from_tile_pos_vec(zoop_start),
            GridPos::from_tile_pos_vec(Vec2::new(cursor_pos.x, zoop_start.y)),
        )
    } else {
        // check up / down direction
        if cursor_pos.y > zoop_start.y {
            // up
            grid_direction = GridPos::new(0, 1);
        } else {
            // down
            grid_direction = GridPos::new(0, -1);
        }
        (
            GridPos::from_tile_pos_vec(zoop_start),
            GridPos::from_tile_pos_vec(Vec2::new(zoop_start.x, cursor_pos.y)),
        )
    };

    let mut vectors_to_place = if is_tileable {
        // get all the vectors between the start and end
        let mut vectors_to_place = HashMap::new();

        let mut current = start;
        while current != end {
            vectors_to_place.insert(current, false);
            current += grid_direction;
        }
        if GridPos::from_tile_pos_vec(cursor_pos) != end {
            vectors_to_place.insert(end, false);
        }
        vectors_to_place
    } else {
        let mut vectors_to_place = HashMap::new();
        vectors_to_place.insert(end, false);
        vectors_to_place
    };

    // If we release the mouse button, we want to place the item(s) using the event, and discontinue all below logic
    if input.just_released(crate::Input::Select) {
        zoop_start_location.0 = None;

        // convert the hashmap above into a Vec of bundles, with the correct transforms applied to them
        let mut bundles = Vec::new();
        for (tile_pos, _) in vectors_to_place {
            // first, ensure tile is walkable
            let nav_tile = &navmesh.0[tile_pos.x as usize][tile_pos.y as usize];
            if !nav_tile.walkable {
                continue;
            }
            let nav_tile_is_wall = nav_tile.occupied_by.iter().any(|entity| {
                // if the entity exists in this query, it's a wall
                q_walls.get(*entity).is_ok()
            });

            // if the tile is a wall, and the item is not placeable on a wall, skip it
            if nav_tile_is_wall && !item.placeable.0.placeable_on_wall() {
                continue;
            }

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
        let tile_pos = transform.translation().xy();
        let grid_pos = GridPos::from_tile_pos_vec(tile_pos.world_pos_to_tile());
        if !vectors_to_place.contains_key(&grid_pos) {
            gizmos.circle_2d(tile_pos, TILE_SIZE * 2., Color::RED);

            let entity_transform = q_temp_placeable
                .get_component::<GlobalTransform>(entity)
                .unwrap();
            let translation = entity_transform.translation();
            info!(
                "Despawning entity: {:?} at {:?} -- grid: {:?} -- cursor at: {:?}",
                entity, translation, grid_pos, cursor_pos
            );

            commands.entity(entity).despawn_recursive();
        } else {
            // Not worried about `unwrap()` here because we already checked to see if it exists above
            if let Some(is_placed) = vectors_to_place.get_mut(&grid_pos) {
                *is_placed = true;
            } else {
                panic!(
                    "Somehow the tile_pos {:?} was not in the vectors_to_place",
                    tile_pos
                );
            }
        }
    }

    let any_are_false = vectors_to_place.iter().any(|(_, is_placed)| !is_placed);

    if any_are_false {
        let all_translations = q_temp_placeable
            .iter()
            .map(|(_, transform)| GridPos::from_world_pos_vec(transform.translation().xy()))
            .collect::<Vec<_>>();
        let all_to_place = vectors_to_place
            .iter()
            .map(|(grid_pos, _)| grid_pos)
            .collect::<Vec<_>>();
        //     info!("all_translations: {:?}", all_translations);
        //     info!("----------------------------------");
        //     info!("all_to_place: {:?}", all_to_place);
    }

    let mut batch_to_spawn = Vec::new();

    // loop through the vectors_to_place and spawn any that are not already spawned
    for (tile_pos, is_spawned) in vectors_to_place {
        if is_spawned {
            continue;
        }
        let tile_pos_vec = tile_pos.to_vec2();
        let tile_pos_world = tile_pos_vec.tile_pos_to_world();

        let mut bundle = item.clone_bundle_dyn();
        bundle.sprite_bundle.transform =
            Transform::from_translation(tile_pos_world.extend(PLACING_Z_INDEX));

        // Change the sprite color to be more transparent (to indicate that we are placing)
        bundle.sprite_bundle.sprite.color = Color::rgba(1., 1., 1., 0.85);

        info!("spawning bundle at world pos: {:?}", bundle.sprite_bundle.transform.translation);

        // TODO: Check if the tile is walkable

        batch_to_spawn.push((bundle, NowPlacing, TempPlaceholder));
    }
    if batch_to_spawn.len() > 0 {
        commands.spawn_batch(batch_to_spawn);
    }
}
