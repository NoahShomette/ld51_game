mod definitions_units;
mod enemy_spawner;
mod game_state;
mod generic_components;
mod map;

use crate::definitions_units::{Enemy, Health, Player, PlayerStats};
use crate::enemy_spawner::{SpawnEvents, Spawner};
use crate::game_state::{GamePlayState, GameStateInfo};
use bevy::math::vec2;
use bevy::prelude::*;
use bevy::sprite::collide_aabb::Collision;
use bevy::time::FixedTimestep;
use bevy::window::{close_on_esc, WindowMode};
use bevy_ecs_tilemap::TilemapPlugin;
use bevy_rapier2d::prelude::*;
use bevy_rapier2d::rapier::prelude::CollisionEventFlags;
use std::ops::ControlFlow;
use std::ops::ControlFlow::Break;

const TIME_STEP: f32 = 1.0 / 30.0;
const MAX_OBJECT_DISTANCE: f32 = 3000.;

const FONT_ASSET_PATH: &str = ("OpenSans-ExtraBold.ttf");

const SCOREBOARD_FONT_SIZE: f32 = 40.0;
const SCOREBOARD_TEXT_PADDING: Val = Val::Px(5.0);
const SCORE_COLOR: Color = Color::rgb(1.0, 0.5, 0.5);
const TEXT_COLOR: Color = Color::rgb(0.5, 0.5, 1.0);

fn main() {
    App::new()
        // setups
        .insert_resource(ClearColor(Color::rgb(0., 0., 0.)))
        .insert_resource(WindowDescriptor {
            //position: WindowPosition::Centered(MonitorSelection::Current),
            //title: String::from("val"),
            //height:
            // resizable: false,
            //decorations: false,
            mode: WindowMode::BorderlessFullscreen,
            ..default()
        })
        //basics
        .add_system(close_on_esc)
        //plugins and tools
        .add_plugins(DefaultPlugins)
        // bevy rapier
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(32.0))
        //.add_plugin(RapierDebugRenderPlugin::default())
        //bevy ecs tilemap
        //.add_plugin(TilemapPlugin)
        //.add_startup_system(map::setup_map)
        // events
        .add_event::<TickEvent>()
        .add_event::<HealthGone>()
        .add_event::<GamePlayState>()
        .add_event::<SpawnEvents>()
        // resources
        .init_resource::<GameTickInfo>()
        .init_resource::<GameStateInfo>()
        .init_resource::<Spawner>()
        .init_resource::<PlayerInput>()
        .init_resource::<PlayerStats>()
        // startup systems
        .add_startup_system(setup_game_core)
        .add_startup_system(setup_player)
        //
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(TIME_STEP as f64))
                .with_system(handle_enemy_ai),
        )
        //generic loop systems
        .add_system(game_tick_manager)
        .add_system(handle_tick_events)
        .add_system(handle_spawn_events)
        .add_system(handle_player_colliding)
        .add_system(handle_score_events)
        .add_system(handle_player_death)
        .add_system(player_menu_controls)
        // specialized systems
        .add_system(player_movement)
        //
        .run();
}

pub struct PlayerInput {
    pub is_holding_forward: bool,
    pub is_holding_turn: bool,
}

impl FromWorld for PlayerInput {
    fn from_world(world: &mut World) -> Self {
        PlayerInput {
            is_holding_forward: false,
            is_holding_turn: false,
        }
    }
}

#[derive(Default)]
struct TickEvent {}
#[derive(Default)]
pub struct HealthGone {}

#[derive(Component)]
pub struct HealthText;

pub struct GameTickInfo {
    do_tick: bool,
    base_time_between_ticks: f32,
    time_between_ticks: f32,
    time_till_next_tick: f32,
}
impl FromWorld for GameTickInfo {
    fn from_world(world: &mut World) -> Self {
        GameTickInfo {
            do_tick: false,
            base_time_between_ticks: 1.,
            time_between_ticks: 1.,
            time_till_next_tick: 0.,
        }
    }
}

fn setup_game_core(mut commands: Commands, asset_server: ResMut<AssetServer>) {
    commands
        .spawn_bundle(Camera2dBundle::default())
        .insert(definitions_units::PlayerCam);
    commands
        .spawn_bundle(
            TextBundle::from_sections([
                TextSection::new(
                    "HEALTH: ",
                    TextStyle {
                        font: asset_server.load(FONT_ASSET_PATH),
                        font_size: SCOREBOARD_FONT_SIZE,
                        color: TEXT_COLOR,
                    },
                ),
                TextSection::from_style(TextStyle {
                    font: asset_server.load(FONT_ASSET_PATH),
                    font_size: SCOREBOARD_FONT_SIZE,
                    color: SCORE_COLOR,
                }),
                TextSection::new(
                    "",
                    TextStyle {
                        font: asset_server.load(FONT_ASSET_PATH),
                        font_size: SCOREBOARD_FONT_SIZE,
                        color: TEXT_COLOR,
                    },
                ),
                TextSection::from_style(TextStyle {
                    font: asset_server.load(FONT_ASSET_PATH),
                    font_size: SCOREBOARD_FONT_SIZE,
                    color: SCORE_COLOR,
                }),
            ])
            .with_style(Style {
                position_type: PositionType::Absolute,
                position: UiRect {
                    top: SCOREBOARD_TEXT_PADDING,
                    left: SCOREBOARD_TEXT_PADDING,
                    ..default()
                },
                ..default()
            }),
        )
        .insert(HealthText {});
}

