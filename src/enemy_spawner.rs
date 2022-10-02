use crate::GameTickInfo;
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use crate::definitions_units::*;



pub struct EnemySpawner {
    amount_to_spawn_next_tick: u32,
}

impl EnemySpawner {
    pub fn spawn_next_wave(
        game_tick_time: &ResMut<GameTickInfo>,
        mut enemy_spawner: &ResMut<EnemySpawner>,
        mut commands: &mut Commands,
    ) {
        for new_enemy in 0..enemy_spawner.amount_to_spawn_next_tick {
            commands.spawn_bundle(EnemyBundle::new());
        }
        
    } 
}

impl FromWorld for EnemySpawner {
    fn from_world(world: &mut World) -> Self {
        EnemySpawner {
            amount_to_spawn_next_tick: 50,
        }
    }
}
