use bevy::prelude::*;
use bevy_asset_loader::AssetCollection;
use heron::prelude::*;

#[derive(AssetCollection)]
pub struct GameSprites {
    #[asset(path = "sprites/lich.png")]
    pub lich: Handle<Image>,
    #[asset(path = "sprites/fireball.png")]
    pub fireball: Handle<Image>,
    #[asset(path = "sprites/soldier.png")]
    pub soldier: Handle<Image>,
}

// Components

#[derive(Component)]
pub struct MainCamera;

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct Projectile;

pub enum EnemyAI {
    ChasesPlayer { speed: f32 },
}

#[derive(Component)]
pub struct Enemy {
    pub ai: EnemyAI,
}

#[derive(Component)]
pub enum Ui {
    HealthBarMain,
    MoraleDisplay,
    AbilitySelect,
}

#[derive(Component)]
pub struct Health {
    pub current: f32,
    pub maximum: f32,
}

impl Health {
    pub fn full(hp: f32) -> Self {
        Self {
            current: hp,
            maximum: hp,
        }
    }
}

#[derive(Component)]
pub struct RegeneratesHealth {
    pub regen: f32,
    pub tick: Timer,
    pub is_regenerating: bool,
}

#[derive(Component)]
pub struct DamagesPlayer {
    pub damage: f32,
    pub tick: Timer,
    pub is_damaging: bool,
}

#[derive(Component)]
pub struct DamagesEnemy {
    pub damage: f32,
}

#[derive(Component)]
pub struct DespawnTimer(pub Timer);

#[derive(Component)]
pub struct EnemyMorale(pub f32);

#[derive(PhysicsLayer)]
pub enum GamePhysicsLayer {
    Player,
    PlayerAttack,
    Enemy,
}

#[derive(SystemLabel, Debug, Hash, PartialEq, Eq, Clone)]
pub enum Label {
    Movement,
    HealthUpdate,
    Despawn,
    UpdateSprites,
}

// Functions

pub trait Vec3Utils {
    fn rotate_2d(self, angle: f32) -> Self;

    fn line_overlaps_circle(self, velocity: Vec3, ahead_len: f32, c_pos: Vec2, c_r: f32) -> bool;
}

impl Vec3Utils for Vec3 {
    /// Rotates a vector by a given amount of radians.
    fn rotate_2d(self, angle: f32) -> Self {
        Vec3::new(
            self.x * angle.cos() - self.y * angle.sin(),
            self.x * angle.sin() + self.y * angle.cos(),
            self.z,
        )
    }

    fn line_overlaps_circle(self, velocity: Vec3, ahead_len: f32, c_pos: Vec2, c_r: f32) -> bool {
        for i in 0..3 {
            if (self + velocity.normalize() * ahead_len * i as f32 / 2.0)
                .distance(c_pos.extend(0.0))
                <= c_r
            {
                return true;
            }
        }
        false
    }
}

/// Gets the position of the cursor.
/// Taken from the bevy cheatbook: https://bevy-cheatbook.github.io/cookbook/cursor2world.html
pub fn get_cursor_position(
    wnds: Res<Windows>,
    q_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
) -> Option<Vec2> {
    let (camera, camera_transform) = q_camera.single();
    let wnd = wnds.get(camera.window).unwrap();

    if let Some(screen_pos) = wnd.cursor_position() {
        let window_size = Vec2::new(wnd.width() as f32, wnd.height() as f32);
        let ndc = (screen_pos / window_size) * 2.0 - Vec2::ONE;
        let ndc_to_world = camera_transform.compute_matrix() * camera.projection_matrix.inverse();
        let world_pos = ndc_to_world.project_point3(ndc.extend(-1.0));
        Some(world_pos.truncate())
    } else {
        None
    }
}

// Systems

/// Ticks all entities that can despawn, and despawn them if their time is up
pub fn check_despawn(
    mut commands: Commands,
    time: Res<Time>,
    mut q_despawn: Query<(Entity, &mut DespawnTimer)>,
) {
    for (ent, mut timer) in q_despawn.iter_mut() {
        if timer.0.tick(time.delta()).just_finished() {
            commands.entity(ent).despawn_recursive();
        }
    }
}

/// Regenerates health for all entities that can regenerate health.
pub fn regen_health(mut q_regen: Query<(&mut Health, &mut RegeneratesHealth)>, time: Res<Time>) {
    for (mut health, mut regen) in q_regen.iter_mut().filter(|(_, e)| e.is_regenerating) {
        if health.current < health.maximum {
            if regen.tick.tick(time.delta()).just_finished() {
                health.current = (health.current + regen.regen).min(health.maximum);
            }
        } else {
            regen.tick.reset();
        }
    }
}
