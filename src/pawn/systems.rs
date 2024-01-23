use super::components::pawn_status::{Idle, PawnStatus};
use super::components::work_order::{AddWorkOrder, WorkOrder};
use super::{AttackEvent, EnemyWave, PawnDeath, SpawnPawnRequestEvent, WorkQueue};
use crate::factory::components::{Factory, Placed};
use crate::navmesh::components::{PathfindAnswer, PathfindRequest, SpatialGrid};
use crate::navmesh::utils::get_pathing;
use crate::pawn::components::pawn_status::AddStatus;
use crate::pawn::components::work_order::PickupStoneFromFactory;
use crate::placeable::prelude::PlaceableType;
use crate::selectable::Selectable;
use crate::stone::{Stone, StoneKind};
use crate::{
    assets::{CharacterFacing, MalePawns},
    pawn::components::*,
    utils::*,
};
use crate::{CursorPosition, GameResources, GameState, SIZE, TILE_SIZE};
use bevy::ecs::query::ReadOnlyWorldQuery;
use bevy::prelude::*;
use bevy::utils::{HashMap, HashSet};
use leafwing_input_manager::prelude::*;
use rand::prelude::*;
use std::collections::VecDeque;

const INITIAL_PAWN_COUNT: usize = 10;
const MOVE_SPEED: f32 = 60.;
const MAX_RESOURCES: usize = 15;
const RESOURCE_GAIN_RATE: usize = 1;
const PAWN_COST: usize = 100;
const PAWN_ATTACK_STRENGTH: usize = 7;
const ENEMY_TILE_RANGE: usize = 10;
const ENEMY_ATTACK_STRENGTH: usize = 10;
const PAWN_SEARCH_TIMER: f32 = 0.25;
const RESOURCE_MAX_SEARCH_RANGE: usize = 25;

fn spawn_pawn_in_random_location(
    commands: &mut Commands,
    pawn_res: &Res<MalePawns>,
    game_resources: &mut ResMut<GameResources>,
    factory_transform: &GlobalTransform,
    _: &Res<SpatialGrid>,
) {
    let radius = TILE_SIZE * 5.;
    let mut rng = rand::thread_rng();

    let pawn = pawn_res.get_random();

    // spawn pawns in a random circle 1 tile around the factory
    let random_angle: f32 = rng.gen_range(0.0..360.0);
    let x = factory_transform.translation().x + random_angle.cos() * radius;
    let y = factory_transform.translation().y + random_angle.sin() * radius;

    let pawn_entity = commands
        .spawn((
            PawnBundle {
                pawn: Pawn {
                    move_path: VecDeque::new(),
                    move_to: None,
                    health: 100,
                    max_health: 100,
                    animation_timer: Timer::from_seconds(0.125, TimerMode::Repeating),
                    work_timer: Timer::from_seconds(0.25, TimerMode::Once),
                    moving: false,
                    search_timer: Timer::from_seconds(PAWN_SEARCH_TIMER, TimerMode::Repeating),
                    retry_pathfinding_timer: Timer::from_seconds(1., TimerMode::Once),
                },
                character_facing: CharacterFacing::Left,
                name: Name::new("Pawn"),
                sprite_bundle: SpriteSheetBundle {
                    texture_atlas: pawn,
                    transform: Transform::from_translation(Vec3::new(x, y, 1.)),
                    sprite: TextureAtlasSprite {
                        anchor: bevy::sprite::Anchor::BottomLeft,
                        index: CharacterFacing::Left as usize,
                        ..default()
                    },
                    ..Default::default()
                },
                pawn_status: PawnStatus::Idle(Idle),
                resources: CarriedResources(0),
            },
            Selectable,
        ))
        .id();

    commands
        .spawn(HealthBundle {
            health_bar: HealthBar,
            health_bundle: SpriteBundle {
                transform: Transform::from_xyz(16. / 2., 20., 1.),
                sprite: Sprite {
                    custom_size: Some(Vec2::new(16., 2.)),
                    color: Color::NONE,
                    ..default()
                },
                ..default()
            },
        })
        .set_parent(pawn_entity);

    game_resources.pawns += 1;
}

pub fn initial_pawn_spawn(
    mut commands: Commands,
    pawn_res: Res<MalePawns>,
    q_factory: Query<&GlobalTransform, (With<Factory>, With<Placed>)>,
    mut game_resources: ResMut<GameResources>,
    navmesh: Res<SpatialGrid>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let Ok(factory_transform) = q_factory.get_single() else {
        return;
    };

    for _ in 0..INITIAL_PAWN_COUNT {
        spawn_pawn_in_random_location(
            &mut commands,
            &pawn_res,
            &mut game_resources,
            factory_transform,
            &navmesh,
        );
    }

    next_state.set(GameState::Main);
}

