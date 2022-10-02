use bevy::prelude::*;
use bevy_rapier2d::prelude::*;


pub struct PlayerInput{
    pub is_holding_forward: bool,
    pub is_holding_turn: bool,
}

impl FromWorld for PlayerInput {
    fn from_world(world: &mut World) -> Self {
        PlayerInput {
            is_holding_forward: false,
            is_holding_turn: false
        }
    }
}

pub struct PlayerStats{
    pub speed_per_frame: f32,
    pub max_speed: f32,
    pub current_speed: Vec3,
}

impl PlayerStats{
    pub fn add_forward_speed(&mut self, added_speed:f32){
        self.current_speed.y += added_speed;
        if self.current_speed.y > self.max_speed{
            self.current_speed.y = self.max_speed;
        }
    }
}

impl FromWorld for PlayerStats {
    fn from_world(world: &mut World) -> Self {
        PlayerStats {
            speed_per_frame: 10.,
            max_speed: 600.,
            current_speed: Vec3{
                x: 0.0,
                y: 0.0,
                z: 0.0
            }
        }
    }
}

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct PlayerCam;

#[derive(Bundle)]
pub struct PlayerBundle {
    #[bundle]
    pub(crate) sprite_bundle: SpriteBundle,
    rigidbody: RigidBody,
    damping:Damping,
    collider: Collider,
    velocity: Velocity,
    ccd: Ccd,
    player:Player,
    gravity_scale:GravityScale,
    locked_axes:LockedAxes,
}

impl PlayerBundle{
    pub(crate) fn new() -> PlayerBundle{
        PlayerBundle{
            sprite_bundle: SpriteBundle{
                sprite: Default::default(),
                transform: Transform {
                    translation: Vec3 {
                        x: 0.0,
                        y: 0.0,
                        z: 100.0
                    },
                    rotation: Default::default(),
                    scale: Vec3 { x: 16.0, y: 16.0, z: 1.0 }
                },
                ..default()
            },
            rigidbody: RigidBody::Dynamic,
            damping: Damping { linear_damping: 4., angular_damping: 4. },
            collider: Collider::cuboid(0.5, 0.5),
            velocity: Velocity { linvel: Vec2::new(0., 0.), angvel: 0. },
            ccd: Ccd { enabled: false },
            player: Player,
            gravity_scale: GravityScale(0.),
            locked_axes: LockedAxes::ROTATION_LOCKED_Z,
        }
    }
    
}

#[derive(Component)]
pub struct Enemy;

#[derive(Bundle)]
pub struct EnemyBundle {
    #[bundle]
    pub(crate) sprite_bundle: SpriteBundle,
    enemy: Enemy,
    rigidbody: RigidBody,
    damping:Damping,
    collider: Collider,
    velocity: Velocity,
    ccd: Ccd,
    gravity_scale:GravityScale,
}

impl EnemyBundle{
    pub(crate) fn new() -> EnemyBundle{
        EnemyBundle{
            sprite_bundle: SpriteBundle{
                sprite: Default::default(),
                transform: Transform {
                    translation: Vec3 {
                        x: 0.0,
                        y: 0.0,
                        z: 50.0
                    },
                    rotation: Default::default(), 
                    scale: Vec3 { x: 32.0, y: 32.0, z: 1.0 }
                },
                ..default()
            },
            enemy: Enemy,
            rigidbody: RigidBody::Dynamic,
            damping: Damping { linear_damping: 7., angular_damping: 7. },
            collider: Collider::cuboid(0.5, 0.5),
            velocity: Velocity { linvel: Vec2::new(0., 0.), angvel: 0. },
            ccd: Ccd { enabled: false },
            gravity_scale: GravityScale(0.)
        }
    }

}
