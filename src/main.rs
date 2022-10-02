mod definitions_units;
mod enemy_spawner;
mod game_state;
mod generic_components;
mod map;

use crate::definitions_units::{Enemy, Player, PlayerInput, PlayerStats};
use crate::enemy_spawner::EnemySpawner;
use crate::game_state::{GamePlayState, GameStateInfo};
use bevy::math::{quat, vec3};
use bevy::prelude::*;
use bevy::time::FixedTimestep;
use bevy::window::close_on_esc;
use bevy_ecs_tilemap::TilemapPlugin;
use bevy_rapier2d::prelude::*;


const TIME_STEP: f32 = 1.0 / 30.0;

fn main() {
    App::new()
        //basics
        .add_system(close_on_esc)
        //plugins and tools
        .insert_resource(ClearColor(Color::rgb(0., 0., 0.)))
        .add_plugins(DefaultPlugins)
        // bevy rapier
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(32.0))
        //.add_plugin(RapierDebugRenderPlugin::default())
        //bevy ecs tilemap
        .add_plugin(TilemapPlugin)
        .add_startup_system(map::setup_map)
        // events
        .add_event::<TickEvent>()
        .add_event::<MegaTickEvent>()
        .add_event::<GamePlayState>()
        // resources
        .init_resource::<GameTickInfo>()
        .init_resource::<GameStateInfo>()
        .init_resource::<enemy_spawner::EnemySpawner>()
        .init_resource::<PlayerInput>()
        .init_resource::<PlayerStats>()
        // startup systems
        .add_startup_system(setup_game_core)
        .add_startup_system(setup_player)
        //
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(TIME_STEP as f64))
                .with_system(handle_enemy_ai)

        )
        //generic loop systems
        .add_system(game_tick_manager)
        .add_system(handle_tick_events)
        // specialized systems
        .add_system(player_movement)
        //
        .run();
}

#[derive(Default)]
struct TickEvent {}
#[derive(Default)]
struct MegaTickEvent {}

pub struct GameTickInfo {
    do_tick: bool,
    tick_count: f32,
    base_time_between_ticks: f32,
    time_between_ticks: f32,
    time_till_next_tick: f32,
}
impl FromWorld for GameTickInfo {
    fn from_world(world: &mut World) -> Self {
        GameTickInfo {
            do_tick: false,
            tick_count: 0.,
            base_time_between_ticks: 1.,
            time_between_ticks: 1.,
            time_till_next_tick: 0.,
        }
    }
}

fn setup_game_core(mut commands: Commands) {
    commands
        .spawn_bundle(Camera2dBundle::default())
        .insert(definitions_units::PlayerCam);
}

fn setup_player(mut commands: Commands) {
    commands.spawn_bundle(definitions_units::PlayerBundle::new());
}

fn game_tick_manager(
    time: Res<Time>,
    mut game_tick_time: ResMut<GameTickInfo>,
    mut event_writer: EventWriter<TickEvent>,
    mut mega_event_writer: EventWriter<MegaTickEvent>,
) {
    if game_tick_time.do_tick {
        game_tick_time.time_till_next_tick += time.delta().as_secs_f32();
        if game_tick_time.time_till_next_tick >= game_tick_time.time_between_ticks {
            game_tick_time.time_till_next_tick -= game_tick_time.time_between_ticks;
            event_writer.send(default());

            game_tick_time.tick_count += 1.;

            if game_tick_time.tick_count % 10. == 0. {
                info!("megaTickEvent");
                mega_event_writer.send(default());
            }
        }
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
        GamePlayState::Menu => {
            if keyboard_input.pressed(KeyCode::Space) {
                game_state.change_game_play_state(GamePlayState::Playing, event_writer);
                game_tick_time.do_tick = true;
                info!("Game Started");
            }
        }
        GamePlayState::Win => {}
        GamePlayState::Lose => {}
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
    mut game_tick_time: ResMut<GameTickInfo>,
    mut enemy_spawner_resource: ResMut<EnemySpawner>,
    mut commands: Commands,
    mut tick_event_reader: EventReader<TickEvent>,
    mega_tick_event_reader: EventReader<MegaTickEvent>,
) {
    for tick in tick_event_reader.iter() {
        EnemySpawner::spawn_next_wave(&game_tick_time, &enemy_spawner_resource, &mut commands);
    }
}

fn handle_enemy_ai(
    mut player_velocity: Query<(&Transform), (With<Player>, Without<Enemy>)>,
    mut enemy_velocity: Query<
        (&mut Velocity, &mut Transform),
        (With<(definitions_units::Enemy)>, Without<Player>),
    >,
) {
    let (player_transform) = player_velocity.single_mut();
    let mut enemy_count = 0;
    for (mut velocity, mut transform) in enemy_velocity.iter_mut() {
        //let angle = transform.translation.angle_between(player_transform.translation);
        let mut angle = f32::atan2(
            player_transform.translation.y - transform.translation.y,
             player_transform.translation.x - transform.translation.x,
        );

        transform.rotation = Quat::from_rotation_z(angle);
        let rotated_velocity = transform.rotation * Vec3{ x: 200.0, y: 0.0, z: 0.0 };
        velocity.linvel = rotated_velocity.truncate();

        enemy_count += 1;
    }
    info!("{}", enemy_count);
}