pub fn work_idle_pawns(
    mut commands: Commands,
    mut q_pawns: Query<
        (Entity, &Transform, &CarriedResources, &PawnStatus),
        (With<Pawn>, Without<WorkOrder>, Without<Enemy>),
    >,
    q_stones: Query<Entity, With<StoneKind>>,
    q_factory: Query<&GlobalTransform, (With<Factory>, With<Placed>)>,
    q_placeable: Query<(Entity, &PlaceableType)>,
    (navmesh, mut work_queue): (Res<SpatialGrid>, ResMut<super::WorkQueue>),
    mut pathfinding_event_writer: EventWriter<PathfindRequest>,
    game_resources: Res<GameResources>,
) {
    let Ok(factory_transform) = q_factory.get_single() else {
        return;
    };

    for (entity, transform, resources, status) in &mut q_pawns {
        if !variant_eq(status, &PawnStatus::Idle(Idle)) {
            continue;
        }

        let pawn_grid_location = GridPos::from_world_pos_vec(transform.translation.truncate());
        // check build queue first

        if let Some(placeable_entity) = work_queue.build_queue.front() {
            // If this if statement fails, then the placeable doesn't exist.
            // Odd, but move on and continue issuing commands
            if let Ok((_, placeable_type)) = q_placeable.get(*placeable_entity) {
                // check if the pawn has enough resources to build the placeable
                if resources.0 + game_resources.stone > placeable_type.get_missing_resource_count()
                {
                    commands.entity(entity).add_work_order(WorkOrder::BuildItem(
                        work_order::BuildItem {
                            item_entity: work_queue.build_queue.pop_front().unwrap(),
                        },
                    ));

                    continue;
                }
            };
        }

        // check if the pawn is full on resources
        if resources.0 >= MAX_RESOURCES {
            commands
                .entity(entity)
                .add_status(PawnStatus::Pathfinding(pawn_status::Pathfinding))
                .add_work_order(WorkOrder::ReturnToFactory(work_order::ReturnToFactory {}));

            pathfinding_event_writer.send(PathfindRequest {
                start: pawn_grid_location,
                end: GridPos::from_world_pos_vec(factory_transform.translation().truncate()),
                entity,
            });

            continue;
        }

        // search the navmesh for non-walkable tiles, and see if the entities within are in q_stones
        let mut stone_location = None;
        let mut stone_entity = None;
        let mut search_radius: i32 = 5;

        // Find the closest stone to the pawn ensuring that the pawn can reach the stone by pathfinding
        // todo! fix this!
        {
            let _span = info_span!("work_idle_pawns::search_for_stones_loop").entered();

            'base: loop {
                if search_radius > RESOURCE_MAX_SEARCH_RANGE as i32 {
                    break 'base;
                }
                let Some(found_stone) = navmesh
                    .get_entities_in_range(&pawn_grid_location, search_radius)
                    .filter(|ent| q_stones.contains(*ent.entity()))
                    .next()
                else {
                    search_radius += 10;
                    continue;
                };

                let PathfindAnswer { path, .. } = navmesh.get_path(
                    PathfindRequest {
                        entity: Entity::PLACEHOLDER,
                        start: pawn_grid_location,
                        end: *found_stone.position(),
                    },
                    |mut iter| iter.any(|ent| ent.walkable() || q_stones.contains(*ent.entity())),
                );

                let Some(path) = path else {
                    search_radius += 10;
                    continue;
                };

                // walk the path and find the first stone (that will indicate that we can path to the stone as everything)
                for grid_pos in path {
                    let Some(spatial_item) = navmesh
                        .get_entities_at(&grid_pos)
                        .map(|mut iter| iter.find(|ent| q_stones.contains(*ent.entity())))
                        .flatten()
                    else {
                        continue;
                    };

                    stone_location = Some(grid_pos);
                    stone_entity = Some(spatial_item.entity());
                    break 'base;
                }

                search_radius += 10;
            }
        }

        if let Some(stone_location) = stone_location {
            commands
                .entity(entity)
                .add_status(PawnStatus::Pathfinding(pawn_status::Pathfinding))
                .add_work_order(WorkOrder::MineStone(work_order::MineStone {
                    stone_entity: *stone_entity.unwrap(),
                }));
            pathfinding_event_writer.send(PathfindRequest {
                start: pawn_grid_location,
                end: stone_location,
                entity,
            });
        }
    }
}

pub fn listen_for_pathfinding_answers(
    mut commands: Commands,
    mut answer_events: EventReader<PathfindAnswer>,
    mut q_pawns: Query<(&mut Pawn, &mut PawnStatus, Option<&WorkOrder>), With<Pawn>>,
    mut work_queue: ResMut<WorkQueue>,
) {
    for evt in answer_events.read() {
        let Ok((mut pawn, mut status, work_order)) = q_pawns.get_mut(evt.entity) else {
            continue;
        };

        // if we are not pathfinding or repathing, continue to the next entity
        if !variant_eq(&PawnStatus::Pathfinding(pawn_status::Pathfinding), &status)
            && !variant_eq(&PawnStatus::Repathing(pawn_status::Repathing), &status)
        {
            continue;
        }

        if let Some(path) = &evt.path {
            pawn.move_path = path.clone().into();

            *status = PawnStatus::Moving(pawn_status::Moving);
        } else {
            // if we have a work order to build or get stone from factory, add it back to the work queue
            if let Some(work_order) = work_order {
                match work_order {
                    WorkOrder::BuildItem(work_order::BuildItem { item_entity })
                    | WorkOrder::PickupStoneFromFactory(work_order::PickupStoneFromFactory {
                        for_entity: item_entity,
                    }) => {
                        work_queue.build_queue.push_back(*item_entity);
                    }

                    _ => {}
                }
            }

            commands
                .entity(evt.entity)
                .clear_work_order()
                .add_status(PawnStatus::PathfindingError(pawn_status::PathfindingError));
        }
    }
}