fn setup_player(mut commands: Commands) {
    commands.spawn_bundle(definitions_units::PlayerBundle::new());
}

fn game_tick_manager(
    time: Res<Time>,
    mut game_tick_time: ResMut<GameTickInfo>,
    mut game_state: ResMut<GameStateInfo>,
    mut event_writer: EventWriter<TickEvent>,
) {
    match game_state.game_state {
        GamePlayState::Menu => {}
        GamePlayState::Win => {}
        GamePlayState::Lose => {}
        GamePlayState::Playing => {
            game_tick_time.time_till_next_tick += time.delta().as_secs_f32();
            if game_tick_time.time_till_next_tick >= game_tick_time.time_between_ticks {
                game_tick_time.time_till_next_tick -= game_tick_time.time_between_ticks;
                event_writer.send(default());
            }
        }
    }
}

fn handle_score_events(
    mut event_reader: EventReader<TickEvent>,
    mut player_stats: ResMut<PlayerStats>,
    mut text_query: Query<(&mut Text, &HealthText)>,
) {
    let (mut score_text, _score_text_component) = text_query.single_mut();
    score_text.sections[1].value = format!("{}", player_stats.health);
}

fn player_menu_controls(
    keyboard_input: Res<Input<KeyCode>>,

    mut player_entity: Query<
        (Entity, &mut Transform, &mut Velocity),
        (With<Player>, Without<Enemy>),
    >,
    mut enemy_velocity: Query<(Entity), (With<(Enemy)>, Without<Player>)>,
    mut health_entity: Query<(Entity), With<(Health)>>,

    mut player_stats: ResMut<PlayerStats>,
    mut game_state: ResMut<GameStateInfo>,
    mut event_writer: EventWriter<GamePlayState>,
    mut game_tick_time: ResMut<GameTickInfo>,
    mut commands: Commands,
) {
    match game_state.game_state {
        GamePlayState::Menu => {
            if keyboard_input.pressed(KeyCode::Space) {
                game_state.change_game_play_state(GamePlayState::Playing, &mut event_writer);
                game_tick_time.do_tick = true;
                info!("Game Started");
            }
        }
        GamePlayState::Win => {}
        GamePlayState::Lose => {
            if keyboard_input.pressed(KeyCode::Space) {
                restart_game(player_entity, enemy_velocity, health_entity, player_stats, game_state, event_writer, game_tick_time, commands);
                info!("Game Started");
            }
        }
        GamePlayState::Playing => {}
    }
}

fn player_movement(
    keyboard_input: Res<Input<KeyCode>>,
    mut game_tick_time: ResMut<GameTickInfo>,
    mut game_state: ResMut<GameStateInfo>,
    mut event_writer: EventWriter<GamePlayState>,

    mut player_stats: ResMut<PlayerStats>,
    mut player_input: ResMut<PlayerInput>,
    mut player_velocity: Query<(&mut Velocity, &Transform), With<definitions_units::Player>>,
    mut camera: Query<
        &mut Transform,
        (
            With<Camera>,
            With<definitions_units::PlayerCam>,
            Without<definitions_units::Player>,
        ),
    >,
) {
    let (mut velocity, transform) = player_velocity.single_mut();
    let (mut cam_transform) = camera.single_mut();

    match game_state.game_state {
        GamePlayState::Menu => {} //implemented in different function for clarity
        GamePlayState::Win => {}
        GamePlayState::Lose => {} //implemented in different function for clarity
        GamePlayState::Playing => {
            if keyboard_input.just_released(KeyCode::W) {
                player_input.is_holding_forward = false;
                player_stats.current_speed = Vec3::ZERO;
            }

            if keyboard_input.just_released(KeyCode::A) || keyboard_input.just_released(KeyCode::D)
            {
                player_input.is_holding_turn = false;
            }

            if keyboard_input.pressed(KeyCode::W) {
                player_input.is_holding_forward = true;
                let speed_per_frame = player_stats.speed_per_frame;
                player_stats.add_forward_speed(speed_per_frame);
                let rotated_velocity = transform.rotation * (player_stats.current_speed);
                velocity.linvel = rotated_velocity.truncate();
            }
            if keyboard_input.pressed(KeyCode::A) {
                velocity.angvel = 1. * 5.;
            }
            if keyboard_input.pressed(KeyCode::D) {
                velocity.angvel = -1. * 5.;
            }
        }
    }

    cam_transform.translation = transform.translation;
}

