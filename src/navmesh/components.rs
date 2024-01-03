use crate::utils::*;
use bevy::{
    prelude::*,
    utils::{hashbrown::HashSet, HashMap},
};

#[derive(Debug, Default, Resource)]
pub struct ToggleNavmeshDebug(pub bool);

#[derive(Debug, Event)]
pub struct PathfindRequest {
    pub start: GridPos,
    pub end: GridPos,
    pub entity: Entity,
}

#[derive(Debug, Event)]
pub struct PathfindAnswer {
    pub path: Option<Vec<GridPos>>,
    pub entity: Entity,
    pub target: GridPos,
}

#[derive(Component)]
pub struct SpatialWatch;

// create a spacial grid which will hold all the entities
#[derive(Resource, Default)]
pub struct SpatialGrid {
    grid: HashMap<GridPos, HashMap<Entity, SpatialEntity>>,
}

impl SpatialGrid {
    pub fn new() -> Self {
        Self {
            grid: HashMap::default(),
        }
    }

    pub fn grid(&self) -> &HashMap<GridPos, HashMap<Entity, SpatialEntity>> {
        &self.grid
    }

    pub fn create_spatial_entity(
        &mut self,
        entity: Entity,
        position: GridPos,
        walkable: bool,
        move_cost: Option<i32>,
        size: Option<(u8, u8)>,
    ) -> SpatialEntity {
        let size = size.unwrap_or((1, 1));
        let origin_spatial_entity = SpatialEntity {
            entity,
            position,
            previous_position: position,
            move_cost: move_cost.unwrap_or(0),
            walkable,
            size,
        };

        for x in position.x..position.x + size.0 as i32 {
            for y in position.y..position.y + size.1 as i32 {
                self.grid
                    .entry(GridPos::new(x, y))
                    .or_insert_with(HashMap::default)
                    .insert(entity, origin_spatial_entity);
            }
        }

        origin_spatial_entity
    }

    pub fn get(&self, entity: &Entity, pos: GridPos) -> Option<SpatialEntity> {
        self.grid
            .get(&pos)
            .and_then(|map| map.get(entity).map(|v| *v))
    }

    pub fn get_mut(&mut self, entity: &Entity, pos: GridPos) -> Option<&mut SpatialEntity> {
        self.grid.get_mut(&pos).and_then(|map| map.get_mut(entity))
    }

    pub fn remove(&mut self, entity: &Entity, pos: &GridPos) -> bool {
        self.grid
            .get_mut(pos)
            .map(|map| map.remove(entity).is_some())
            .unwrap_or(false)
    }

    pub fn update(&mut self, mut entity: SpatialEntity, world_pos: Vec2) {
        let _span = info_span!("SpatialGrid::update").entered();

        let new_pos = GridPos::from_world_pos_vec(world_pos);

        if new_pos != entity.position {
            let size = entity.size;
            for x in entity.position.x..entity.position.x + size.0 as i32 {
                for y in entity.position.y..entity.position.y + size.1 as i32 {
                    self.grid
                        .get_mut(&GridPos::new(x, y))
                        .unwrap()
                        .remove(entity.entity());
                }
            }
            entity.previous_position = entity.position;
            entity.position = new_pos;

            for x in new_pos.x..new_pos.x + size.0 as i32 {
                for y in new_pos.y..new_pos.y + size.1 as i32 {
                    self.grid
                        .entry(GridPos::new(x, y))
                        .or_insert_with(HashMap::default)
                        .insert(entity.entity, entity);
                }
            }
        }
    }

    pub fn get_entities_at(
        &self,
        position: &GridPos,
    ) -> impl std::iter::Iterator<Item = &SpatialEntity> {
        let _span = info_span!("SpatialGrid::get_entities_at").entered();

        let iter = self
            .grid
            .get(position)
            .map(|map| map.values().collect::<HashSet<_>>())
            .unwrap_or_default();

        iter.into_iter()
    }

    pub fn get_entities_in_range(
        &self,
        position: &GridPos,
        range: i32,
    ) -> impl std::iter::Iterator<Item = &SpatialEntity> {
        let _span = info_span!("SpatialGrid::get_entities_in_range").entered();

        let mut to_return = HashSet::new();

        for x in position.x - range..=position.x + range {
            for y in position.y - range..=position.y + range {
                if let Some(entities) = self.grid.get(&GridPos::new(x, y)) {
                    to_return.extend(entities.values());
                }
            }
        }

        to_return.into_iter()
    }

    pub fn walk_cost_at(&self, pos: &GridPos) -> i32 {
        let _span = info_span!("SpatialGrid::walk_cost_at").entered();

        self.grid
            .get(pos)
            .map(|map| map.values().map(|e| e.move_cost).sum())
            .unwrap_or(0)
    }

    pub fn is_walkable(&self, pos: &GridPos) -> bool {
        let _span = info_span!("SpatialGrid::is_walkable").entered();

        self.grid
            .get(pos)
            .map(|map| map.values().all(|e| e.walkable))
            .unwrap_or(false)
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub struct SpatialEntity {
    entity: Entity,
    position: GridPos,
    previous_position: GridPos,
    move_cost: i32,
    walkable: bool,
    size: (u8, u8),
}

// only hash the entity, don't care about the position
impl core::hash::Hash for SpatialEntity {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.entity.hash(state);
    }
}

impl SpatialEntity {
    pub fn entity(&self) -> &Entity {
        &self.entity
    }

    pub fn position(&self) -> &GridPos {
        &self.position
    }

    pub fn move_cost(&self) -> i32 {
        self.move_cost
    }

    pub fn set_move_cost(&mut self, move_cost: i32) {
        self.move_cost = move_cost;
    }

    pub fn walkable(&self) -> bool {
        self.walkable
    }

    pub fn set_walkable(&mut self, walkable: bool) {
        self.walkable = walkable;
    }

    pub fn watch(&self) -> SpatialWatch {
        SpatialWatch
    }
}
