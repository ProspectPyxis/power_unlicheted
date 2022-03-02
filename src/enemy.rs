use crate::common::{
    DamagePlayerEvent, DamagesPlayer, Enemy, EnemyAI, EnemyMorale, GamePhysicsLayer, GameSprites,
    Health, Player, Vec3Utils, WaveCore, WaveManager,
};
use bevy::prelude::*;
use heron::prelude::*;
use itertools::Itertools;

pub fn spawn_enemy_wave(
    mut commands: Commands,
    mut wave_manager: ResMut<WaveManager>,
    sprites: Res<GameSprites>,
    time: Res<Time>,
) {
    wave_manager.wave_timer.tick(time.delta());
    if wave_manager.wave_timer.finished() && wave_manager.active_waves < wave_manager.max_waves {
        let wave_to_spawn = alea::u32_less_than(4);
        if (0..2).contains(&wave_to_spawn) {
            spawn_enemy_line_wave(&mut commands, sprites);
        } else if (2..4).contains(&wave_to_spawn) {
            spawn_enemy_square_wave(&mut commands, sprites);
        }
        wave_manager.active_waves += 1;
        wave_manager.wave_timer.reset();
    }
}

pub fn spawn_enemy_square_wave(commands: &mut Commands, sprites: Res<GameSprites>) {
    let wave_width = alea::u32_in_range(4, 7);
    let wave_height = alea::u32_in_range(3, 5);
    let start_x = alea::f32_in_range(-640.0, 640.0);

    let wave_core = commands
        .spawn()
        .insert(WaveCore {
            remaining: wave_width * wave_height,
        })
        .id();

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
                wave_core: Some(wave_core),
            })
            .insert(RigidBody::KinematicVelocityBased)
            .insert(CollisionShape::Sphere { radius: 10.0 })
            .insert(Velocity::from_linear(Vec3::ZERO))
            .insert(RotationConstraints::lock())
            .insert(
                CollisionLayers::none()
                    .with_group(GamePhysicsLayer::Enemy)
                    .with_masks(&[GamePhysicsLayer::PlayerAttack, GamePhysicsLayer::Player]),
            )
            .insert(DamagesPlayer {
                damage: 1.0,
                tick: Timer::from_seconds(1.0, true),
                is_damaging: false,
            })
            .insert(Health::full(3.0));
    }
}

pub fn spawn_enemy_line_wave(commands: &mut Commands, sprites: Res<GameSprites>) {
    let wave_size = alea::u32_in_range(20, 25);

    let wave_core = commands
        .spawn()
        .insert(WaveCore {
            remaining: wave_size,
        })
        .id();

    for i in 0..wave_size {
        let spawn_x = (i as f32 * (1280.0 / wave_size as f32)) - 640.0;
        let pos = Vec3::new(spawn_x, -540.0, 0.0);
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
                wave_core: Some(wave_core),
            })
            .insert(RigidBody::KinematicVelocityBased)
            .insert(CollisionShape::Sphere { radius: 10.0 })
            .insert(Velocity::from_linear(Vec3::ZERO))
            .insert(RotationConstraints::lock())
            .insert(
                CollisionLayers::none()
                    .with_group(GamePhysicsLayer::Enemy)
                    .with_masks(&[GamePhysicsLayer::PlayerAttack, GamePhysicsLayer::Player]),
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
                            (Vec3::Y * (-510.0 - current_pos.y)).normalize() * speed * 3.0;
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
                    if health.current < health.maximum {
                        if transform.translation.x > player.translation.x {
                            sprite.flip_x = false;
                        } else {
                            sprite.flip_x = true;
                        }
                    } else if transform.translation.x > player.translation.x {
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
    time: Res<Time>,
    mut damage_writer: EventWriter<DamagePlayerEvent>,
) {
    for mut enemy in q_enemies.iter_mut().filter(|e| e.is_damaging) {
        if enemy.tick.tick(time.delta()).just_finished() {
            damage_writer.send(DamagePlayerEvent(enemy.damage));
        }
    }
}

pub fn despawn_enemies(
    mut commands: Commands,
    q_enemies: Query<(Entity, &Health, &Transform, &Enemy)>,
    mut q_wave_cores: Query<(Entity, &mut WaveCore)>,
    mut morale: ResMut<EnemyMorale>,
    mut wave_manager: ResMut<WaveManager>,
) {
    for (ent, health, transform, enemy) in q_enemies.iter() {
        let despawned = if health.current <= 0.0 {
            commands.entity(ent).despawn();
            morale.0 -= 0.1;
            true
        } else if health.current < health.maximum && transform.translation.y <= -500.0 {
            commands.entity(ent).despawn();
            morale.0 += 0.25;
            true
        } else {
            false
        };
        if despawned {
            if let Some(e_core) = enemy.wave_core {
                if let Ok((_, mut wave_core)) = q_wave_cores.get_mut(e_core) {
                    wave_core.remaining -= 1;
                }
            }
        }
    }
    for (ent, wave_core) in q_wave_cores.iter_mut() {
        if wave_core.remaining == 0 {
            commands.entity(ent).despawn();
            wave_manager.active_waves -= 1;
        }
    }
}