pub fn move_pawn(
    mut commands: Commands,
    mut q_pawns: Query<(
        Entity,
        &mut Pawn,
        &mut Transform,
        &PawnStatus,
        Option<&WorkOrder>,
        Option<&Enemy>,
    )>,
    time: Res<Time>,
) {
    for (entity, mut pawn, mut transform, status, order, _) in &mut q_pawns {
        // cleanup pawns that are moving with no work order
        if variant_eq(&PawnStatus::Moving(pawn_status::Moving), status) && order.is_none() {
            commands
                .entity(entity)
                .add_status(PawnStatus::Idle(pawn_status::Idle));
            continue;
        }
        // The, stop all non-moving pawns, ignoring anything that is repathing
        if !variant_eq(&PawnStatus::Moving(pawn_status::Moving), status)
            && !variant_eq(&PawnStatus::Repathing(pawn_status::Repathing), status)
        {
            pawn.moving = false;
            pawn.move_path.clear();
            continue;
        }

        // finally, move the pawns

        let current_grid = transform.translation.world_pos_to_tile();
        if pawn.move_to.is_none() {
            pawn.move_to = pawn.move_path.pop_front();
        }
        let Some(path) = pawn.move_to else {
            pawn.moving = false;
            continue;
        };

        let direction = (path.to_vec2() - current_grid).normalize_or_zero();
        transform.translation += direction.extend(0.) * MOVE_SPEED * time.delta_seconds();
        pawn.moving = true;
        if (path.to_vec2() - current_grid).length() < 0.2 {
            pawn.move_to = pawn.move_path.pop_front();
        }
    }

    // for (mut transform, mut pawn, mut facing) in &mut q_pawn.p1() {
    //     let current_grid = transform.translation.world_pos_to_tile();

    //     if pawn.move_to.is_none() {
    //         pawn.move_to = pawn.move_path.pop_front();
    //     }

    //     let Some(path) = pawn.move_to else {
    //         pawn.moving = false;
    //         continue;
    //     };

    //     let direction = (path - current_grid).normalize_or_zero();

    //     transform.translation += direction.extend(0.) * MOVE_SPEED * time.delta_seconds();
    //     pawn.moving = true;
    //     // update facing direction depending on direction (right, left, forward, backwards)

    //     if direction.length() > 0. {
    //         if direction.x.abs() > direction.y.abs() {
    //             if direction.x > 0. {
    //                 *facing = CharacterFacing::Right;
    //             } else {
    //                 *facing = CharacterFacing::Left;
    //             }
    //         } else if direction.y > 0. {
    //             *facing = CharacterFacing::Backward;
    //         } else {
    //             *facing = CharacterFacing::Forward;
    //         }
    //     }
    //     if (path - current_grid).length() < 0.2 {
    //         pawn.move_to = pawn.move_path.pop_front();
    //     }
    // }

    // // cleanup pawns that are moving but have no work order
    // for (entity, pawn) in &q_pawn.p2() {
    //     if pawn.move_path.is_empty() && !pawn.moving {
    //         commands.entity(entity).add_status(pawn_status::Idle);
    //     }
    // }

    // // stop the pawn in place if it's attacking
    // for mut pawn in &mut q_pawn.p0() {
    //     pawn.moving = false;
    //     pawn.move_path.clear();
    // }
}

// TODO! Fix this function because it doesn't work properly. But it's not a priority right now.
pub fn update_pawn_animation(
    mut q_pawn: Query<(&mut TextureAtlasSprite, &Pawn, &CharacterFacing), With<Pawn>>,
) {
    for (mut sprite, pawn, facing) in &mut q_pawn {
        if !pawn.moving {
            sprite.index = *facing as usize;
            continue;
        }

        if pawn.animation_timer.finished() {
            // // step forward 4 cells in the texture atlas to reach the next step in the animation
            // sprite.index += 4;

            let final_animation_frame = 15 - *facing as usize;

            if sprite.index + 4 > final_animation_frame {
                sprite.index = *facing as usize;
            } else {
                sprite.index += 4;
            }
        }
    }
}

pub fn update_health_ui(
    q_pawns: Query<&Pawn>,
    mut q_health_bar: Query<(&Parent, &mut Sprite), With<HealthBar>>,
) {
    let green_health_threshold: usize = 75;
    let yellow_health_threshold: usize = 50;
    let red_health_threshold: usize = 25;

    for (parent, mut sprite) in &mut q_health_bar {
        let pawn_entity = parent.get();

        let Ok(pawn) = q_pawns.get(pawn_entity) else {
            continue;
        };

        sprite.custom_size = Some(Vec2::new(
            pawn.health as f32 / pawn.max_health as f32 * 16.,
            2.,
        ));

        if pawn.health == pawn.max_health {
            sprite.color = Color::NONE;
        } else if pawn.health > green_health_threshold {
            sprite.color = Color::GREEN;
        } else if pawn.health > yellow_health_threshold {
            sprite.color = Color::YELLOW;
        } else if pawn.health > red_health_threshold {
            sprite.color = Color::RED;
        } else {
            sprite.color = Color::rgb(0.5, 0., 0.);
        }
    }
}

pub fn mine_stone(
    mut commands: Commands,
    mut q_pawns_new: Query<
        (
            Entity,
            &mut Pawn,
            &mut CarriedResources,
            Option<&WorkOrder>,
            &mut PawnStatus,
        ),
        Without<Enemy>,
    >,
    mut q_stones: Query<(Entity, &mut Stone, &Transform), With<StoneKind>>,
    mut navmesh: ResMut<SpatialGrid>,
) {
    let mut destroyed_stones = HashSet::<Entity>::default();

    for (pawn_entity, mut pawn, mut carried_resources, work_order, mut pawn_status) in
        &mut q_pawns_new
    {
        // We don't have a mine stone work order, skip this entity.
        let Some(WorkOrder::MineStone(work_order::MineStone { stone_entity })) = work_order else {
            continue;
        };

        // We were moving, but we have reached out destination. Start mining and continue logic.
        if !pawn.moving && variant_eq(&PawnStatus::Moving(pawn_status::Moving), &pawn_status) {
            *pawn_status = PawnStatus::Mining(pawn_status::Mining);
        }

        // We are not mining yet, continue to next entity
        if !variant_eq(&PawnStatus::Mining(pawn_status::Mining), &pawn_status) {
            continue;
        }

        // We were mining but our resources are full. Deposit at the factory and move to next entity.
        if carried_resources.0 >= MAX_RESOURCES {
            commands
                .entity(pawn_entity)
                .add_work_order(WorkOrder::ReturnToFactory(work_order::ReturnToFactory {}))
                .add_status(PawnStatus::Idle(pawn_status::Idle));

            continue;
        }

        // Our mine work cooldown has not completed. Skip to the next entity.
        if !pawn.work_timer.finished() {
            continue;
        }

        pawn.work_timer.reset();

        // If the stone does not exist, then it's been destoyed. Set idle and skip to the next entity.
        let Ok((stone_entity, mut stone, stone_transform)) = q_stones.get_mut(*stone_entity) else {
            commands
                .entity(pawn_entity)
                .clear_work_order()
                .add_status(PawnStatus::Idle(pawn_status::Idle));
            continue;
        };

        if stone.remaining_resources > 0 {
            stone.remaining_resources =
                stone.remaining_resources.saturating_sub(RESOURCE_GAIN_RATE);
            carried_resources.0 = carried_resources.0.saturating_add(RESOURCE_GAIN_RATE);
        } else {
            // we're about to despawn an entity, get it's grid transform and remove it from the navmesh before we despawn it

            // Ensure the stone has not already been despawned
            if destroyed_stones.contains(&stone_entity) {
                continue;
            }

            let stone_grid = stone_transform.translation.world_pos_to_tile();

            navmesh.remove(&stone_entity, &GridPos::from_tile_pos_vec(stone_grid));

            commands.entity(stone_entity).despawn_recursive();
            commands
                .entity(pawn_entity)
                .clear_work_order()
                .add_status(PawnStatus::Idle(pawn_status::Idle));
            destroyed_stones.insert(stone_entity);
        }
    }
}