fn handle_tick_events(
    mut spawn_event_writer: EventWriter<SpawnEvents>,
    mut tick_event_reader: EventReader<TickEvent>,
    mut player_stats: ResMut<PlayerStats>,
    mut player_health_event: EventWriter<HealthGone>,
) {
    for tick in tick_event_reader.iter() {
        player_stats.health_damage(1, &mut player_health_event);
        spawn_event_writer.send(SpawnEvents(false));
    }
}

fn handle_spawn_events(
    mut game_tick_time: ResMut<GameTickInfo>,
    mut enemy_spawner_resource: ResMut<Spawner>,
    mut commands: Commands,
    mut player_transform: Query<(&Transform), (With<Player>, Without<Enemy>)>,

    mut spawn_events: EventReader<SpawnEvents>,
) {
    for event in spawn_events.iter() {
        Spawner::spawn_next_wave(
            &mut player_transform,
            &game_tick_time,
            &enemy_spawner_resource,
            &mut commands,
        );

        Spawner::spawn_health(
            &mut player_transform,
            &game_tick_time,
            &enemy_spawner_resource,
            &mut commands,
        );
    }
}

fn handle_enemy_ai(
    mut commands: Commands,
    mut player_velocity: Query<(&Transform), (With<Player>, Without<Enemy>)>,
    mut enemy_velocity: Query<
        (Entity, &mut Velocity, &mut Transform),
        (With<(Enemy)>, Without<Player>),
    >,
    mut game_state: ResMut<GameStateInfo>,
) {
    if game_state.game_state != GamePlayState::Playing {
        return;
    }
    let (player_transform) = player_velocity.single_mut();
    let mut enemy_count = 0;
    for (entity, mut velocity, mut transform) in enemy_velocity.iter_mut() {
        let distance_to_player = player_transform.translation - transform.translation;

        if distance_to_player.x > MAX_OBJECT_DISTANCE || distance_to_player.y > MAX_OBJECT_DISTANCE
        {
            commands.entity(entity).despawn();
        }

        let mut angle = f32::atan2(
            player_transform.translation.y - transform.translation.y,
            player_transform.translation.x - transform.translation.x,
        );

        transform.rotation = Quat::from_rotation_z(angle);
        let rotated_velocity = transform.rotation
            * Vec3 {
                x: 200.0,
                y: 0.0,
                z: 0.0,
            };
        velocity.linvel = rotated_velocity.truncate();

        enemy_count += 1;
    }
    info!("{}", enemy_count);
}

fn handle_player_colliding(
    mut active_events: EventReader<CollisionEvent>,
    player: Query<&CollidingEntities, With<Player>>,
    mut enemy_entity: Query<(&Enemy)>,
    mut health_entity: Query<(&Health)>,
    //mut powerup_entity: Query<(&PowerUp)>,
    mut player_stats: ResMut<PlayerStats>,
    mut health_event: EventWriter<HealthGone>,
    mut commands: Commands,
) {


    for player in player.iter() {
        for collision in player.iter() {
            if let Ok(health) = health_entity.get(collision){
                player_stats.health_heal_up_to_ten();
                info!("HEALING EVENT");
                commands.entity(collision).despawn();
            }
        }
    }

    for player in player.iter() {
        for collision in player.iter() {
            if let Ok(enemy) = enemy_entity.get(collision){
                player_stats.health_damage(2, &mut health_event);
                info!("collision death event happening");
                commands.entity(collision).despawn();
            }


        }
    }
}

fn handle_player_death(
    mut health_event: EventReader<HealthGone>,
    mut game_state: ResMut<GameStateInfo>,
    mut event_writer: EventWriter<GamePlayState>,
) {
    for event in health_event.iter() {
        game_state.change_game_play_state(GamePlayState::Lose, &mut event_writer);
    }
}

fn restart_game(
    mut player_entity: Query<
        (Entity, &mut Transform, &mut Velocity),
        (With<Player>, Without<Enemy>),
    >,
    mut enemy_velocity: Query<(Entity), (With<(Enemy)>, Without<Player>)>,
    mut health_entity: Query<(Entity), With<(Health)>>,
    mut player_stats: ResMut<PlayerStats>,
    mut game_state: ResMut<GameStateInfo>,
    mut event_writer: EventWriter<GamePlayState>,
    mut game_tick_time: ResMut<GameTickInfo>,
    mut commands: Commands,
) {
    for (entity) in enemy_velocity.iter_mut() {
        commands.entity(entity).despawn();
    }
    
    for entity in health_entity.iter_mut(){
        commands.entity(entity).despawn();

    }

    for (entity, mut transform, mut velocity) in player_entity.iter_mut() {
        transform.translation = Vec3 {
            x: 0.0,
            y: 0.0,
            z: 100.0,
        };

        velocity.linvel = Vec2 { x: 0.0, y: 0.0 };
        velocity.angvel = 0.;
    }
    game_tick_time.do_tick = true;
    player_stats.health_heal_up_to_ten();
    game_state.change_game_play_state(GamePlayState::Playing, &mut event_writer);
}
