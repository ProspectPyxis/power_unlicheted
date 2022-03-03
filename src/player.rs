use crate::common::{
    get_cursor_position, DamagePlayerEvent, DamagesEnemy, DayEndReason, DespawnTimer, EndDayEvent,
    EnemyMorale, GamePhysicsLayer, GameSprites, GameState, Health, MainCamera, Player, Projectile,
    RegeneratesHealth, Ui, Vec3Utils,
};
use bevy::{input::keyboard::KeyCode, prelude::*};
use heron::prelude::*;
use std::f32::consts::PI;

pub fn spawn_player(mut commands: Commands, sprites: Res<GameSprites>) {
    let player = commands
        .spawn_bundle(SpriteBundle {
            texture: sprites.lich.clone(),
            transform: Transform {
                translation: Vec3::new(0.0, 0.0, 0.5),
                scale: Vec3::new(4.0, 4.0, 0.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Player)
        .insert(RigidBody::KinematicPositionBased)
        .insert(CollisionShape::Sphere { radius: 24.0 })
        .insert(CollisionLayers::new(
            GamePhysicsLayer::Player,
            GamePhysicsLayer::Enemy,
        ))
        .insert(Health::full(500.0))
        .insert(RegeneratesHealth {
            regen: 1.0,
            tick: Timer::from_seconds(2.0, true),
            is_regenerating: true,
        })
        .id();

    let hp_bar_main = spawn_health_bar(&mut commands);
    commands.entity(player).add_child(hp_bar_main);
}

fn spawn_health_bar(commands: &mut Commands) -> Entity {
    let hp_bar_main = commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: Color::GREEN,
                custom_size: Some(Vec2::new(100.0, 12.0)),
                ..Default::default()
            },
            transform: Transform {
                translation: Vec3::new(0.0, -15.0, 10.0),
                scale: Vec3::new(0.25, 0.25, 0.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Ui::HealthBarMain)
        .id();

    hp_bar_main
}

pub fn player_move(
    mut q: Query<(&mut Transform, &mut Sprite), With<Player>>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    if let Some((mut transform, mut sprite)) = q.iter_mut().next() {
        if keyboard_input.pressed(KeyCode::A) {
            transform.translation.x -= 4.0;
            sprite.flip_x = true;
        }
        if keyboard_input.pressed(KeyCode::D) {
            transform.translation.x += 4.0;
            sprite.flip_x = false;
        }
        if keyboard_input.pressed(KeyCode::W) {
            transform.translation.y += 4.0;
        }
        if keyboard_input.pressed(KeyCode::S) {
            transform.translation.y -= 4.0;
        }
    }
}

pub fn player_shoot(
    mut commands: Commands,
    sprites: Res<GameSprites>,
    q_player: Query<&Transform, With<Player>>,
    wnds: Res<Windows>,
    q_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    mouse_input: Res<Input<MouseButton>>,
) {
    if mouse_input.just_pressed(MouseButton::Left) {
        if let Some(cursor_pos) = get_cursor_position(wnds, q_camera) {
            if let Some(player) = q_player.iter().next() {
                for i in -1..=1 {
                    commands
                        .spawn_bundle(SpriteBundle {
                            texture: sprites.fireball.clone(),
                            transform: Transform {
                                translation: player.translation,
                                scale: Vec3::new(2.0, 2.0, 0.0),
                                ..Default::default()
                            },
                            ..Default::default()
                        })
                        .insert(Projectile)
                        .insert(RigidBody::KinematicVelocityBased)
                        .insert(Velocity::from_linear(
                            (cursor_pos - player.translation.truncate())
                                .extend(0.0)
                                .normalize()
                                .rotate_2d(PI * i as f32 / 16.0)
                                * 360.0,
                        ))
                        .insert(CollisionShape::Sphere { radius: 8.0 })
                        .insert(CollisionLayers::new(
                            GamePhysicsLayer::PlayerAttack,
                            GamePhysicsLayer::Enemy,
                        ))
                        .insert(DespawnTimer(Timer::from_seconds(1.5, false)))
                        .insert(DamagesEnemy { damage: 2.0 })
                        .with_children(|parent| {
                            parent
                                .spawn_bundle(SpriteBundle::default())
                                .insert(RigidBody::Sensor)
                                .insert(CollisionShape::Sphere { radius: 16.0 })
                                .insert(CollisionLayers::new(
                                    GamePhysicsLayer::PlayerAttack,
                                    GamePhysicsLayer::Enemy,
                                ))
                                .insert(DamagesEnemy { damage: 1.0 });
                        });
                }
            }
        }
    }
}

pub fn register_player_damage(
    mut q_player: Query<&mut Health, With<Player>>,
    mut damages: EventReader<DamagePlayerEvent>,
    mut morale: ResMut<EnemyMorale>,
    mut state: ResMut<State<GameState>>,
    mut day_end_writer: EventWriter<EndDayEvent>,
) {
    if let Some(mut player) = q_player.iter_mut().next() {
        for damage in damages.iter() {
            player.current -= damage.0;
            morale.0 += damage.0 / 10.0;
        }
        if player.current <= 0.0 {
            morale.0 += 25.0;
            day_end_writer.send(EndDayEvent {
                reason: DayEndReason::PlayerDeath,
            });
            state.set(GameState::MoraleStatus).unwrap();
        }
    }
}

pub fn update_health_display(
    mut q_health_bar: Query<(&Parent, &mut Sprite, &Ui)>,
    q_player: Query<&Health, With<Player>>,
) {
    for (parent, mut sprite) in q_health_bar.iter_mut().filter_map(|(p, s, i)| match i {
        Ui::HealthBarMain => Some((p, s)),
        _ => None,
    }) {
        if let Ok(health) = q_player.get(parent.0) {
            sprite.custom_size = Some(Vec2::new(
                (health.current / health.maximum).max(0.0) * 100.0,
                12.0,
            ));
            if health.current <= health.maximum * 0.25 {
                sprite.color = Color::RED;
            } else {
                sprite.color = Color::GREEN;
            }
        }
    }
}
