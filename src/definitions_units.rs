use crate::HealthGone;
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

pub const PLAYER_COLOR: Color = Color::Rgba {
    red: 0.0,
    green: 0.4,
    blue: 1.0,
    alpha: 1.0,
};

const HEALTH_COLOR: Color = Color::Rgba {
    red: 0.0,
    green: 0.4,
    blue: 1.0,
    alpha: 1.0,
};

pub const POWERUP_COLOR: Color = Color::Rgba {
    red: 1.0,
    green: 0.6,
    blue: 0.0,
    alpha: 1.0,
};
pub const ENEMY_COLOR: Color = Color::Rgba {
    red: 0.8,
    green: 0.2,
    blue: 0.2,
    alpha: 1.0,
};

pub struct PlayerStats {
    pub speed_per_frame: f32,
    pub max_speed: f32,
    pub current_speed: Vec3,
    pub health: i32,
    pub kill_mode: bool,
    pub time_left_in_kill_mode: f32,
}

impl PlayerStats {
    pub fn add_forward_speed(&mut self, added_speed: f32) {
        self.current_speed.y += added_speed;
        if self.current_speed.y > self.max_speed {
            self.current_speed.y = self.max_speed;
        }
    }

    pub fn health_damage(
        &mut self,
        amount_to_remove: i32,
        mut health_event: &mut EventWriter<HealthGone>,
    ) {
        self.health -= amount_to_remove;
        if self.health <= 0 {
            health_event.send(HealthGone {})
        }
    }

    pub fn health_heal(&mut self, amount_to_add: i32) {
        self.health += amount_to_add;
    }

    pub fn health_heal_up_to_ten(&mut self) {
        self.health = 10;
    }

    pub fn powerup_mode(&mut self, mut player_sprite: &mut Mut<Sprite>) {
        player_sprite.color = POWERUP_COLOR;
        self.kill_mode = true;
        self.time_left_in_kill_mode += 3.;
        if self.time_left_in_kill_mode > 5.{
            self.time_left_in_kill_mode = 5.;
        }
    }
    pub fn powerup_time_decrease(&mut self, mut player_sprite: &mut Mut<Sprite>) {
        if self.time_left_in_kill_mode > 0.{
            self.time_left_in_kill_mode -= 1.;

        }
        info!(self.time_left_in_kill_mode);
        if self.time_left_in_kill_mode <= 0. {
            player_sprite.color = PLAYER_COLOR;
            self.kill_mode = false;
        }
    }
}

impl FromWorld for PlayerStats {
    fn from_world(world: &mut World) -> Self {
        PlayerStats {
            speed_per_frame: 10.,
            max_speed: 600.,
            current_speed: Vec3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            health: 10,
            kill_mode: false,
            time_left_in_kill_mode: 0.0,
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
    damping: Damping,
    collider: Collider,
    velocity: Velocity,
    ccd: Ccd,
    player: Player,
    gravity_scale: GravityScale,
    locked_axes: LockedAxes,
    active_events: ActiveEvents,
    colliding_entities: CollidingEntities,
}

impl PlayerBundle {
    pub(crate) fn new() -> PlayerBundle {
        PlayerBundle {
            sprite_bundle: SpriteBundle {
                sprite: Sprite {
                    color: PLAYER_COLOR,
                    ..default()
                },
                transform: Transform {
                    translation: Vec3 {
                        x: 0.0,
                        y: 0.0,
                        z: 100.0,
                    },
                    rotation: Default::default(),
                    scale: Vec3 {
                        x: 16.0,
                        y: 16.0,
                        z: 1.0,
                    },
                },
                ..default()
            },
            rigidbody: RigidBody::Dynamic,
            damping: Damping {
                linear_damping: 4.,
                angular_damping: 4.,
            },
            collider: Collider::cuboid(0.5, 0.5),
            velocity: Velocity {
                linvel: Vec2::new(0., 0.),
                angvel: 0.,
            },
            ccd: Ccd { enabled: false },
            player: Player,
            gravity_scale: GravityScale(0.),
            locked_axes: LockedAxes::ROTATION_LOCKED_Z,
            active_events: ActiveEvents::COLLISION_EVENTS,
            colliding_entities: Default::default(),
        }
    }
}

#[derive(Component)]
pub struct Powerup;

#[derive(Bundle)]
pub struct PowerupBundle {
    #[bundle]
    pub(crate) sprite_bundle: SpriteBundle,
    rigidbody: RigidBody,
    collider: Collider,
    sensor: Sensor,
    gravity_scale: GravityScale,
    powerup: Powerup,
}

impl PowerupBundle {
    pub(crate) fn new(spawn_position: Vec2) -> PowerupBundle {
        PowerupBundle {
            sprite_bundle: SpriteBundle {
                sprite: Sprite {
                    color: POWERUP_COLOR,
                    ..default()
                },
                transform: Transform {
                    translation: spawn_position.extend(50.),
                    rotation: Default::default(),
                    scale: Vec3 {
                        x: 48.0,
                        y: 48.0,
                        z: 1.0,
                    },
                },
                ..default()
            },
            rigidbody: RigidBody::Dynamic,
            collider: Collider::cuboid(0.5, 0.5),
            sensor: Sensor,
            gravity_scale: GravityScale(0.),
            powerup: Powerup,
        }
    }
}

#[derive(Component)]
pub struct Health;

#[derive(Bundle)]
pub struct HealthBundle {
    #[bundle]
    pub(crate) sprite_bundle: SpriteBundle,
    rigidbody: RigidBody,
    collider: Collider,
    sensor: Sensor,
    gravity_scale: GravityScale,
    health: Health,
}

impl HealthBundle {
    pub(crate) fn new(spawn_position: Vec2) -> HealthBundle {
        HealthBundle {
            sprite_bundle: SpriteBundle {
                sprite: Sprite {
                    color: HEALTH_COLOR,
                    ..default()
                },
                transform: Transform {
                    translation: spawn_position.extend(50.),
                    rotation: Default::default(),
                    scale: Vec3 {
                        x: 48.0,
                        y: 48.0,
                        z: 1.0,
                    },
                },
                ..default()
            },
            rigidbody: RigidBody::Dynamic,
            collider: Collider::cuboid(0.5, 0.5),
            sensor: Sensor,
            gravity_scale: GravityScale(0.),
            health: Health,
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
    damping: Damping,
    collider: Collider,
    velocity: Velocity,
    ccd: Ccd,
    gravity_scale: GravityScale,
    //restitution: Restitution,
}

impl EnemyBundle {
    pub(crate) fn new(spawn_position: Vec2) -> EnemyBundle {
        EnemyBundle {
            sprite_bundle: SpriteBundle {
                sprite: Sprite {
                    color: ENEMY_COLOR,
                    ..default()
                },
                transform: Transform {
                    translation: spawn_position.extend(50.),
                    rotation: Default::default(),
                    scale: Vec3 {
                        x: 32.0,
                        y: 32.0,
                        z: 1.0,
                    },
                },
                ..default()
            },
            enemy: Enemy,
            rigidbody: RigidBody::Dynamic,
            damping: Damping {
                linear_damping: 7.,
                angular_damping: 7.,
            },
            collider: Collider::cuboid(0.5, 0.5),
            velocity: Velocity {
                linvel: Vec2::new(0., 0.),
                angvel: 0.,
            },
            ccd: Ccd { enabled: false },
            gravity_scale: GravityScale(0.),
            //restitution: Restitution { coefficient: 5., combine_rule: CoefficientCombineRule::Max }
        }
    }
}
