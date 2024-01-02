use crate::utils::GridPos;

use super::{PathfindRequest, SpatialGrid};
use bevy::prelude::*;
use pathfinding::prelude::*;

pub fn get_pathing(request: PathfindRequest, navmesh: &Res<SpatialGrid>) -> Option<Vec<GridPos>> {
    let navmesh_grid = navmesh.grid();

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
    result
}
