mod definitions_units;
mod enemy_spawner;
mod game_state;
mod generic_components;
mod map;

use crate::definitions_units::{
    Enemy, Health, Player, PlayerStats, Powerup, ENEMY_COLOR, PLAYER_COLOR, POWERUP_COLOR,
};
use crate::enemy_spawner::{SpawnEvents, Spawner};
use crate::game_state::{GamePlayState, GameStateInfo};
use bevy::app::AppExit;
use bevy::prelude::*;
use bevy::time::FixedTimestep;
use bevy::window::{close_on_esc, WindowMode};
use bevy_kira_audio::*;
use bevy_rapier2d::prelude::*;
const HEALTH_PICKUP_ASSET_PATH: &str = ("483602__raclure__game-bump.mp3");
const POWERUP_PICKUP_ASSET_PATH: &str = ("344522__jeremysykes__powerup05.wav");
const ENEMY_COLLISION_KILL_SOUND_ASSET_PATH: &str = ("242857__plasterbrain__coin-get.ogg");
const ENEMY_COLLISION_DAMAGE_SOUND_ASSET_PATH: &str = ("391667__jeckkech__put.wav");
const DEATH_SOUND_ASSET_PATH: &str = ("538151__fupicat__8bit-fall.wav");
const GAME_START_SOUND_ASSET_PATH: &str = ("455021__tissman__checkpoint.wav");
const PLAYER_TURN_SOUND_ASSET_PATH: &str = ("483602__raclure__game-bump.mp3");
const BG_MUSIC_ASSET_PATH: &str = ("483602__raclure__game-bump.mp3");

const TIME_STEP: f32 = 1.0 / 30.0;
const MAX_OBJECT_DISTANCE: f32 = 3000.;

const FONT_ASSET_PATH: &str = ("OpenSans-ExtraBold.ttf");

const HEALTH_TEXT_PADDING: Val = Val::Px(1920.0 / 2.);

const HEALTH_FONT_SIZE: f32 = 40.0;
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
        //.add_system(close_on_esc)
        //plugins and tools
        .add_plugins(DefaultPlugins)
        .add_plugin(AudioPlugin)
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
        .init_resource::<Score>()
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
        .add_system(update_ui)
        .add_system(handle_player_death)
        .add_system(player_menu_controls)
        // specialized systems
        .add_system(player_movement)
        //
        .run();
}

enum AudioType {
    HealthPickup,
    PowerupPickup,
    EnemyCollisionDamage,
    EnemyCollisionKillMode,
    Death,
    GameStart,
    PlayerTurn,
}

