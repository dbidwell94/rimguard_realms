pub mod components;
mod systems;
mod work_orders;

use crate::GameState;
use bevy::prelude::*;
use std::collections::VecDeque;

#[derive(SystemSet, Hash, Debug, Clone, Eq, PartialEq)]
pub enum PawnSystemSet {
    First,
    Move,
    Work,
    Attack,
    Last,
}

pub struct PawnPlugin;

impl Plugin for PawnPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(work_orders::WorkOrderPlugin)
            .add_systems(OnEnter(GameState::PawnSpawn), systems::initial_pawn_spawn)
            .init_resource::<WorkQueue>()
            .init_resource::<EnemyWave>()
            .register_type::<components::Pawn>()
            .register_type::<components::work_order::WorkOrder>()
            .register_type::<components::pawn_status::PawnStatus>()
            .register_type::<components::CarriedResources>()
            .add_event::<SpawnPawnRequestEvent>()
            .add_event::<RequestWorkOrder>()
            .add_event::<AttackEvent>()
            // setup systems scheduling
            .configure_sets(
                Update,
                (
                    PawnSystemSet::First,
                    PawnSystemSet::Move,
                    PawnSystemSet::Work,
                    PawnSystemSet::Attack,
                    PawnSystemSet::Last,
                )
                    .chain()
                    .run_if(in_state(GameState::Main))
                    .after(crate::navmesh::NavmeshSystemSet::Last),
            )
            // add work systems
            .add_systems(
                Update,
                (
                    systems::work_idle_pawns,
                    systems::build_placeable,
                    systems::pickup_stone_from_factory,
                    systems::mine_stone,
                    systems::return_to_factory,
                )
                    .chain()
                    .in_set(PawnSystemSet::Work),
            )
            // add attack systems
            .add_systems(
                Update,
                (systems::attack_pawn, systems::search_for_attack_target_pawn)
                    .chain()
                    .in_set(PawnSystemSet::Attack),
            )
            .add_systems(
                Update,
                (
                    systems::repath_if_navmesh_changes
                        .run_if(resource_changed::<crate::navmesh::Navmesh>()),
                    systems::retry_pathfinding,
                    systems::enemy_search_for_factory,
                    systems::listen_for_pathfinding_answers,
                    systems::move_pawn,
                )
                    .chain()
                    .in_set(PawnSystemSet::Move),
            )
            // add general systems
            .add_systems(
                Update,
                (
                    systems::update_health_ui,
                    systems::update_pawn_animation,
                    systems::listen_for_spawn_pawn_event,
                    systems::spawn_enemy_pawns,
                    systems::tick_timers,
                )
                    .chain()
                    .run_if(in_state(GameState::Main)),
            );
    }
}

#[derive(Resource, Default)]
pub struct WorkQueue {
    pub build_queue: VecDeque<Entity>,
}

#[derive(Event, Debug)]
pub struct SpawnPawnRequestEvent;

#[derive(Event, Debug)]
pub struct AttackEvent {
    pub attacker: Entity,
    pub target: Entity,
}

#[derive(Event, Debug)]
pub struct RequestWorkOrder {}

#[derive(Resource)]
pub struct EnemyWave {
    pub wave: usize,
    pub enemy_count_multiplier: usize,
    pub enemy_spawn_timer: Timer,
    pub enemies: usize,
}

impl Default for EnemyWave {
    fn default() -> Self {
        Self {
            wave: 0,
            enemy_count_multiplier: 1,
            enemy_spawn_timer: Timer::from_seconds(30.0, TimerMode::Repeating),
            enemies: 0,
        }
    }
}
