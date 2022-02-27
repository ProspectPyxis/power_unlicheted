use bevy::prelude::*;
use bevy_asset_loader::AssetCollection;

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
pub struct AngledMovement {
    pub speed: f32,
    pub angle: f32,
}

#[derive(Component)]
pub struct DespawnTimer(pub Timer);

// Functions

/// Moves the transform by a certain speed, at the angle (in radians).
pub fn move_in_direction(mut transform: Mut<Transform>, speed: f32, angle: f32) {
    transform.translation.x += speed * angle.cos();
    transform.translation.y += speed * angle.sin();
}

/// Get the angle from point v1 to point v2.
pub fn angle_between_points(v1: Vec2, v2: Vec2) -> f32 {
    (v2.y - v1.y).atan2(v2.x - v1.x)
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

/// Move all entities with the AngledMovement component
pub fn apply_angled_movement(mut q_angled: Query<(&mut Transform, &AngledMovement)>) {
    for (transform, angle) in q_angled.iter_mut() {
        move_in_direction(transform, angle.speed, angle.angle);
    }
}

/// Ticks all entities that can despawn, and despawn them if their time is up
pub fn check_despawn(
    mut commands: Commands,
    time: Res<Time>,
    mut q_despawn: Query<(Entity, &mut DespawnTimer)>,
) {
    for (ent, mut timer) in q_despawn.iter_mut() {
        if timer.0.tick(time.delta()).just_finished() {
            commands.entity(ent).despawn();
        }
    }
}
