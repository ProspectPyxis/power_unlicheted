use crate::common::{DamagesPlayer, Enemy, EnemyAI, GamePhysicsLayer, GameSprites, Health, Player};
use bevy::prelude::*;
use heron::prelude::*;
use itertools::Itertools;
use std::f32::consts::{FRAC_PI_2, PI};

pub fn spawn_enemy_wave(mut commands: Commands, sprites: Res<GameSprites>) {
    let wave_width = alea::u32_in_range(4, 7);
    let wave_height = alea::u32_in_range(3, 5);
    let start_x = alea::f32_in_range(-600.0, 600.0);

    for (x, y) in (0..wave_width).cartesian_product(0..wave_height) {
        let spawn_x = start_x + ((x as f32 - x as f32 / 2.0) * 30.0);
        let spawn_y = -550.0 + ((y as f32 - y as f32 / 2.0) * 50.0);
        let pos = Vec3::new(spawn_x, spawn_y, 1.0);
        commands
            .spawn_bundle(SpriteBundle {
                texture: sprites.soldier.clone(),
                transform: Transform {
                    translation: pos,
                    scale: Vec3::new(1.5, 1.5, 0.0),
                    ..Default::default()
                },
                ..Default::default()
            })
            .insert(Enemy {
                ai: EnemyAI::ChasesPlayer { speed: 120.0 },
            })
            .insert(RigidBody::Dynamic)
            .insert(CollisionShape::Sphere { radius: 10.0 })
            .insert(Velocity::from_linear(Vec3::ZERO))
            .insert(RotationConstraints::lock())
            .insert(
                CollisionLayers::none()
                    .with_group(GamePhysicsLayer::Enemy)
                    .with_masks(&[
                        GamePhysicsLayer::Projectile,
                        GamePhysicsLayer::Player,
                        GamePhysicsLayer::Enemy,
                    ]),
            )
            .insert(DamagesPlayer {
                damage: 1.0,
                tick: Timer::from_seconds(1.0, true),
                is_damaging: false,
            });
    }
}

pub fn update_enemy(
    mut q_enemies: Query<(&Enemy, &Transform, &mut Velocity, &mut Sprite), Without<Player>>,
    q_player: Query<&Transform, With<Player>>,
) {
    let player = q_player.single();
    for (enemy, transform, mut velocity, mut sprite) in q_enemies.iter_mut() {
        let current_pos = transform.translation;
        match enemy.ai {
            EnemyAI::ChasesPlayer { speed } => {
                velocity.linear = (player.translation - current_pos).normalize() * speed;
                if ((velocity.linear.angle_between(Vec3::X) + FRAC_PI_2) / PI)
                    .rem_euclid(2.0)
                    .floor()
                    == 1.0
                {
                    sprite.flip_x = true;
                } else {
                    sprite.flip_x = false;
                }
            }
        }
    }
}

pub fn check_enemy_player_collision(
    mut collision_events: EventReader<CollisionEvent>,
    mut q_enemies: Query<&mut DamagesPlayer, With<Enemy>>,
) {
    fn is_player(layers: CollisionLayers) -> bool {
        layers.contains_group(GamePhysicsLayer::Player)
            && !layers.contains_group(GamePhysicsLayer::Enemy)
    }
    fn is_enemy(layers: CollisionLayers) -> bool {
        layers.contains_group(GamePhysicsLayer::Enemy)
            && !layers.contains_group(GamePhysicsLayer::Player)
    }

    for (evt, e_enemy) in collision_events.iter().filter_map(|event| {
        let (entity_1, entity_2) = event.rigid_body_entities();
        let (layers_1, layers_2) = event.collision_layers();
        if is_enemy(layers_1) && is_player(layers_2) {
            Some((event, entity_1))
        } else if is_enemy(layers_2) && is_player(layers_1) {
            Some((event, entity_2))
        } else {
            None
        }
    }) {
        if let Ok(mut enemy) = q_enemies.get_mut(e_enemy) {
            match evt {
                CollisionEvent::Started(_d1, _d2) => {
                    enemy.is_damaging = true;
                }
                CollisionEvent::Stopped(_d1, _d2) => {
                    enemy.is_damaging = false;
                }
            }
        }
    }
}

pub fn enemy_damage_player(
    mut q_enemies: Query<&mut DamagesPlayer, With<Enemy>>,
    mut q_player: Query<&mut Health, With<Player>>,
    time: Res<Time>,
) {
    let mut player = q_player.single_mut();
    for mut enemy in q_enemies.iter_mut().filter(|e| e.is_damaging) {
        if enemy.tick.tick(time.delta()).just_finished() {
            player.current -= enemy.damage;
        }
    }
}