pub fn pickup_stone_from_factory(
    mut commands: Commands,
    mut q_pawns: Query<
        (
            Entity,
            &mut CarriedResources,
            &WorkOrder,
            &mut PawnStatus,
            &Transform,
        ),
        Without<Enemy>,
    >,
    q_placeable: Query<&PlaceableType>,
    q_factory: Query<&Transform, (With<Factory>, With<Placed>)>,
    mut nav_request: EventWriter<PathfindRequest>,
    mut game_resources: ResMut<GameResources>,
    mut work_queue: ResMut<WorkQueue>,
) {
    for (pawn_entity, mut carried_resources, order, mut status, pawn_transform) in &mut q_pawns {
        // if we aren't picking up stone, we don't need to do anything. Continue to the next entity.
        let WorkOrder::PickupStoneFromFactory(work_order::PickupStoneFromFactory { for_entity }) =
            order
        else {
            continue;
        };

        let Ok(placeable) = q_placeable.get(*for_entity) else {
            // something went wrong here. Clear work order, set status to idle, and continue to the next entity.
            commands.entity(pawn_entity).clear_work_order();
            *status = PawnStatus::Idle(pawn_status::Idle);
            continue;
        };

        let Ok(factory_transform) = q_factory.get_single() else {
            return;
        };

        // we are idle, we need to pathfind to the factory and move to the next entity
        if variant_eq(&PawnStatus::Idle(pawn_status::Idle), &status) {
            *status = PawnStatus::Pathfinding(pawn_status::Pathfinding);
            nav_request.send(PathfindRequest {
                start: GridPos::from_world_pos_vec(pawn_transform.translation.truncate()),
                end: GridPos::from_world_pos_vec(factory_transform.translation.truncate()),
                entity: pawn_entity,
            });
            continue;
        }

        let pawn_location_tile = pawn_transform.translation.world_pos_to_tile();
        let factory_location_tile = factory_transform.translation.world_pos_to_tile();

        let distance = (pawn_location_tile - factory_location_tile).length();

        // we are still moving to the factory
        if distance > 2. {
            continue;
        }

        *status = PawnStatus::Idle(pawn_status::Idle);
        commands
            .entity(pawn_entity)
            .add_work_order(WorkOrder::BuildItem(work_order::BuildItem {
                item_entity: *for_entity,
            }));

        let required_resources = placeable.get_missing_resource_count();

        let mut to_add_to_pawn = std::cmp::min(MAX_RESOURCES, required_resources);
        if to_add_to_pawn > game_resources.stone {
            to_add_to_pawn = game_resources.stone;
        }

        game_resources.stone -= to_add_to_pawn;
        carried_resources.0 += to_add_to_pawn;

        // edge case. If carried resources is 0, clear work order, set idle, add to work queue, and continue to next entity
        if carried_resources.0 == 0 {
            work_queue.build_queue.push_back(*for_entity);
            commands.entity(pawn_entity).clear_work_order();
            *status = PawnStatus::Idle(pawn_status::Idle);
        }
    }
}

pub fn build_placeable(
    mut commands: Commands,
    mut q_pawns: Query<
        (
            Entity,
            &Transform,
            &mut Pawn,
            &mut CarriedResources,
            &mut PawnStatus,
            &WorkOrder,
        ),
        Without<Enemy>,
    >,
    mut q_placeable: Query<
        (Entity, &mut PlaceableType, &Transform),
        Without<crate::placeable::components::Built>,
    >,
    mut nav_request: EventWriter<PathfindRequest>,
) {
    for (entity, transform, _, mut carried_resources, mut status, order) in &mut q_pawns {
        // If we don't have a build work order, skip this entity
        let WorkOrder::BuildItem(work_order::BuildItem { item_entity }) = order else {
            continue;
        };

        let Ok((_, mut placeable_type, placeable_transform)) = q_placeable.get_mut(*item_entity)
        else {
            // Not sure what happened here, but clear the status and work order and keep working.
            // The placeable does not exist in the query so it's not workable
            *status = PawnStatus::Idle(pawn_status::Idle);
            commands.entity(entity).clear_work_order();
            continue;
        };

        // we are idle, we need to either get resources OR pathfind to the placeable
        if variant_eq(&PawnStatus::Idle(pawn_status::Idle), &status) {
            // we do not have enough resources OR we are not full on resources. We need to get more resources
            // from the factory
            if carried_resources.0 < placeable_type.get_missing_resource_count()
                && carried_resources.0 < MAX_RESOURCES
            {
                *status = PawnStatus::Idle(pawn_status::Idle);
                commands
                    .entity(entity)
                    .add_work_order(WorkOrder::PickupStoneFromFactory(
                        work_order::PickupStoneFromFactory {
                            for_entity: *item_entity,
                        },
                    ));
                continue;
            }
            // we have the required resources. Pathfind to the placeable
            let placeable_grid_pos = placeable_transform.translation.world_pos_to_tile();
            let pawn_grid_pos = transform.translation.world_pos_to_tile();
            *status = PawnStatus::Pathfinding(pawn_status::Pathfinding);
            nav_request.send(PathfindRequest {
                start: GridPos::from_tile_pos_vec(pawn_grid_pos),
                end: GridPos::from_tile_pos_vec(placeable_grid_pos),
                entity,
            });
        }

        // check to see if we are close enough to start work.
        let distance_to_placeable = (transform.translation.world_pos_to_tile()
            - placeable_transform.translation.world_pos_to_tile())
        .length();
        if distance_to_placeable < 1.5
            && variant_eq(&PawnStatus::Moving(pawn_status::Moving), &status)
        {
            // we are close enough to start work. Set the status to building and continue to the next entity
            *status = PawnStatus::Building(pawn_status::Building);
        }
        // if we are not in the building state, continue to the next entity
        if !variant_eq(&PawnStatus::Building(pawn_status::Building), &status) {
            continue;
        }

        // we are in the building state. Add pawn's resources to the placeable, subtract the resources from the pawn,
        // and depending on if the build is finished set idle or get more resources from the factory.
        let to_set = (placeable_type.get_current_resources() + carried_resources.0)
            .clamp(0, placeable_type.get_max_resources());

        let diff = to_set.saturating_sub(placeable_type.get_current_resources());

        placeable_type.set_current_resources(to_set);

        carried_resources.0 = diff;

        // we are either going to factory or getting a new job
        *status = PawnStatus::Idle(pawn_status::Idle);

        // if we have finished building, set the status to idle and clear the work order
        if placeable_type.get_current_resources() == placeable_type.get_max_resources() {
            commands.entity(entity).clear_work_order();
            continue;
        }
        // otherwise we need to get more resources from the factory
        commands
            .entity(entity)
            .add_work_order(WorkOrder::PickupStoneFromFactory(
                work_order::PickupStoneFromFactory {
                    for_entity: *item_entity,
                },
            ));
    }
}

