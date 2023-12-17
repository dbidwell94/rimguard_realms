use super::{components::*, RequestPlacementEvent};
use crate::{utils::*, TILE_SIZE};
use bevy::{prelude::*, utils::hashbrown::HashSet};
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

    let mut to_place = item.clone();

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
pub fn populate_item_grid_placement_res_and_send_spawn_event(
    (cursor_pos, mut zoop_start_location, placeable_item, navmesh, mut locations_to_place): (
        Res<crate::CursorPosition>,
        ResMut<super::ZoopStartLocation>,
        Res<super::CurrentPlaceableItem>,
        Res<crate::navmesh::Navmesh>,
        ResMut<super::ItemGridPlacement>,
    ),
    q_input: Query<&ActionState<crate::Input>>,
    q_walls: Query<&PlaceableType, Without<Cursor>>,
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
    let is_tileable = item.placeable.is_tileable();

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

    let vectors_to_place = if is_tileable {
        // get all the vectors between the start and end
        let mut vectors_to_place = HashSet::new();

        let mut current = start;
        while current != end {
            vectors_to_place.insert(current);
            current += grid_direction;
            if !locations_to_place.0.contains(&current) {
                locations_to_place.0.insert(current);
            }
        }
        vectors_to_place.insert(end);
        if !locations_to_place.0.contains(&end) {
            locations_to_place.0.insert(end);
        }
        vectors_to_place
    } else {
        let mut vectors_to_place = HashSet::new();
        vectors_to_place.insert(end);
        if !locations_to_place.0.contains(&end) {
            locations_to_place.0.insert(end);
        }
        vectors_to_place
    };

    let mut to_remove: HashSet<GridPos> = HashSet::new();

    // loop through the locations_to_place before mutating it, adding items to the to_remove HashSet
    for tile_pos in &locations_to_place.0 {
        if !vectors_to_place.contains(tile_pos) {
            to_remove.insert(*tile_pos);
        }
    }

    // remove the items in the to_remove HashSet from the locations_to_place HashSet
    for tile_pos in &to_remove {
        locations_to_place.0.remove(tile_pos);
    }

    // If we release the mouse button, we want to place the item(s) using the event, and discontinue all below logic
    if input.just_released(crate::Input::Select) {
        zoop_start_location.0 = None;

        // convert the hashmap above into a Vec of bundles, with the correct transforms applied to them
        let mut bundles = Vec::new();
        for tile_pos in vectors_to_place {
            // first, ensure tile is walkable
            let nav_tile = &navmesh.0[tile_pos.x as usize][tile_pos.y as usize];
            if !nav_tile.walkable {
                continue;
            }
            let nav_tile_has_wall = nav_tile.occupied_by.iter().any(|entity| {
                // if the entity exists in this query, it's a wall

                let Ok(placeable_item) = q_walls.get(*entity) else {
                    return false;
                };
                if let PlaceableType::Wall(_) = placeable_item {
                    return true;
                }
                return false;
            });

            // if the tile is a wall, and the item is not placeable on a wall, skip it
            if nav_tile_has_wall && !item.placeable.placeable_on_wall() {
                continue;
            }

            let tile_pos_vec = tile_pos.to_vec2();
            let tile_pos_world = tile_pos_vec.tile_pos_to_world();

            let mut bundle = item.clone();

            // Change transform to be at the tile_pos
            bundle.sprite_bundle.transform.translation = tile_pos_world.extend(PLACING_Z_INDEX);
            bundles.push(bundle);
        }

        request_placement.send(RequestPlacementEvent(bundles));

        return;
    }

    // use gizmos to show the transforms to place
    for tile_pos in &locations_to_place.0 {
        let tile_pos_vec = tile_pos.to_vec2();
        let tile_pos_world = tile_pos_vec.tile_pos_to_world();

        gizmos.rect_2d(
            tile_pos_world + Vec2::new(TILE_SIZE / 2., TILE_SIZE / 2.),
            0.,
            Vec2::new(TILE_SIZE, TILE_SIZE),
            Color::WHITE,
        );
    }
}

pub fn handle_built_added(
    mut navmesh: ResMut<crate::navmesh::Navmesh>,
    q_added: Query<(Entity, &GlobalTransform), Added<Built>>,
) {
    for (entity, transform) in &q_added {
        let tile_pos = transform.translation().world_pos_to_tile();

        let mesh_item = &mut navmesh.0[tile_pos.x as usize][tile_pos.y as usize];

        mesh_item.occupied_by.insert(entity);
        mesh_item.walkable = false;
    }
}

pub fn handle_built_removed(
    mut navmesh: ResMut<crate::navmesh::Navmesh>,
    mut removed_components: RemovedComponents<Built>,
    q_placeables: Query<(Entity, &GlobalTransform), With<PlaceableType>>,
    q_built: Query<Entity, With<Built>>,
) {
    for entity in removed_components.read() {
        if let Ok((_, transform)) = q_placeables.get(entity) {
            let tile_pos = transform.translation().world_pos_to_tile();

            let mesh_item = &mut navmesh.0[tile_pos.x as usize][tile_pos.y as usize];

            mesh_item.occupied_by.remove(&entity);
            // we have a built entity still in this nav tile, we don't want to make it walkable
            if q_built.get(entity).is_ok() {
                continue;
            }
            mesh_item.walkable = true;
        }
    }
}

pub fn add_unbuilt_to_navmesh(
    mut navmesh: ResMut<crate::navmesh::Navmesh>,
    q_unbuilt: Query<(Entity, &GlobalTransform), (Without<Built>, Added<PlaceableType>)>,
) {
    for (entity, transform) in &q_unbuilt {
        let tile_pos = transform.translation().world_pos_to_tile();

        let mesh_item = &mut navmesh.0[tile_pos.x as usize][tile_pos.y as usize];

        mesh_item.occupied_by.insert(entity);
    }
}

pub fn check_if_unbuilt_has_been_finished(
    mut commands: Commands,
    mut q_unbuilt: Query<(Entity, &mut Sprite, &PlaceableType), Without<Built>>,
) {
    for (entity, mut sprite, placeable) in &mut q_unbuilt {
        if placeable.get_missing_resource_count() == 0 {
            sprite.color = Color::WHITE;
            commands.entity(entity).insert(Built);
        }
    }
}
