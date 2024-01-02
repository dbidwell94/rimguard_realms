use super::components::*;
use crate::utils::*;
use crate::TILE_SIZE;
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;
use pathfinding::prelude::*;

pub fn debug_navmesh(
    navmesh: Res<SpatialGrid>,
    mut toggle_debug: ResMut<ToggleNavmeshDebug>,
    mut gizmos: Gizmos,
    input: Query<&ActionState<crate::Input>>,
) {
    let Ok(input) = input.get_single() else {
        return;
    };

    if input.just_pressed(crate::Input::Debug) {
        toggle_debug.0 = !toggle_debug.0;
    }

    if !toggle_debug.0 {
        return;
    }

    for pos in navmesh.grid().keys() {
        let tile_position = Vec2::new(pos.x as f32, pos.y as f32).tile_pos_to_world()
            + Vec2::new(TILE_SIZE / 2., TILE_SIZE / 2.);

        let walkable = navmesh.is_walkable(pos);

        if !walkable {
            gizmos.rect_2d(
                tile_position,
                0.,
                Vec2::new(TILE_SIZE, TILE_SIZE),
                Color::RED,
            );
        } else {
            let weight_color = Color::GREEN;
            gizmos.rect_2d(
                tile_position,
                0.,
                Vec2::new(TILE_SIZE, TILE_SIZE),
                weight_color,
            );
        }
    }
}

pub fn listen_for_pathfinding_requests(
    mut pathfinding_event_reader: EventReader<PathfindRequest>,
    navmesh: Res<SpatialGrid>,
    mut pathfinding_event_writer: EventWriter<PathfindAnswer>,
) {
    let navmesh_grid = navmesh.grid();
    for request in pathfinding_event_reader.read() {
        let GridPos { x: end_x, y: end_y } = request.end;

        let result = astar(
            &request.start,
            |&GridPos { x, y }| {
                let up = GridPos::new(x, y.saturating_add(1));
                let down = GridPos::new(x, y.saturating_sub(1));
                let left = GridPos::new(x.saturating_sub(1), y);
                let right = GridPos::new(x.saturating_add(1), y);

                [up, down, left, right]
                    .iter()
                    .filter(|&pos| {
                        // check neighbor cell to see if it's walkable or it's the end cell
                        navmesh_grid
                            .get(pos)
                            .map(|contents| {
                                let mut walkable_array = contents.values().map(|v| v.walkable());
                                (pos.x == end_x && pos.y == end_y) || !walkable_array.any(|v| !v)
                            })
                            .unwrap_or(false)
                    })
                    .map(|pos| {
                        (
                            *pos,
                            // Add the total cell walk cost to the distance required to get to the end cell
                            navmesh.walk_cost_at(pos)
                                + (Vec2::new(pos.x as f32, pos.y as f32)
                                    - Vec2::new(end_x as f32, end_y as f32))
                                .length() as i32,
                        )
                    })
                    .collect::<Vec<_>>()
            },
            |&tile| {
                (Vec2::new(tile.x as f32, tile.y as f32) - Vec2::new(end_x as f32, end_y as f32))
                    .length() as i32
            },
            |GridPos { x, y }| x == &end_x && y == &end_y,
        )
        .map(|(data, _)| data);

        pathfinding_event_writer.send(PathfindAnswer {
            path: result,
            entity: request.entity,
            target: request.end,
        });
    }
}

pub fn update_spatial_grid(
    q_spatial: Query<(Entity, &Transform), (With<SpatialWatch>, Changed<Transform>)>,
    mut navmesh: ResMut<SpatialGrid>,
) {
    for (entity, transform) in q_spatial.iter() {
        if let Some(ent) = navmesh.get(
            &entity,
            GridPos::from_world_pos_vec(transform.translation.truncate()),
        ) {
            navmesh.update(ent, transform.translation.truncate());
        }
    }
}