pub fn return_to_factory(
    mut commands: Commands,
    mut q_pawns: Query<
        (
            Entity,
            &Transform,
            &Pawn,
            &mut CarriedResources,
            &mut PawnStatus,
            &WorkOrder,
        ),
        Without<Enemy>,
    >,
    q_factory: Query<&Transform, (With<Factory>, With<Placed>)>,
    mut resources: ResMut<GameResources>,
    mut pathfinding_event_writer: EventWriter<PathfindRequest>,
) {
    let Ok(factory_transform) = q_factory.get_single() else {
        return;
    };

    let factory_grid = factory_transform.translation.world_pos_to_tile();

    for (pawn_entity, transform, pawn, mut carried_resources, mut pawn_status, work_order) in
        &mut q_pawns
    {
        // We are not returning to factory. Skip this entity.
        if !variant_eq(
            &WorkOrder::ReturnToFactory(work_order::ReturnToFactory {}),
            &work_order,
        ) {
            continue;
        }

        // We are idle, we need to pathfind to the factory and move to the next entity
        if variant_eq(&PawnStatus::Idle(pawn_status::Idle), &pawn_status) {
            *pawn_status = PawnStatus::Pathfinding(pawn_status::Pathfinding);
            pathfinding_event_writer.send(PathfindRequest {
                start: GridPos::from_world_pos_vec(transform.translation.truncate()),
                end: GridPos::from_tile_pos_vec(factory_grid),
                entity: pawn_entity,
            });
            continue;
        }

        // We are not longer moving. We have made it to the factory and need to deposit the resources.
        // After which we will set the status to Idle, clear the work order, and move to the next entity.
        if !pawn.moving && variant_eq(&PawnStatus::Moving(pawn_status::Moving), &pawn_status) {
            *pawn_status = PawnStatus::Idle(pawn_status::Idle);
            commands.entity(pawn_entity).clear_work_order();
            resources.stone += carried_resources.0;
            carried_resources.0 = 0;
            continue;
        }
    }
}

pub fn listen_for_spawn_pawn_event(
    mut commands: Commands,
    pawn_res: Res<MalePawns>,
    q_factory: Query<&GlobalTransform, (With<Factory>, With<Placed>)>,
    mut game_resources: ResMut<GameResources>,
    mut spawn_pawn_event_reader: EventReader<SpawnPawnRequestEvent>,
    navmesh: Res<SpatialGrid>,
) {
    let Ok(factory_transform) = q_factory.get_single() else {
        return;
    };

    for _ in spawn_pawn_event_reader.read() {
        if game_resources.stone >= PAWN_COST {
            game_resources.stone -= 100;
        } else {
            continue;
        }
        spawn_pawn_in_random_location(
            &mut commands,
            &pawn_res,
            &mut game_resources,
            factory_transform,
            &navmesh,
        );
    }
}

pub fn tick_timers(mut q_pawns: Query<&mut Pawn>, time: Res<Time>) {
    for mut pawn in &mut q_pawns {
        pawn.search_timer.tick(time.delta());
        pawn.work_timer.tick(time.delta());
        pawn.animation_timer.tick(time.delta());
        pawn.retry_pathfinding_timer.tick(time.delta());
    }
}

