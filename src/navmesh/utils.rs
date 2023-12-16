use super::{Navmesh, PathfindRequest};
use bevy::prelude::*;
use pathfinding::prelude::*;

pub fn get_pathing(request: PathfindRequest, navmesh: &Res<Navmesh>) -> Option<Vec<Vec2>> {
    let Vec2 { x, y } = request.start;
    let start_x = x as usize;
    let start_y = y as usize;

    let Vec2 { x: end_x, y: end_y } = request.end;
    let end_x = end_x as usize;
    let end_y = end_y as usize;

    let result = astar(
        &(start_x, start_y),
        |&(x, y)| {
            let up = (x, y.saturating_add(1));
            let down = (x, y.saturating_sub(1));
            let left = (x.saturating_sub(1), y);
            let right = (x.saturating_add(1), y);

            let neighbors = [up, down, left, right]
                .iter()
                .filter(|&(x, y)| {
                    navmesh
                        .0
                        .get(*x)
                        .and_then(|row| row.get(*y))
                        .map(|tile| {
                            tile.walkable
                                || (*x == end_x && *y == end_y)
                                || (*x == start_x && *y == start_y)
                        })
                        .unwrap_or(false)
                })
                .map(|(x, y)| ((*x, *y), 0)) // Modify this line
                .collect::<Vec<_>>();

            neighbors
        },
        |&(x, y)| {
            (Vec2::new(x as f32, y as f32) - Vec2::new(end_x as f32, end_y as f32)).length() as i32
        },
        |(x, y)| x == &end_x && y == &end_y,
    )
    .map(|(data, _)| {
        data.iter()
            .map(|item| Vec2::new(item.0 as f32, item.1 as f32))
            .collect::<Vec<_>>()
    });

    result
}
