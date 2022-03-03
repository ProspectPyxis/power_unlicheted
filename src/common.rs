use std::time::Duration;

use bevy::prelude::*;
use bevy_asset_loader::AssetCollection;
use bevy_kira_audio::AudioSource;
use heron::prelude::*;

pub const SCREEN_WIDTH: f32 = 960.0;
pub const SCREEN_HEIGHT: f32 = 720.0;

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub enum GameState {
    AssetLoading,
    Opening,
    MoraleStatus,
    ActiveGame,
    GameOver,
    Credits,
}

#[derive(AssetCollection)]
pub struct GameSprites {
    #[asset(path = "sprites/lich.png")]
    pub lich: Handle<Image>,
    #[asset(path = "sprites/fireball.png")]
    pub fireball: Handle<Image>,
    #[asset(path = "sprites/lightning_bolt.png")]
    pub lightning_bolt: Handle<Image>,
    #[asset(texture_atlas(tile_size_x = 64.0, tile_size_y = 64.0, columns = 1, rows = 4))]
    #[asset(path = "sprites/lightning_explosion.png")]
    pub lightning_explosion: Handle<TextureAtlas>,
    #[asset(path = "sprites/soldier.png")]
    pub soldier: Handle<Image>,
    #[asset(path = "sprites/grass.png")]
    pub grass: Handle<Image>,
    #[asset(path = "sprites/bevy.png")]
    pub bevy: Handle<Image>,
}

#[derive(AssetCollection)]
pub struct GameFonts {
    #[asset(path = "fonts/m5x7.ttf")]
    pub main: Handle<Font>,
}

#[derive(AssetCollection)]
pub struct GameAudio {
    #[asset(path = "sounds/click.wav")]
    pub click: Handle<AudioSource>,
    #[asset(path = "sounds/fireball.wav")]
    pub fireball: Handle<AudioSource>,
    #[asset(path = "sounds/lightning_explosion.wav")]
    pub lightning_explosion: Handle<AudioSource>,
    #[asset(path = "sounds/enemy_kill.wav")]
    pub enemy_kill: Handle<AudioSource>,
    #[asset(path = "sounds/player_hurt.wav")]
    pub player_hurt: Handle<AudioSource>,
}

pub struct DamagePlayerEvent(pub f32);

pub enum DayEndReason {
    Timeout,
    PlayerDeath,
}

pub struct EndDayEvent {
    pub reason: DayEndReason,
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
    pub wave_core: Option<Entity>,
}

#[derive(Component)]
pub struct WaveCore {
    pub remaining: u32,
}

#[derive(Component)]
pub enum Ui {
    Core,
    HealthBarMain,
    TimeLeftDisplay,
    NarrationText,
}

#[derive(Component)]
pub struct InGameUI;

#[derive(Component)]
pub struct OpeningNarration(pub usize);

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
pub struct Animated {
    pub frames: usize,
    pub timer: Timer,
}

#[derive(Component)]
pub enum GameOverButton {
    Restart,
    Credits,
    Exit,
}

pub enum PlayerSpell {
    Fireball,
    LightningStrike,
}

pub struct SpellCooldowns {
    pub fireball: Timer,
    pub lightning_strike: Timer,
}

impl Default for SpellCooldowns {
    fn default() -> Self {
        Self {
            fireball: Timer::from_seconds(0.2, false),
            lightning_strike: Timer::from_seconds(0.7, false),
        }
    }
}

impl SpellCooldowns {
    pub fn tick_all(&mut self, delta: Duration) {
        self.fireball.tick(delta);
        self.lightning_strike.tick(delta);
    }
}

#[derive(Component)]
pub struct PlayerSpellData {
    pub selected: PlayerSpell,
    pub cooldowns: SpellCooldowns,
}

#[derive(Component)]
pub struct LightningStrikeBolt {
    pub end_y: f32,
}

// Resources

#[derive(Component)]
pub struct EnemyMorale(pub f32);

#[derive(Component)]
pub struct WaveManager {
    pub active_waves: u32,
    pub max_waves: u32,
    pub wave_timer: Timer,
}

#[derive(Component)]
pub struct CurrentDay(pub u32);

#[derive(Component)]
pub struct CurrentTime(pub Timer);

impl CurrentTime {
    pub fn time_remaining(&self) -> Duration {
        self.0.duration().saturating_sub(self.0.elapsed())
    }
}

#[derive(PhysicsLayer)]
pub enum GamePhysicsLayer {
    Player,
    PlayerAttack,
    Enemy,
}

#[derive(SystemLabel, Debug, Hash, PartialEq, Eq, Clone)]
pub enum Label {
    Movement,
    CollisionCheck,
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

/// Animates all sprites with attached animation
pub fn animate_sprites(
    time: Res<Time>,
    mut q_anim: Query<(&mut Animated, &mut TextureAtlasSprite)>,
) {
    for (mut anim, mut sprite) in q_anim.iter_mut() {
        if anim.timer.tick(time.delta()).just_finished() {
            sprite.index = (sprite.index + 1) % anim.frames;
        }
    }
}