pub fn retry_pathfinding(
    mut commands: Commands,
    mut q_pawns: Query<(
        Entity,
        &mut Pawn,
        &Transform,
        &mut PawnStatus,
        Option<&WorkOrder>,
    )>,
    q_factory: Query<&GlobalTransform, (With<Factory>, With<Placed>)>,
    mut pathfinding_event_writer: EventWriter<PathfindRequest>,
    mut work_queue: ResMut<WorkQueue>,
) {
    let mut pathfinding_requests = Vec::new();
    let Ok(factory_transform) = q_factory.get_single() else {
        return;
    };
    for (entity, mut pawn, pawn_transform, mut pawn_status, order) in &mut q_pawns {
        // The pawn is not in a pathfinding error state, skip this entity
        if !variant_eq(
            &PawnStatus::PathfindingError(pawn_status::PathfindingError),
            &pawn_status,
        ) {
            continue;
        }

        // Buffer the pathfinding requests so we don't overload the CPU with pathfinding requests if we cannot find a path
        if !pawn.retry_pathfinding_timer.finished() {
            continue;
        }
        pawn.retry_pathfinding_timer.reset();

        *pawn_status = PawnStatus::Idle(pawn_status::Idle);

        if let Some(order) = order {
            match order {
                WorkOrder::BuildItem(work_order::BuildItem { item_entity })
                | WorkOrder::PickupStoneFromFactory(PickupStoneFromFactory {
                    for_entity: item_entity,
                }) => {
                    // requeue the work order so it can be picked up by another pawn.
                    work_queue.build_queue.push_back(*item_entity);
                }
                _ => {}
            }
        }

        commands.entity(entity).clear_work_order();

        pathfinding_requests.push(PathfindRequest {
            start: GridPos::from_world_pos_vec(pawn_transform.translation.truncate()),
            end: GridPos::from_world_pos_vec(factory_transform.translation().truncate()),
            entity,
        });
    }

    pathfinding_event_writer.send_batch(pathfinding_requests);
}

pub fn repath_if_navmesh_changes(
    mut q_pawns: Query<(Entity, &Pawn, &mut PawnStatus)>,
    navmesh: Res<SpatialGrid>,
    mut nav_request: EventWriter<PathfindRequest>,
) {
    for (entity, pawn, mut status) in &mut q_pawns {
        if pawn.move_path.is_empty() {
            continue;
        }

        // ignore unless the status is moving or repathing
        if !variant_eq(&PawnStatus::Moving(pawn_status::Moving), &status)
            && !variant_eq(&PawnStatus::Repathing(pawn_status::Repathing), &status)
        {
            continue;
        }

        for grid_pos in &pawn.move_path {
            if !navmesh.is_walkable(&grid_pos) {
                // we've already verified that the path is not empty, so we can unwrap here
                let target_location = pawn.move_path.back().unwrap().clone();
                let current_location = pawn.move_path.front().unwrap().clone();

                *status = PawnStatus::Repathing(pawn_status::Repathing);

                // request a path from the current location to the target location
                nav_request.send(PathfindRequest {
                    start: current_location,
                    end: target_location,
                    entity,
                });

                break;
            }
        }
    }
}

pub fn search_for_attack_target_pawn(
    mut commands: Commands,
    q_pawns: Query<(Entity, &Pawn, &Transform, Option<&WorkOrder>), Without<Enemy>>,
    q_enemies: Query<(Entity, &Pawn, &Transform, Option<&WorkOrder>), With<Enemy>>,
    mut pathfinding_event_writer: EventWriter<PathfindRequest>,
    mut work_queue: ResMut<WorkQueue>,
    navmesh: Res<SpatialGrid>,
) {
    #[derive(Debug)]
    struct PawnAttacking {
        pawn_entity: Entity,
        pawn_location: Vec2,
        target_entity: Entity,
        target_location: Vec2,
    }
    fn find_pawns_to_attack(
        search_query: &Query<
            (Entity, &Pawn, &Transform, Option<&WorkOrder>),
            impl ReadOnlyWorldQuery,
        >,
        to_attack_query: &Query<
            (Entity, &Pawn, &Transform, Option<&WorkOrder>),
            impl ReadOnlyWorldQuery,
        >,
        attack_map: &mut HashMap<Entity, Vec<PawnAttacking>>,
        navmesh: &Res<SpatialGrid>,
    ) {
        for (pawn_entity, pawn, transform, work_order) in search_query {
            // we already have an attack work order, skip this pawn
            if let Some(WorkOrder::AttackPawn(work_order::AttackPawn { pawn_entity: _ })) =
                work_order
            {
                continue;
            }

            if !pawn.search_timer.finished() {
                continue;
            }
            let pawn_position = transform.world_pos_to_tile();
            let mut results = to_attack_query
                .iter()
                .filter(|&(_, _, enemy_pos, _)| {
                    let enemy_position = enemy_pos.world_pos_to_tile();
                    (enemy_position - pawn_position).length() <= ENEMY_TILE_RANGE as f32 && {
                        let path = get_pathing(
                            PathfindRequest {
                                start: GridPos::from_tile_pos_vec(pawn_position),
                                end: GridPos::from_tile_pos_vec(enemy_position),
                                entity: pawn_entity,
                            },
                            &navmesh,
                        );
                        path.is_some() && path.unwrap().len() <= ENEMY_TILE_RANGE
                    }
                })
                .collect::<Vec<_>>();
            results.sort_by(|&(_, _, a, _), &(_, _, b, _)| {
                let a_distance = (a.world_pos_to_tile() - pawn_position).length();
                let b_distance = (b.world_pos_to_tile() - pawn_position).length();
                a_distance.partial_cmp(&b_distance).unwrap()
            });
            let Some((enemy_entity, _, enemy_transform, _)) = results.into_iter().next() else {
                continue;
            };

            let pawn_attacking = PawnAttacking {
                pawn_entity,
                target_entity: enemy_entity,
                pawn_location: pawn_position,
                target_location: enemy_transform.world_pos_to_tile(),
            };

            if attack_map.contains_key(&enemy_entity) {
                attack_map
                    .get_mut(&enemy_entity)
                    .unwrap()
                    .push(pawn_attacking);
            } else {
                attack_map.insert(enemy_entity, vec![pawn_attacking]);
            }
        }
    }

    // A map which contains the target of the attack, and the details about the attack
    let mut attack_map = HashMap::<Entity, Vec<PawnAttacking>>::new();

    find_pawns_to_attack(&q_pawns, &q_enemies, &mut attack_map, &navmesh);
    find_pawns_to_attack(&q_enemies, &q_pawns, &mut attack_map, &navmesh);

    let nav_requests = attack_map
        .values()
        .into_iter()
        .flat_map(|v| {
            v.into_iter().map(
                |&PawnAttacking {
                     pawn_entity,
                     pawn_location,
                     target_location,
                     target_entity,
                 }| {
                    (
                        PathfindRequest {
                            start: GridPos::from_tile_pos_vec(pawn_location),
                            end: GridPos::from_tile_pos_vec(target_location),
                            entity: pawn_entity,
                        },
                        target_entity,
                    )
                },
            )
        })
        .collect::<Vec<_>>();

    for &(PathfindRequest { entity, .. }, target_entity) in &nav_requests {
        if let Ok((_, _, _, work_order)) = q_pawns.get(entity) {
            // handle other work orders here
            if let Some(order) = work_order {
                match order {
                    WorkOrder::BuildItem(work_order::BuildItem { item_entity })
                    | WorkOrder::PickupStoneFromFactory(PickupStoneFromFactory {
                        for_entity: item_entity,
                    }) => {
                        // requeue the work order so it can be picked up by another pawn.
                        work_queue.build_queue.push_back(*item_entity);
                        commands.entity(entity).clear_work_order();
                    }
                    _ => {}
                }
            }
        }

        commands
            .entity(entity)
            .add_status(PawnStatus::Pathfinding(pawn_status::Pathfinding))
            .add_work_order(WorkOrder::AttackPawn(work_order::AttackPawn {
                pawn_entity: target_entity,
            }));
    }

    pathfinding_event_writer.send_batch(nav_requests.into_iter().map(|(r, _)| r));
}

