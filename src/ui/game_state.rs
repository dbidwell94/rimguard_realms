use super::styles::*;
use crate::placeable::components as placeable_components;
use crate::TILE_SIZE;
use crate::{pawn::SpawnPawnRequestEvent, GameResources, GameState, WorldInteraction};
use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy_ui_dsl::*;

pub struct GameStateUIPlugin;

impl Plugin for GameStateUIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Main), game_state_ui)
            .add_systems(OnExit(GameState::Main), destroy_game_state_ui)
            .add_systems(
                Update,
                ((update_resource_counter, update_pawn_counter).run_if(
                    in_state(GameState::Main).and_then(resource_changed::<GameResources>()),
                ),),
            )
            .add_systems(
                Update,
                update_enemy_counter.run_if(
                    in_state(GameState::Main)
                        .and_then(resource_changed::<crate::pawn::EnemyWave>()),
                ),
            )
            .add_systems(
                Update,
                (
                    listen_for_spawn_pawn,
                    listen_for_wall_spawn,
                    listen_for_turret_spawn,
                )
                    .run_if(in_state(GameState::Main)),
            );
    }
}

#[derive(Component)]
struct GameResourceCounter;
#[derive(Component)]
struct PawnResourceCounter;
#[derive(Component)]
struct EnemyResourceCounter;

#[derive(Component)]
struct GameStateUI;
#[derive(Component)]
struct PawnSpawnButton;

#[derive(Component)]
struct WallSpawnButton;

#[derive(Component)]
struct TurretSpawnButton;

fn game_state_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    let mut resource_entity = None;
    let mut pawn_entity = None;
    let mut enemy_entity = None;

    let mut pawn_spawn_button = None;
    let mut wall_spawn_button = None;
    let mut turret_spawn_button = None;

    let root_entity = root(
        root_full_screen(Some(JustifyContent::Center), Some(AlignItems::Center)),
        &asset_server,
        &mut commands,
        |p| {
            node(top_right_anchor, p, |p| {
                node((), p, |p| {
                    text("Resources: ", c_pixel_text, text_style(Some(28.)), p);
                    text("0", c_pixel_text, text_style(Some(28.)), p).set(&mut resource_entity);
                });
                node((), p, |p| {
                    text("Pawns: ", c_pixel_text, text_style(Some(28.)), p);
                    text("0", c_pixel_text, text_style(Some(28.)), p).set(&mut pawn_entity);
                });
                node((), p, |p| {
                    text("Enemies: ", c_pixel_text, text_style(Some(28.)), p);
                    text("0", c_pixel_text, text_style(Some(28.)), p).set(&mut enemy_entity);
                });
            });
            node(bottom_center_anchor, p, |p| {
                // pawn spawn button
                button(spawn_menu_button(Some("objects/pawns/pawn.png")), p, |_| {})
                    .set(&mut pawn_spawn_button);
                // wall spawn button
                button(
                    spawn_menu_button(Some("objects/walls/wallStone.png")),
                    p,
                    |_| {},
                )
                .set(&mut wall_spawn_button);
                // turret spawn button
                button(
                    spawn_menu_button(Some("objects/turret/machineGun.png")),
                    p,
                    |_| {},
                )
                .set(&mut turret_spawn_button);
            });
        },
    );

    commands
        .entity(wall_spawn_button.unwrap())
        .insert(WallSpawnButton);
    commands
        .entity(pawn_entity.unwrap())
        .insert(PawnResourceCounter);
    commands
        .entity(enemy_entity.unwrap())
        .insert(EnemyResourceCounter);
    commands
        .entity(pawn_spawn_button.unwrap())
        .insert(PawnSpawnButton);
    commands
        .entity(turret_spawn_button.unwrap())
        .insert(TurretSpawnButton);

    commands
        .entity(resource_entity.unwrap())
        .insert(GameResourceCounter);
    commands.entity(root_entity).insert(GameStateUI);
}

fn destroy_game_state_ui(mut commands: Commands, query: Query<Entity, With<GameStateUI>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn update_resource_counter(
    game_resources: Res<GameResources>,
    mut query: Query<&mut Text, With<GameResourceCounter>>,
) {
    for mut text in &mut query {
        text.sections[0].value = game_resources.stone.to_string();
    }
}

fn update_pawn_counter(
    game_resources: Res<GameResources>,
    mut query: Query<&mut Text, With<PawnResourceCounter>>,
) {
    for mut text in &mut query {
        text.sections[0].value = game_resources.pawns.to_string();
    }
}

fn update_enemy_counter(
    enemy_wave: Res<crate::pawn::EnemyWave>,
    mut query: Query<&mut Text, With<EnemyResourceCounter>>,
) {
    for mut text in &mut query {
        text.sections[0].value = enemy_wave.enemies.to_string();
    }
}

fn listen_for_spawn_pawn(
    pawn_spawn_button: Query<&Interaction, (With<PawnSpawnButton>, Changed<Interaction>)>,
    mut events: EventWriter<SpawnPawnRequestEvent>,
) {
    for interaction in pawn_spawn_button.iter() {
        if let Interaction::Pressed = interaction {
            events.send(SpawnPawnRequestEvent);
        }
    }
}

fn listen_for_wall_spawn(
    wall_spawn_button: Query<&Interaction, (With<WallSpawnButton>, Changed<Interaction>)>,
    mut update_world_state: ResMut<NextState<WorldInteraction>>,
    wall_resource: Res<crate::assets::walls::Wall>,
    mut placeable_item: ResMut<crate::placeable::CurrentPlaceableItem>,
) {
    for interaction in wall_spawn_button.iter() {
        if let Interaction::Pressed = interaction {
            update_world_state.set(WorldInteraction::Placing);

            placeable_item.0 = Some(placeable_components::PlaceableBundle {
                sprite_bundle: SpriteBundle {
                    texture: wall_resource.stone.clone(),
                    sprite: Sprite {
                        custom_size: Some(Vec2::new(TILE_SIZE, TILE_SIZE)),
                        anchor: Anchor::BottomLeft,
                        ..default()
                    },
                    ..default()
                },
                placeable: placeable_components::PlaceableType::Wall(placeable_components::Wall {
                    max_resources: 50,
                    ..default()
                }),
            });
        }
    }
}

fn listen_for_turret_spawn(
    turret_spawn_button: Query<&Interaction, (With<TurretSpawnButton>, Changed<Interaction>)>,
    mut update_world_state: ResMut<NextState<WorldInteraction>>,
    mut placeable_item: ResMut<crate::placeable::CurrentPlaceableItem>,
) {
    for interaction in turret_spawn_button.iter() {
        if let Interaction::Pressed = interaction {
            update_world_state.set(WorldInteraction::Placing);
            // TODO! Spawn a turret here
        }
    }
}
