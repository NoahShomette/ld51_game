use crate::definitions_units::*;
use crate::{definitions_units, GameTickInfo};
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use rand::*;

const SCREEN_SAFE_WIDTH: f32 = 2050.;
const SCREEN_SAFE_HEIGHT: f32 = 1150.;
const SPAWN_WIDTH: f32 = 50.;

#[derive(Default)]
pub struct SpawnEvents(pub bool);

pub struct Spawner {
    amount_to_spawn_next_tick: u32,
}

impl Spawner {
    pub fn spawn_next_wave(
        mut player_transform: &mut Query<&Transform, (With<Player>, Without<Enemy>)>,

        game_tick_time: &ResMut<GameTickInfo>,
        mut enemy_spawner: &ResMut<Spawner>,
        mut commands: &mut Commands,
    ) {
        let (player_transform) = player_transform.single_mut();

        let mut rng = thread_rng();

        let mut x_position: f32 = 0.;
        let mut y_position: f32 = 0.;

        for new_enemy in 0..enemy_spawner.amount_to_spawn_next_tick {
            let location = rng.gen_range(1..=4);
            let playerx = player_transform.translation.x;
            let playery = player_transform.translation.y;
            match location {
                1 => {
                    //top
                    x_position = rng.gen_range((playerx - 1920.)..(playerx + SCREEN_SAFE_WIDTH));
                    y_position = rng.gen_range(
                        (player_transform.translation.y - 1150. - 100.)
                            ..(player_transform.translation.y - 1080.),
                    );
                }
                2 => {
                    //right
                    x_position =
                        rng.gen_range((playerx + 1080.)..(playerx + SCREEN_SAFE_WIDTH + 100.));
                    y_position = rng
                        .gen_range((playery - SCREEN_SAFE_HEIGHT)..(playery + SCREEN_SAFE_HEIGHT));
                }
                3 => {
                    //bottom
                    x_position =
                        rng.gen_range((playerx - SCREEN_SAFE_WIDTH)..(playerx + SCREEN_SAFE_WIDTH));
                    y_position = rng.gen_range(
                        (player_transform.translation.y + 1080.)
                            ..(player_transform.translation.y + 1150. + 100.),
                    );
                }
                _ => {
                    //left and everything else
                    x_position =
                        rng.gen_range((playerx - SCREEN_SAFE_WIDTH + 100.)..(playerx - 1920.));
                    y_position =
                        rng.gen_range((playery - 1080. - 100.)..(playery + SCREEN_SAFE_HEIGHT));
                }
            }
            commands.spawn_bundle(EnemyBundle::new(Vec2 {
                x: x_position as f32,
                y: y_position as f32,
            }));
        }
    }

    pub fn spawn_health(
        mut player_transform: &mut Query<&Transform, (With<Player>, Without<Enemy>)>,

        mut enemy_spawner: &ResMut<Spawner>,
        mut commands: &mut Commands,
    ) {
        let (player_transform) = player_transform.single_mut();

        let mut rng = thread_rng();

        let mut x_position: f32 = 0.;
        let mut y_position: f32 = 0.;

        for new_enemy in 0..5 {
            let location = rng.gen_range(1..=4);
            let playerx = player_transform.translation.x;
            let playery = player_transform.translation.y;
            match location {
                1 => {
                    //top
                    x_position = rng.gen_range((playerx - 1920.)..(playerx + SCREEN_SAFE_WIDTH));
                    y_position = rng.gen_range(
                        (player_transform.translation.y - 1150. - 100.)
                            ..(player_transform.translation.y - 1080.),
                    );
                }
                2 => {
                    //right
                    x_position =
                        rng.gen_range((playerx + 1080.)..(playerx + SCREEN_SAFE_WIDTH + 100.));
                    y_position = rng
                        .gen_range((playery - SCREEN_SAFE_HEIGHT)..(playery + SCREEN_SAFE_HEIGHT));
                }
                3 => {
                    //bottom
                    x_position =
                        rng.gen_range((playerx - SCREEN_SAFE_WIDTH)..(playerx + SCREEN_SAFE_WIDTH));
                    y_position = rng.gen_range(
                        (player_transform.translation.y + 1080.)
                            ..(player_transform.translation.y + 1150. + 100.),
                    );
                }
                _ => {
                    //left and everything else
                    x_position =
                        rng.gen_range((playerx - SCREEN_SAFE_WIDTH + 100.)..(playerx - 1920.));
                    y_position =
                        rng.gen_range((playery - 1080. - 100.)..(playery + SCREEN_SAFE_HEIGHT));
                }
            }
            commands.spawn_bundle(HealthBundle::new(Vec2 {
                x: x_position as f32,
                y: y_position as f32,
            }));
        }
    }

    pub fn spawn_powerup(
        mut player_transform: &mut Query<&Transform, (With<Player>, Without<Enemy>)>,

        mut enemy_spawner: &ResMut<Spawner>,
        mut commands: &mut Commands,
    ) {
        let (player_transform) = player_transform.single_mut();

        let mut rng = thread_rng();

        let mut x_position: f32 = 0.;
        let mut y_position: f32 = 0.;
        
        let chance = rng.gen_range(0..5);
        
        if chance == 1 {
            let location = rng.gen_range(1..=4);
            let playerx = player_transform.translation.x;
            let playery = player_transform.translation.y;
            match location {
                1 => {
                    //top
                    x_position = rng.gen_range((playerx - 1920.)..(playerx + SCREEN_SAFE_WIDTH));
                    y_position = rng.gen_range(
                        (player_transform.translation.y - 1150. - 100.)
                            ..(player_transform.translation.y - 1080.),
                    );
                }
                2 => {
                    //right
                    x_position =
                        rng.gen_range((playerx + 1080.)..(playerx + SCREEN_SAFE_WIDTH + 100.));
                    y_position = rng
                        .gen_range((playery - SCREEN_SAFE_HEIGHT)..(playery + SCREEN_SAFE_HEIGHT));
                }
                3 => {
                    //bottom
                    x_position =
                        rng.gen_range((playerx - SCREEN_SAFE_WIDTH)..(playerx + SCREEN_SAFE_WIDTH));
                    y_position = rng.gen_range(
                        (player_transform.translation.y + 1080.)
                            ..(player_transform.translation.y + 1150. + 100.),
                    );
                }
                _ => {
                    //left and everything else
                    x_position =
                        rng.gen_range((playerx - SCREEN_SAFE_WIDTH + 100.)..(playerx - 1920.));
                    y_position =
                        rng.gen_range((playery - 1080. - 100.)..(playery + SCREEN_SAFE_HEIGHT));
                }
            }
            commands.spawn_bundle(PowerupBundle::new(Vec2 {
                x: x_position as f32,
                y: y_position as f32,
            }));
        }
    }
}

impl FromWorld for Spawner {
    fn from_world(world: &mut World) -> Self {
        Spawner {
            amount_to_spawn_next_tick: 35,
        }
    }
}
