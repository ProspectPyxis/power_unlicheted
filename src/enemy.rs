use crate::common::{
    DamagesPlayer, Enemy, EnemyAI, EnemyMorale, GamePhysicsLayer, GameSprites, Health, Player,
    Vec3Utils,
};
use bevy::prelude::*;
use heron::prelude::*;
use itertools::Itertools;

pub fn spawn_enemy_wave(mut commands: Commands, sprites: Res<GameSprites>) {
    let wave_width = alea::u32_in_range(4, 7);
    let wave_height = alea::u32_in_range(3, 5);
    let start_x = alea::f32_in_range(-600.0, 600.0);

    for (x, y) in (0..wave_width).cartesian_product(0..wave_height) {
        let spawn_x = start_x + ((x as f32 - x as f32 / 2.0) * 40.0);
        let spawn_y = -540.0 - ((y as f32 / 2.0) * 60.0);
        let pos = Vec3::new(spawn_x, spawn_y, 0.0);
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
            .insert(RigidBody::KinematicVelocityBased)
            .insert(CollisionShape::Sphere { radius: 10.0 })
            .insert(Velocity::from_linear(Vec3::ZERO))
            .insert(RotationConstraints::lock())
            .insert(
                CollisionLayers::none()
                    .with_group(GamePhysicsLayer::Enemy)
                    .with_masks(&[
                        GamePhysicsLayer::PlayerAttack,
                        GamePhysicsLayer::Player,
                        // GamePhysicsLayer::Enemy,
                    ]),
            )
            .insert(DamagesPlayer {
                damage: 1.0,
                tick: Timer::from_seconds(1.0, true),
                is_damaging: false,
            })
            .insert(Health::full(3.0));
    }
}

pub fn update_enemy(
    mut q_enemies: Query<(Entity, &Enemy, &Transform, &Health, &mut Velocity), Without<Player>>,
    q_enemies_other: Query<(Entity, &Transform), With<Enemy>>,
    q_player: Query<&Transform, With<Player>>,
) {
    if let Some(player) = q_player.iter().next() {
        for (ent, enemy, transform, health, mut velocity) in q_enemies.iter_mut() {
            let current_pos = transform.translation;
            match enemy.ai {
                EnemyAI::ChasesPlayer { speed } => {
                    if health.current < health.maximum {
                        let desired_velocity =
                            (Vec3::Y * (-650.0 - current_pos.y)).normalize() * speed * 3.0;
                        let seek_force = desired_velocity - velocity.linear;

                        let desired_velocity = (current_pos.truncate()
                            - player.translation.truncate())
                        .extend(0.0)
                        .normalize()
                            * speed;
                        let flee_force = desired_velocity - velocity.linear;

                        let steering = seek_force + flee_force;
                        velocity.linear = (velocity.linear + steering).clamp_length_max(speed);
                    } else {
                        // Seeking and arrival
                        let desired_velocity =
                            (player.translation.truncate() - current_pos.truncate()).extend(0.0);
                        let distance = desired_velocity.length();
                        let desired_velocity = if distance < 32.0 {
                            Vec3::ZERO
                        } else {
                            desired_velocity.normalize() * speed
                        };
                        let seek_force = desired_velocity - velocity.linear;

                        // Avoiding others
                        let ahead_len = velocity.linear.length() / speed;
                        let nearest_obstacle = q_enemies_other
                            .iter()
                            .filter(|(e, t)| {
                                *e != ent
                                    && current_pos.line_overlaps_circle(
                                        velocity.linear,
                                        ahead_len,
                                        t.translation.truncate(),
                                        20.0,
                                    )
                            })
                            .fold(None, |accum, (_, t)| {
                                if accum.is_none()
                                    || current_pos.distance(t.translation)
                                        < current_pos.distance(accum.unwrap())
                                {
                                    Some(t.translation)
                                } else {
                                    accum
                                }
                            });
                        let avoidance = if let Some(obstacle) = nearest_obstacle {
                            let ahead = current_pos + velocity.linear.normalize() * ahead_len;
                            Vec3::new(ahead.x - obstacle.x, ahead.y - obstacle.y, 0.0).normalize()
                                * speed
                        } else {
                            Vec3::ZERO
                        };

                        let steering = (seek_force + avoidance).clamp_length_max(6.0);
                        velocity.linear = (velocity.linear + steering).clamp_length_max(speed);
                    }
                }
            }
        }
    }
}

pub fn update_enemy_render(
    mut q_enemies: Query<(&Enemy, &Transform, &Health, &mut Sprite)>,
    q_player: Query<&Transform, With<Player>>,
) {
    if let Some(player) = q_player.iter().next() {
        for (enemy, transform, health, mut sprite) in q_enemies.iter_mut() {
            match enemy.ai {
                EnemyAI::ChasesPlayer { speed: _ } => {
                    if transform.translation.x > player.translation.x {
                        sprite.flip_x = true;
                    } else {
                        sprite.flip_x = false;
                    }
                }
            }
            if health.current < health.maximum {
                sprite.color = Color::rgb(
                    1.0,
                    0.5 + (health.current.max(0.0) / health.maximum) / 2.0,
                    0.5 + (health.current.max(0.0) / health.maximum) / 2.0,
                );
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

pub fn despawn_enemies(
    mut commands: Commands,
    q_enemies: Query<(Entity, &Health, &Transform), With<Enemy>>,
    mut morale: ResMut<EnemyMorale>,
) {
    for (ent, health, transform) in q_enemies.iter() {
        if health.current <= 0.0 {
            commands.entity(ent).despawn();
            morale.0 -= 0.1;
        } else if health.current < health.maximum && transform.translation.y <= -640.0 {
            commands.entity(ent).despawn();
            morale.0 += 0.25;
        }
    }
}