impl AudioType {
    pub fn return_asset_path(&self) -> &str {
        match self {
            AudioType::HealthPickup => HEALTH_PICKUP_ASSET_PATH,
            AudioType::PowerupPickup => POWERUP_PICKUP_ASSET_PATH,
            AudioType::EnemyCollisionDamage => ENEMY_COLLISION_DAMAGE_SOUND_ASSET_PATH,
            AudioType::EnemyCollisionKillMode => ENEMY_COLLISION_DAMAGE_SOUND_ASSET_PATH,
            AudioType::Death => DEATH_SOUND_ASSET_PATH,
            AudioType::GameStart => GAME_START_SOUND_ASSET_PATH,
            AudioType::PlayerTurn => PLAYER_TURN_SOUND_ASSET_PATH,
        }
    }
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

pub struct Score {
    score: f32,
}

impl FromWorld for Score {
    fn from_world(world: &mut World) -> Self {
        Score { score: 0. }
    }
}

#[derive(Default)]
struct TickEvent {}
#[derive(Default)]
pub struct HealthGone {}

#[derive(Component)]
pub struct PlayingText; //used to enable and disable all playing text
#[derive(Component)]
pub struct RunText;
#[derive(Component)]
pub struct HealthText;
#[derive(Component)]
pub struct PowerupText;
#[derive(Component)]
pub struct ScoreText;
#[derive(Component)]
pub struct MenuText; // used to enable and disable menu text when in menu
#[derive(Component)]
pub struct LoseScoreText; //updated to show final score at end of game
#[derive(Component)]
pub struct LoseText; //all lose text to enable and disable lose text at end of game
#[derive(Component)]
pub struct PauseText; //all lose text to enable and disable lose text at end of game
#[derive(Component)]
pub struct ResumeText;

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

fn setup_game_core(
    mut commands: Commands,
    mut asset_server: ResMut<AssetServer>,
    audio: Res<Audio>,
) {
    commands
        .spawn_bundle(Camera2dBundle::default())
        .insert(definitions_units::PlayerCam);
    setup_playing_ui(&mut commands, &mut asset_server);
    setup_menu_ui(&mut commands, &mut asset_server);
    setup_lose_ui(&mut commands, &mut asset_server);
    audio
        .play(asset_server.load(
            "651183__josefpres__8-bit-music-loop-002-part-02-simple-mix-02-short-loop-120-bpm.wav",
        ))
        .with_volume(0.2)
        .loop_from(0.5);
}
fn setup_playing_ui(mut commands: &mut Commands, asset_server: &mut ResMut<AssetServer>) {
    commands
        .spawn_bundle(
            // Create a TextBundle that has a Text with a single section.
            TextBundle::from_section(
                // Accepts a `String` or any type that converts into a `String`, such as `&str`
                "10",
                TextStyle {
                    font: asset_server.load(FONT_ASSET_PATH),
                    font_size: HEALTH_FONT_SIZE - 10.,
                    color: PLAYER_COLOR,
                },
            ) // Set the alignment of the Text
            .with_text_alignment(TextAlignment::TOP_CENTER)
            // Set the style of the TextBundle itself.
            .with_style(Style {
                align_self: AlignSelf::FlexEnd,
                position_type: PositionType::Absolute,
                position: UiRect {
                    top: Val::Percent(52.),
                    left: Val::Percent(48.),
                    ..default()
                },
                ..default()
            }),
        )
        .insert(HealthText)
        .insert(PlayingText);

    commands
        .spawn_bundle(
            // Create a TextBundle that has a Text with a single section.
            TextBundle::from_section(
                // Accepts a `String` or any type that converts into a `String`, such as `&str`
                "10",
                TextStyle {
                    font: asset_server.load(FONT_ASSET_PATH),
                    font_size: HEALTH_FONT_SIZE - 10.,
                    color: POWERUP_COLOR,
                },
            ) // Set the alignment of the Text
            .with_text_alignment(TextAlignment::TOP_CENTER)
            // Set the style of the TextBundle itself.
            .with_style(Style {
                align_self: AlignSelf::FlexEnd,
                position_type: PositionType::Absolute,
                position: UiRect {
                    top: Val::Percent(52.),
                    left: Val::Percent(51.),
                    ..default()
                },
                ..default()
            }),
        )
        .insert(PowerupText)
        .insert(PlayingText);
    commands
        .spawn_bundle(
            // Create a TextBundle that has a Text with a single section.
            TextBundle::from_section(
                // Accepts a `String` or any type that converts into a `String`, such as `&str`
                "RUN!",
                TextStyle {
                    font: asset_server.load(FONT_ASSET_PATH),
                    font_size: HEALTH_FONT_SIZE + 10.,
                    color: ENEMY_COLOR,
                },
            ) // Set the alignment of the Text
            .with_text_alignment(TextAlignment::TOP_CENTER)
            // Set the style of the TextBundle itself.
            .with_style(Style {
                align_self: AlignSelf::FlexEnd,
                position_type: PositionType::Absolute,
                position: UiRect {
                    top: Val::Percent(47.),
                    left: Val::Percent(53.),
                    ..default()
                },
                ..default()
            }),
        )
        .insert(RunText)
        .insert(PlayingText);

    commands
        .spawn_bundle(
            TextBundle::from_sections([
                TextSection::new(
                    "SCORE: ",
                    TextStyle {
                        font: asset_server.load(FONT_ASSET_PATH),
                        font_size: HEALTH_FONT_SIZE,
                        color: TEXT_COLOR,
                    },
                ),
                TextSection::from_style(TextStyle {
                    font: asset_server.load(FONT_ASSET_PATH),
                    font_size: HEALTH_FONT_SIZE,
                    color: SCORE_COLOR,
                }),
                TextSection::new(
                    "",
                    TextStyle {
                        font: asset_server.load(FONT_ASSET_PATH),
                        font_size: HEALTH_FONT_SIZE,
                        color: TEXT_COLOR,
                    },
                ),
                TextSection::from_style(TextStyle {
                    font: asset_server.load(FONT_ASSET_PATH),
                    font_size: HEALTH_FONT_SIZE,
                    color: SCORE_COLOR,
                }),
            ])
            .with_text_alignment(TextAlignment::TOP_CENTER)
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
        .insert(ScoreText)
        .insert(PlayingText);
}
fn setup_menu_ui(mut commands: &mut Commands, asset_server: &mut ResMut<AssetServer>) {
    commands
        .spawn_bundle(
            // Create a TextBundle that has a Text with a single section.
            TextBundle::from_section(
                // Accepts a `String` or any type that converts into a `String`, such as `&str`
                "Press (Escape) to quit",
                TextStyle {
                    font: asset_server.load(FONT_ASSET_PATH),
                    font_size: HEALTH_FONT_SIZE,
                    color: Color::WHITE,
                },
            ) // Set the alignment of the Text
            .with_text_alignment(TextAlignment::TOP_CENTER)
            // Set the style of the TextBundle itself.
            .with_style(Style {
                align_self: AlignSelf::FlexEnd,
                position_type: PositionType::Absolute,
                position: UiRect {
                    top: Val::Percent(10.),
                    left: Val::Percent(20.0),
                    ..default()
                },
                ..default()
            }),
        )
        .insert(PauseText);

    commands
        .spawn_bundle(
            // Create a TextBundle that has a Text with a single section.
            TextBundle::from_section(
                // Accepts a `String` or any type that converts into a `String`, such as `&str`
                "Press (Space) to resume",
                TextStyle {
                    font: asset_server.load(FONT_ASSET_PATH),
                    font_size: HEALTH_FONT_SIZE,
                    color: Color::WHITE,
                },
            ) // Set the alignment of the Text
            .with_text_alignment(TextAlignment::TOP_CENTER)
            // Set the style of the TextBundle itself.
            .with_style(Style {
                align_self: AlignSelf::FlexEnd,
                position_type: PositionType::Absolute,
                position: UiRect {
                    top: Val::Percent(35.),
                    left: Val::Percent(35.0),
                    ..default()
                },
                ..default()
            }),
        )
        .insert(PauseText)
        .insert(ResumeText);

    commands
        .spawn_bundle(
            // Create a TextBundle that has a Text with a single section.
            TextBundle::from_section(
                // Accepts a `String` or any type that converts into a `String`, such as `&str`
                "RED HOARD",
                TextStyle {
                    font: asset_server.load(FONT_ASSET_PATH),
                    font_size: HEALTH_FONT_SIZE + 40.,
                    color: ENEMY_COLOR,
                },
            ) // Set the alignment of the Text
            .with_text_alignment(TextAlignment::TOP_CENTER)
            // Set the style of the TextBundle itself.
            .with_style(Style {
                align_self: AlignSelf::FlexEnd,
                position_type: PositionType::Absolute,
                position: UiRect {
                    top: Val::Percent(35.),
                    left: Val::Percent(41.0),
                    ..default()
                },
                ..default()
            }),
        )
        .insert(MenuText);

    commands
        .spawn_bundle(
            // Create a TextBundle that has a Text with a single section.
            TextBundle::from_section(
                // Accepts a `String` or any type that converts into a `String`, such as `&str`
                "Press Space to start",
                TextStyle {
                    font: asset_server.load(FONT_ASSET_PATH),
                    font_size: HEALTH_FONT_SIZE - 10.,
                    color: Color::WHITE,
                },
            ) // Set the alignment of the Text
            .with_text_alignment(TextAlignment::TOP_CENTER)
            // Set the style of the TextBundle itself.
            .with_style(Style {
                align_self: AlignSelf::FlexEnd,
                position_type: PositionType::Absolute,
                position: UiRect {
                    top: Val::Percent(60.),
                    left: Val::Percent(44.0),
                    ..default()
                },
                ..default()
            }),
        )
        .insert(MenuText);
}

fn setup_lose_ui(mut commands: &mut Commands, asset_server: &mut ResMut<AssetServer>) {
    commands
        .spawn_bundle(
            // Create a TextBundle that has a Text with a single section.
            TextBundle::from_section(
                // Accepts a `String` or any type that converts into a `String`, such as `&str`
                "FINAL SCORE:",
                TextStyle {
                    font: asset_server.load(FONT_ASSET_PATH),
                    font_size: HEALTH_FONT_SIZE + 20.,
                    color: Color::WHITE,
                },
            ) // Set the alignment of the Text
            .with_text_alignment(TextAlignment::TOP_CENTER)
            // Set the style of the TextBundle itself.
            .with_style(Style {
                align_self: AlignSelf::FlexEnd,
                position_type: PositionType::Absolute,
                position: UiRect {
                    top: Val::Percent(30.),
                    left: Val::Percent(35.0),
                    ..default()
                },
                ..default()
            }),
        )
        .insert(LoseText);

    commands
        .spawn_bundle(
            // Create a TextBundle that has a Text with a single section.
            TextBundle::from_section(
                // Accepts a `String` or any type that converts into a `String`, such as `&str`
                "",
                TextStyle {
                    font: asset_server.load(FONT_ASSET_PATH),
                    font_size: HEALTH_FONT_SIZE + 40.,
                    color: PLAYER_COLOR,
                },
            ) // Set the alignment of the Text
            .with_text_alignment(TextAlignment::TOP_CENTER)
            // Set the style of the TextBundle itself.
            .with_style(Style {
                align_self: AlignSelf::FlexEnd,
                position_type: PositionType::Absolute,
                position: UiRect {
                    top: Val::Percent(40.),
                    left: Val::Percent(45.0),
                    ..default()
                },
                ..default()
            }),
        )
        .insert(LoseScoreText)
        .insert(LoseText);

    commands
        .spawn_bundle(
            // Create a TextBundle that has a Text with a single section.
            TextBundle::from_section(
                // Accepts a `String` or any type that converts into a `String`, such as `&str`
                "Press Space to go again!",
                TextStyle {
                    font: asset_server.load(FONT_ASSET_PATH),
                    font_size: HEALTH_FONT_SIZE - 10.,
                    color: Color::WHITE,
                },
            ) // Set the alignment of the Text
            .with_text_alignment(TextAlignment::TOP_CENTER)
            // Set the style of the TextBundle itself.
            .with_style(Style {
                align_self: AlignSelf::FlexEnd,
                position_type: PositionType::Absolute,
                position: UiRect {
                    top: Val::Percent(60.),
                    left: Val::Percent(50.0),
                    ..default()
                },
                ..default()
            }),
        )
        .insert(LoseText);
}

fn update_ui(
    score: Res<Score>,
    game_state: Res<GameStateInfo>,
    player_stats: Res<PlayerStats>,
    mut playing_text_query: Query<
        (
            &mut Text,
            &mut Visibility,
            Option<&HealthText>,
            Option<&PowerupText>,
            Option<&RunText>,
            Option<&ScoreText>,
        ),
        (With<PlayingText>, Without<MenuText>, Without<LoseText>),
    >,
    mut menu_text_query: Query<
        (&mut Text, &mut Visibility),
        (With<MenuText>, Without<PlayingText>, Without<LoseText>),
    >,
    mut lose_text_query: Query<
        (&mut Text, &mut Visibility, Option<&LoseScoreText>),
        (With<LoseText>, Without<PlayingText>, Without<MenuText>),
    >,
    mut pause_text_query: Query<(&mut Text, &mut Visibility, Option<&ResumeText>),
        (
            With<PauseText>,
            Without<PlayingText>,
            Without<MenuText>,
            Without<LoseText>,
        ),
    >,
) {
    for (mut text, mut visibility, health_text, powerup_text, run_text, score_text) in
        playing_text_query.iter_mut()
    {
        match game_state.game_state {
            GamePlayState::Menu => {
                visibility.is_visible = false;
            }
            GamePlayState::Pause => {
                visibility.is_visible = false;
            }
            GamePlayState::Lose => {
                visibility.is_visible = false;
            }
            GamePlayState::Playing => {
                visibility.is_visible = true;
                if let Some(text_comp) = health_text {
                    text.sections[0].value = format!("{}", player_stats.health);
                }
                if let Some(text_comp) = powerup_text {
                    text.sections[0].value = format!("{}", player_stats.time_left_in_kill_mode);
                }
                if let Some(text_comp) = run_text {
                    if score.score >= 3. {
                        visibility.is_visible = false;
                    }
                    text.sections[0].value = format!("RUN!");
                }
                if let Some(text_comp) = score_text {
                    text.sections[1].value = format!("{}", score.score);
                    info!(score.score);
                }
            }
        }
    }
    for (mut text, mut visibility) in menu_text_query.iter_mut() {
        match game_state.game_state {
            GamePlayState::Menu => {
                visibility.is_visible = true;
            }
            GamePlayState::Pause => {
                visibility.is_visible = false;
            }
            GamePlayState::Lose => {
                visibility.is_visible = false;
            }
            GamePlayState::Playing => {
                visibility.is_visible = false;
            }
        }
    }

    for (mut text, mut visibility, lose_score_text) in lose_text_query.iter_mut() {
        match game_state.game_state {
            GamePlayState::Menu => {
                visibility.is_visible = false;
            }
            GamePlayState::Pause => {
                visibility.is_visible = false;
            }
            GamePlayState::Lose => {
                visibility.is_visible = true;
                if let Some(text_comp) = lose_score_text {
                    text.sections[0].value = format!("{}", score.score);
                    info!(score.score);
                }
            }
            GamePlayState::Playing => {
                visibility.is_visible = false;
            }
        }
    }

    for (mut text, mut visibility, resume_text) in pause_text_query.iter_mut() {
        match game_state.game_state {
            GamePlayState::Menu => {
                visibility.is_visible = true;

                if let Some(text_comp) = resume_text {
                    visibility.is_visible = false;
                }

            }
            GamePlayState::Pause => {
                visibility.is_visible = true;
            }
            GamePlayState::Lose => {
                visibility.is_visible = true;

                if let Some(text_comp) = resume_text {
                    visibility.is_visible = false;
                }
            }
            GamePlayState::Playing => {
                visibility.is_visible = false;
            }
        }
    }
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
        GamePlayState::Pause => {}
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

fn player_menu_controls(
    keyboard_input: Res<Input<KeyCode>>,

    mut player_entity: Query<
        (Entity, &mut Transform, &mut Velocity),
        (With<Player>, Without<Enemy>),
    >,
    mut enemy_velocity: Query<(Entity), (With<(Enemy)>, Without<Player>)>,
    mut health_entity: Query<(Entity), With<(Health)>>,
    mut powerup_entity: Query<(Entity), With<(Powerup)>>,

    mut player_stats: ResMut<PlayerStats>,
    mut game_state: ResMut<GameStateInfo>,
    mut event_writer: EventWriter<GamePlayState>,
    mut game_tick_time: ResMut<GameTickInfo>,
    mut commands: Commands,
    mut score: ResMut<Score>,
    mut audio: Res<Audio>,
    mut asset_server: ResMut<AssetServer>,
    mut exit: EventWriter<AppExit>,
) {
    match game_state.game_state {
        GamePlayState::Menu => {
            if keyboard_input.pressed(KeyCode::Space) {
                game_state.change_game_play_state(GamePlayState::Playing, &mut event_writer);
                game_tick_time.do_tick = true;
                play_sound(&mut asset_server, &mut audio, AudioType::GameStart);
                info!("Game Started");
            }

            if keyboard_input.just_pressed(KeyCode::Escape) {
                exit.send(AppExit);
            }
        }
        GamePlayState::Pause => {
            if keyboard_input.just_pressed(KeyCode::Escape) {
                exit.send(AppExit);
            }

            if keyboard_input.pressed(KeyCode::Space) {
                game_state.change_game_play_state(GamePlayState::Playing, &mut event_writer);
                game_tick_time.do_tick = true;
                play_sound(&mut asset_server, &mut audio, AudioType::GameStart);
                info!("Game Started");
            }
        }
        GamePlayState::Lose => {
            if keyboard_input.pressed(KeyCode::Space) {
                restart_game(
                    player_entity,
                    enemy_velocity,
                    health_entity,
                    powerup_entity,
                    player_stats,
                    game_state,
                    event_writer,
                    game_tick_time,
                    commands,
                    score,
                );
                play_sound(&mut asset_server, &mut audio, AudioType::GameStart);
                info!("Game Started");
            }

            if keyboard_input.just_pressed(KeyCode::Escape) {
                exit.send(AppExit);
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
        GamePlayState::Pause => {}
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

            if keyboard_input.pressed(KeyCode::Escape) {
                game_state.change_game_play_state(GamePlayState::Pause, &mut event_writer);
            }
        }
    }

    cam_transform.translation = transform.translation;
}

fn handle_tick_events(
    mut player_sprite: Query<&mut Sprite, With<Player>>,

    mut spawn_event_writer: EventWriter<SpawnEvents>,
    mut tick_event_reader: EventReader<TickEvent>,
    mut player_stats: ResMut<PlayerStats>,
    mut player_health_event: EventWriter<HealthGone>,
    mut score: ResMut<Score>,
) {
    let mut player_sprite = player_sprite.single_mut();

    for tick in tick_event_reader.iter() {
        player_stats.health_damage(1, &mut player_health_event);
        player_stats.powerup_time_decrease(&mut player_sprite);
        spawn_event_writer.send(SpawnEvents(false));
        score.score += 1.;
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
            &enemy_spawner_resource,
            &mut commands,
        );

        Spawner::spawn_powerup(
            &mut player_transform,
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
    //info!("{}", enemy_count);
}

fn handle_player_colliding(
    mut active_events: EventReader<CollisionEvent>,
    player: Query<&CollidingEntities, With<Player>>,
    mut player_sprite: Query<&mut Sprite, With<Player>>,
    mut enemy_entity: Query<(&Enemy)>,
    mut health_entity: Query<(&Health)>,
    mut powerup_entity: Query<(&Powerup)>,
    mut player_stats: ResMut<PlayerStats>,
    mut health_event: EventWriter<HealthGone>,
    mut commands: Commands,
    mut score: ResMut<Score>,
    mut audio: Res<Audio>,
    mut asset_server: ResMut<AssetServer>,
) {
    let mut player_sprite = player_sprite.single_mut();

    for player in player.iter() {
        for collision in player.iter() {
            if let Ok(health) = health_entity.get(collision) {
                player_stats.health_heal_up_to_ten();
                play_sound(&mut asset_server, &mut audio, AudioType::HealthPickup);
                commands.entity(collision).despawn();
            }
        }
    }

    for player in player.iter() {
        for collision in player.iter() {
            if let Ok(enemy) = enemy_entity.get(collision) {
                if player_stats.kill_mode {
                    score.score += 5.;
                    play_sound(
                        &mut asset_server,
                        &mut audio,
                        AudioType::EnemyCollisionKillMode,
                    );
                } else {
                    player_stats.health_damage(2, &mut health_event);
                    play_sound(
                        &mut asset_server,
                        &mut audio,
                        AudioType::EnemyCollisionDamage,
                    );
                }
                commands.entity(collision).despawn();
            }
        }
    }
    for player in player.iter() {
        for collision in player.iter() {
            if let Ok(powerup) = powerup_entity.get(collision) {
                play_sound(&mut asset_server, &mut audio, AudioType::PowerupPickup);
                player_stats.health_heal_up_to_ten();
                player_stats.powerup_mode(&mut player_sprite);
                commands.entity(collision).despawn();
            }
        }
    }
}

fn handle_player_death(
    mut health_event: EventReader<HealthGone>,
    mut game_state: ResMut<GameStateInfo>,
    mut event_writer: EventWriter<GamePlayState>,
    mut audio: Res<Audio>,
    mut asset_server: ResMut<AssetServer>,
) {
    for event in health_event.iter() {
        game_state.change_game_play_state(GamePlayState::Lose, &mut event_writer);
        play_sound(&mut asset_server, &mut audio, AudioType::Death);
    }
}

fn restart_game(
    mut player_entity: Query<
        (Entity, &mut Transform, &mut Velocity),
        (With<Player>, Without<Enemy>),
    >,
    mut enemy_velocity: Query<(Entity), (With<(Enemy)>, Without<Player>)>,
    mut health_entity: Query<(Entity), With<(Health)>>,
    mut powerup_entity: Query<(Entity), With<(Powerup)>>,
    mut player_stats: ResMut<PlayerStats>,
    mut game_state: ResMut<GameStateInfo>,
    mut event_writer: EventWriter<GamePlayState>,
    mut game_tick_time: ResMut<GameTickInfo>,
    mut commands: Commands,
    mut score: ResMut<Score>,
) {
    for (entity) in enemy_velocity.iter_mut() {
        commands.entity(entity).despawn();
    }

    for entity in health_entity.iter_mut() {
        commands.entity(entity).despawn();
    }

    for entity in powerup_entity.iter_mut() {
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
    score.score = 0.;
    game_tick_time.do_tick = true;
    player_stats.health_heal_up_to_ten();
    player_stats.time_left_in_kill_mode = 0.;
    player_stats.kill_mode = false;
    game_state.change_game_play_state(GamePlayState::Playing, &mut event_writer);
}

fn play_sound(
    asset_server: &mut ResMut<AssetServer>,
    audio: &mut Res<Audio>,
    audio_type: AudioType,
) {
    match audio_type {
        AudioType::HealthPickup => {
            audio
                .play(asset_server.load(HEALTH_PICKUP_ASSET_PATH))
                .with_volume(0.7);
        }
        AudioType::PowerupPickup => {
            audio
                .play(asset_server.load(POWERUP_PICKUP_ASSET_PATH))
                .with_volume(0.7);
        }
        AudioType::EnemyCollisionDamage => {
            audio
                .play(asset_server.load(ENEMY_COLLISION_DAMAGE_SOUND_ASSET_PATH))
                .with_volume(0.7);
        }
        AudioType::EnemyCollisionKillMode => {
            audio
                .play(asset_server.load(ENEMY_COLLISION_KILL_SOUND_ASSET_PATH))
                .with_volume(0.3);
        }
        AudioType::Death => {
            audio
                .play(asset_server.load(DEATH_SOUND_ASSET_PATH))
                .with_volume(1.0);
        }
        AudioType::GameStart => {
            audio
                .play(asset_server.load(GAME_START_SOUND_ASSET_PATH))
                .with_volume(0.2);
        }
        AudioType::PlayerTurn => {
            audio
                .play(asset_server.load(PLAYER_TURN_SOUND_ASSET_PATH))
                .with_volume(0.7);
        }
    }
}
