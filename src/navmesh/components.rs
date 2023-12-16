use crate::utils::*;
use crate::SIZE;
use bevy::{prelude::*, utils::HashSet};

#[derive(Debug, Default, Resource)]
pub struct ToggleNavmeshDebug(pub bool);

#[derive(Debug, Default)]
pub struct NavTileOccupant {
    pub weight: f32,
    pub occupied_by: HashSet<Entity>,
    pub walkable: bool,
}

#[derive(Resource)]
pub struct Navmesh(pub Vec<Vec<NavTileOccupant>>);

impl Default for Navmesh {
    fn default() -> Self {
        let to_return = (0..SIZE)
            .map(|_| {
                (0..SIZE)
                    .map(|_| NavTileOccupant::default())
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        Self(to_return)
    }
}

#[derive(Debug, Event)]
pub struct PathfindRequest {
    pub start: Vec2,
    pub end: Vec2,
    pub entity: Entity,
}

pub type PathfindingQuery<'world, 'state> = Query<'world, 'state, Entity>;

impl PathfindRequest {
    pub fn from_query(
        requesting: Entity,
        to_entity: Entity,
        query: &PathfindingQuery,
    ) -> Option<Self> {
        let requesting_transform = query.get_component::<Transform>(requesting).ok()?;
        let to_entity_transform = query.get_component::<Transform>(to_entity).ok()?;

        Some(Self {
            start: requesting_transform
                .translation
                .truncate()
                .world_pos_to_tile(),
            end: to_entity_transform
                .translation
                .truncate()
                .world_pos_to_tile(),
            entity: requesting,
        })
    }
}

#[derive(Debug, Event)]
pub struct PathfindAnswer {
    pub path: Option<Vec<Vec2>>,
    pub entity: Entity,
    pub target: Vec2,
}