pub fn spawn_enemy_pawns(
    mut commands: Commands,
    mut enemy_wave: ResMut<EnemyWave>,
    pawn_res: Res<MalePawns>,
    time: Res<Time>,
    navmesh: Res<SpatialGrid>,
    input: Query<&ActionState<crate::Input>>,
    mouse_position: Res<CursorPosition>,
) {
    let mut spawn_enemy = move |spawn_location: Vec2| {
        let pawn_entity = commands
            .spawn(PawnBundle {
                pawn: Pawn {
                    move_path: VecDeque::new(),
                    move_to: None,
                    health: 100,
                    max_health: 100,
                    search_timer: Timer::from_seconds(PAWN_SEARCH_TIMER, TimerMode::Repeating),
                    animation_timer: Timer::from_seconds(0.125, TimerMode::Repeating),
                    work_timer: Timer::from_seconds(0.25, TimerMode::Once),
                    retry_pathfinding_timer: Timer::from_seconds(1., TimerMode::Once),
                    moving: false,
                },
                character_facing: CharacterFacing::Left,
                name: Name::new("Enemy"),
                sprite_bundle: SpriteSheetBundle {
                    texture_atlas: pawn_res.get_random(),
                    transform: Transform::from_translation(Vec3::new(
                        spawn_location.x,
                        spawn_location.y,
                        1.,
                    )),
                    sprite: TextureAtlasSprite {
                        anchor: bevy::sprite::Anchor::BottomLeft,
                        index: CharacterFacing::Left as usize,
                        color: Color::RED,
                        ..default()
                    },
                    ..Default::default()
                },
                pawn_status: PawnStatus::Idle(pawn_status::Idle),
                resources: CarriedResources(0),
            })
            .insert(Enemy)
            .id();

        commands
            .spawn(HealthBundle {
                health_bar: HealthBar,
                health_bundle: SpriteBundle {
                    transform: Transform::from_xyz(16. / 2., 20., 1.),
                    sprite: Sprite {
                        custom_size: Some(Vec2::new(16., 2.)),
                        color: Color::NONE,
                        ..default()
                    },
                    ..default()
                },
            })
            .set_parent(pawn_entity);
    };

    let Ok(input) = input.get_single() else {
        return;
    };

    if input.just_pressed(crate::Input::DebugSpawnPawn) && mouse_position.0.is_some() {
        #[cfg(debug_assertions)]
        spawn_enemy(mouse_position.0.unwrap().tile_pos_to_world());
    }

    enemy_wave.enemy_spawn_timer.tick(time.delta());

    if !enemy_wave.enemy_spawn_timer.just_finished() {
        return;
    }
    enemy_wave.wave += 1;

    for _ in 0..enemy_wave.wave * enemy_wave.enemy_count_multiplier {
        // get a random boolean true or false
        let mut rng = rand::thread_rng();
        let spawn_x = rng.gen_bool(0.5);

        let spawn_location: Vec2;

        loop {
            let temp_location: (usize, usize) = if spawn_x {
                // randomly choose between 0 or SIZE (left or right)
                let x: usize = if rng.gen_bool(0.5) { SIZE } else { 0 };
                let y = rng.gen_range(0..SIZE - 1);

                (x, y)
            } else {
                let x = rng.gen_range(0..SIZE - 1);
                let y: usize = if rng.gen_bool(0.5) { SIZE } else { 0 };
                (x, y)
            };

            // check navtile to ensure it's walkable

            if navmesh.is_walkable(&GridPos::new(
                temp_location.0 as i32,
                temp_location.1 as i32,
            )) {
                spawn_location = Vec2::new(temp_location.0 as f32, temp_location.1 as f32);
                break;
            }
        }

        // convert spawn_location to world coordinates
        let spawn_location = spawn_location.tile_pos_to_world();
        spawn_enemy(spawn_location);
        enemy_wave.enemies += 1;
        // spawn enemy pawn
    }
}

pub fn enemy_search_for_factory(
    mut commands: Commands,
    mut q_enemy_pawns: Query<(Entity, &Transform, &mut PawnStatus), With<Enemy>>,
    q_factory: Query<&GlobalTransform, (With<Factory>, With<Placed>)>,
    mut nav_request: EventWriter<PathfindRequest>,
) {
    let Ok(factory) = q_factory.get_single() else {
        return;
    };

    for (entity, transform, mut pawn_status) in &mut q_enemy_pawns {
        // we are not idle, skip this entity
        if !variant_eq(&PawnStatus::Idle(pawn_status::Idle), &pawn_status) {
            continue;
        }

        nav_request.send(PathfindRequest {
            start: GridPos::from_world_pos_vec(transform.translation.truncate()),
            end: GridPos::from_world_pos_vec(factory.translation().truncate()),
            entity,
        });

        *pawn_status = PawnStatus::Pathfinding(pawn_status::Pathfinding);

        commands
            .entity(entity)
            .add_work_order(WorkOrder::AttackFactory(work_order::AttackFactory {}));
    }
}

