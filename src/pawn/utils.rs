use crate::{
    navmesh::{
        prelude::{PathfindRequest, SpatialEntity},
        utils::get_pathing,
        SpatialGrid,
    },
    stone::StoneKind,
    utils::GridPos,
};
use bevy::prelude::*;

/// Find the closest stone to the given position. Traverses the neighbor stones to find the closest to the given position.
pub fn find_closest_stone(
    q_stones: &Query<Entity, With<StoneKind>>,
    closest_to: &GridPos,
    starting_stone: &SpatialEntity,
    navmesh: &Res<SpatialGrid>,
) -> Option<Entity> {
    let _ = info_span!("find_closest_stone").entered();

    // leverage A* to walk from the starting stone to the 'clostest_to' position, return the last stone we found as that will be the one that will not be blocking the path

    let Some(pathing) = get_pathing(
        PathfindRequest {
            end: *starting_stone.position(),
            entity: Entity::PLACEHOLDER,
            start: *closest_to,
        },
        navmesh,
        true,
    ) else {
        return None;
    };

    // return the first stone found in the pathing starting at the beginning of the pathing.
    // This will ensure that it's the closest stone to the starting position
    pathing.iter().find_map(|pos| {
        navmesh.get_entities_at(pos).find_map(|e| {
            if q_stones.get(*e.entity()).is_ok() {
                Some(e.entity().clone())
            } else {
                None
            }
        })
    })
}