pub fn attack_pawn(
    mut commands: Commands,
    mut q_pawns: Query<(
        Entity,
        Option<&WorkOrder>,
        &mut Pawn,
        &mut PawnStatus,
        &Transform,
        Option<&Enemy>,
        &CarriedResources,
    )>,
    q_all_pawns: Query<(Entity, &Transform), With<Pawn>>,
    mut game_resources: ResMut<GameResources>,
    mut enemy_wave: ResMut<EnemyWave>,
    mut pathfinding_event_writer: EventWriter<PathfindRequest>,
    mut attack_event_writer: EventWriter<AttackEvent>,
    mut work_queue: ResMut<WorkQueue>,
    mut pawn_death_writer: EventWriter<PawnDeath>,
) {
    #[derive(Debug)]
    struct AttackMetadata {
        entity: Entity,
        attacking_entity: Entity,
        attack_for: usize,
        entity_is_enemy: bool,
    }

    let mut queued_attacks: Vec<AttackMetadata> = Vec::new();
    let mut destroyed_pawns = HashSet::<Entity>::default();

    for (entity, order, mut pawn, mut status, transform, enemy, _) in &mut q_pawns {
        // we are not set to attack a pawn, skip this entity
        let Some(WorkOrder::AttackPawn(work_order::AttackPawn {
            pawn_entity: attacking_entity,
        })) = order
        else {
            continue;
        };
        let Ok((_, attacking_transform)) = q_all_pawns.get(*attacking_entity) else {
            // oh no, the pawn is missing. Set status to idle and clear work order
            *status = PawnStatus::Idle(pawn_status::Idle);
            commands.entity(entity).clear_work_order();
            continue;
        };

        let distance_to_target_grid = (attacking_transform.translation.world_pos_to_tile()
            - transform.translation.world_pos_to_tile())
        .length();

        if distance_to_target_grid > 2. {
            // we are not close enough to attack, continue OR update pathfinding
            // If our search time is finished, we need to update our pathfinding to the pawn we're attacking
            if pawn.search_timer.finished() {
                *status = PawnStatus::Repathing(pawn_status::Repathing);
                pathfinding_event_writer.send(PathfindRequest {
                    start: GridPos::from_world_pos_vec(transform.translation.truncate()),
                    end: GridPos::from_world_pos_vec(attacking_transform.translation.truncate()),
                    entity,
                });
            }
            continue;
        }

        // We are close enough to attack, but we are not attacking yet. Set status to attacking and continue
        *status = PawnStatus::Attacking(pawn_status::Attacking);

        // We are still winding up our attack, skip this entity
        if !pawn.work_timer.finished() {
            continue;
        }

        // We're about to attack, reset our work timer
        pawn.work_timer.reset();

        // queue an attack
        queued_attacks.push(AttackMetadata {
            entity,
            attacking_entity: *attacking_entity,
            attack_for: if enemy.is_some() {
                ENEMY_ATTACK_STRENGTH
            } else {
                PAWN_ATTACK_STRENGTH
            },
            entity_is_enemy: enemy.is_some(),
        });

        // Check the attacking_entity to let it know that we are attacking it, and if it's not attacking, make it attack us back
        // by firing an AttackEvent, which will be listened to and responded to accordingly
        attack_event_writer.send(AttackEvent {
            attacker: entity,
            target: *attacking_entity,
        });
    }

    if queued_attacks.is_empty() {
        return;
    }

    for AttackMetadata {
        attack_for,
        attacking_entity,
        entity,
        entity_is_enemy,
        ..
    } in queued_attacks
    {
        let Ok((_, order, mut pawn, _, tx, _, carried_resources)) =
            q_pawns.get_mut(attacking_entity)
        else {
            // oops, what happened here? We should have a pawn but we don't. Reset the attacker to idle.
            commands
                .entity(attacking_entity)
                .clear_work_order()
                .add_status(PawnStatus::Idle(pawn_status::Idle));
            continue;
        };

        // the pawn we're attacking has already been destroyed. Reset the attacker to idle.
        if destroyed_pawns.contains(&attacking_entity) {
            commands
                .entity(entity)
                .clear_work_order()
                .add_status(PawnStatus::Idle(pawn_status::Idle));
            continue;
        }

        if pawn.health <= attack_for {
            // whelp, this pawn is about to die. Despawn it and update the game resources
            // making sure to add it to the destroyed_pawns set so we don't try to attack it again
            commands.entity(attacking_entity).despawn_recursive();
            destroyed_pawns.insert(attacking_entity);
            if entity_is_enemy {
                game_resources.pawns = game_resources.pawns.saturating_sub(1);

                // check the work order to see if it needs to be requeued
                if let Some(order) = order {
                    match order {
                        WorkOrder::BuildItem(work_order::BuildItem { item_entity })
                        | WorkOrder::PickupStoneFromFactory(PickupStoneFromFactory {
                            for_entity: item_entity,
                        }) => {
                            // requeue the work order so it can be picked up by another pawn.
                            work_queue.build_queue.push_back(*item_entity);
                        }
                        _ => {}
                    }
                }

                pawn_death_writer.send(PawnDeath {
                    pawn: attacking_entity,
                    killer: entity,
                    carried_resources: carried_resources.0,
                    work_order: order.cloned(),
                    death_location_tile: tx.translation.world_pos_to_tile(),
                });
            } else {
                enemy_wave.enemies = enemy_wave.enemies.saturating_sub(1);
            }
        }

        pawn.health = pawn.health.saturating_sub(attack_for);
    }
}
