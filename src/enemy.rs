use crate::common::{
    DamagePlayerEvent, DamagesPlayer, DespawnTimer, Enemy, EnemyAI, EnemyMorale, EnemyProjectile,
    EnemyShoots, GamePhysicsLayer, GameSprites, Health, Player, Vec3Utils, WaveCore, WaveManager,
    SCREEN_HEIGHT, SCREEN_WIDTH,
};
use bevy::prelude::*;
use heron::prelude::*;
use itertools::Itertools;
use std::f32::consts::PI;

pub fn spawn_enemy_wave(
    mut commands: Commands,
    mut wave_manager: ResMut<WaveManager>,
    sprites: Res<GameSprites>,
    time: Res<Time>,
) {
    wave_manager.wave_timer.tick(time.delta());
    if wave_manager.wave_timer.finished() && wave_manager.active_waves < wave_manager.max_waves {
        let wave_to_spawn = alea::u32_less_than(3);
        match wave_to_spawn {
            x if x < 1 => spawn_knight_line_wave(&mut commands, sprites),
            x if x < 2 => spawn_knight_square_wave(&mut commands, sprites),
            _ => spawn_archer_square_wave(&mut commands, sprites),
        }
        wave_manager.active_waves += 1;
        wave_manager.wave_timer.reset();
    }
}

pub fn spawn_knight(commands: &mut Commands, sprite: Handle<Image>, position: Vec3, core: Entity) {
    commands
        .spawn_bundle(SpriteBundle {
            texture: sprite,
            transform: Transform {
                translation: position,
                scale: Vec3::new(1.5, 1.5, 0.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Enemy {
            ai: EnemyAI::ChasesPlayer { speed: 120.0 },
            wave_core: Some(core),
            fear_threshold: 2.5,
        })
        .insert(RigidBody::KinematicVelocityBased)
        .insert(CollisionShape::Sphere { radius: 10.0 })
        .insert(Velocity::from_linear(Vec3::ZERO))
        .insert(
            CollisionLayers::none()
                .with_group(GamePhysicsLayer::Enemy)
                .with_masks(&[GamePhysicsLayer::PlayerAttack, GamePhysicsLayer::Player]),
        )
        .insert(DamagesPlayer {
            damage: 1.0,
            tick: Timer::from_seconds(1.5, true),
            is_damaging: false,
        })
        .insert(Health::full(3.0));
}

pub fn spawn_archer(
    commands: &mut Commands,
    sprite: Handle<Image>,
    position: Vec3,
    core: Entity,
    y_offset: f32,
) {
    commands
        .spawn_bundle(SpriteBundle {
            texture: sprite,
            transform: Transform {
                translation: position,
                scale: Vec3::new(1.5, 1.5, 0.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Enemy {
            ai: EnemyAI::Archer {
                target_y: position.y + y_offset,
            },
            wave_core: Some(core),
            fear_threshold: 1.5,
        })
        .insert(RigidBody::KinematicVelocityBased)
        .insert(CollisionShape::Sphere { radius: 10.0 })
        .insert(Velocity::from_linear(
            Vec3::Y.rotate_2d(alea::f32_in_range(-PI / 128.0, PI / 128.0)) * 180.0,
        ))
        .insert(CollisionLayers::new(
            GamePhysicsLayer::Enemy,
            GamePhysicsLayer::PlayerAttack,
        ))
        .insert(Health::full(3.0))
        .insert(EnemyShoots(Timer::from_seconds(2.0, true)));
}

pub fn spawn_knight_square_wave(commands: &mut Commands, sprites: Res<GameSprites>) {
    let wave_width = alea::u32_in_range(4, 7);
    let wave_height = alea::u32_in_range(3, 5);
    let start_x = alea::f32_in_range(-SCREEN_WIDTH / 2.0, SCREEN_WIDTH / 2.0);

    let wave_core = commands
        .spawn()
        .insert(WaveCore {
            remaining: wave_width * wave_height,
        })
        .id();

    for (x, y) in (0..wave_width).cartesian_product(0..wave_height) {
        let spawn_x = start_x + ((x as f32 - x as f32 / 2.0) * 40.0);
        let spawn_y = (-SCREEN_HEIGHT * 0.6) - ((y as f32 / 2.0) * 60.0);
        let pos = Vec3::new(spawn_x, spawn_y, 0.1);
        spawn_knight(commands, sprites.soldier.clone(), pos, wave_core);
    }
}

pub fn spawn_knight_line_wave(commands: &mut Commands, sprites: Res<GameSprites>) {
    let wave_size = alea::u32_in_range(20, 25);

    let wave_core = commands
        .spawn()
        .insert(WaveCore {
            remaining: wave_size,
        })
        .id();

    for i in 0..wave_size {
        let spawn_x = (i as f32 * (SCREEN_WIDTH / wave_size as f32)) - SCREEN_WIDTH / 2.0;
        let pos = Vec3::new(spawn_x, -SCREEN_HEIGHT * 0.6, 0.1);
        spawn_knight(commands, sprites.soldier.clone(), pos, wave_core);
    }
}

pub fn spawn_archer_square_wave(commands: &mut Commands, sprites: Res<GameSprites>) {
    let wave_width = alea::u32_in_range(3, 4);
    let wave_height = alea::u32_in_range(2, 3);
    let start_x = alea::f32_in_range(-SCREEN_WIDTH * 0.3, SCREEN_WIDTH * 0.3);

    let wave_core = commands
        .spawn()
        .insert(WaveCore {
            remaining: wave_width * wave_height,
        })
        .id();

    for (x, y) in (0..wave_width).cartesian_product(0..wave_height) {
        let spawn_x = start_x + ((x as f32 - x as f32 / 2.0) * 40.0);
        let spawn_y = (-SCREEN_HEIGHT * 0.6) - ((y as f32 / 2.0) * 60.0);
        let pos = Vec3::new(spawn_x, spawn_y, 0.1);
        spawn_archer(
            commands,
            sprites.archer.clone(),
            pos,
            wave_core,
            ((wave_height + 2) * 30) as f32,
        );
    }
}

pub fn update_enemy(
    mut q_enemies: Query<(Entity, &mut Enemy, &Transform, &mut Velocity), Without<Player>>,
    q_enemies_other: Query<(Entity, &Transform), With<Enemy>>,
    q_player: Query<&Transform, With<Player>>,
) {
    if let Some(player) = q_player.iter().next() {
        for (ent, enemy, transform, mut velocity) in q_enemies.iter_mut() {
            let current_pos = transform.translation;
            match enemy.ai {
                EnemyAI::ChasesPlayer { speed } => {
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
                                && current_pos.distance(t.translation) <= 40.0
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
                EnemyAI::Archer { target_y } => {
                    if velocity.linear != Vec3::ZERO && transform.translation.y >= target_y {
                        let sub = velocity.linear.normalize() * 15.0;
                        velocity.linear -= sub;
                        if velocity.linear.y <= 0.0 {
                            velocity.linear = Vec3::ZERO;
                        }
                    }
                }
                EnemyAI::Afraid { speed } => {
                    let desired_velocity = (Vec3::Y * ((-SCREEN_HEIGHT * 0.7) - current_pos.y))
                        .normalize()
                        * speed
                        * 3.0;
                    let seek_force = desired_velocity - velocity.linear;

                    let desired_velocity = (current_pos.truncate() - player.translation.truncate())
                        .extend(0.0)
                        .normalize()
                        * speed;
                    let flee_force = desired_velocity - velocity.linear;

                    let steering = seek_force + flee_force;
                    velocity.linear = (velocity.linear + steering).clamp_length_max(speed);
                }
            }
        }
    }
}

pub fn update_enemy_shoot(
    mut commands: Commands,
    mut q_shoots: Query<(&mut EnemyShoots, &Velocity, &Transform), With<Enemy>>,
    q_player: Query<&Transform, With<Player>>,
    sprites: Res<GameSprites>,
    time: Res<Time>,
) {
    if let Some(p_transform) = q_player.iter().next() {
        for (mut timer, vel, e_transform) in q_shoots.iter_mut() {
            if vel.linear == Vec3::ZERO && timer.0.tick(time.delta()).just_finished() {
                commands
                    .spawn_bundle(SpriteBundle {
                        texture: sprites.arrow.clone(),
                        transform: Transform {
                            translation: e_transform.translation,
                            scale: Vec3::splat(2.0),
                            rotation: Quat::from_rotation_z(
                                p_transform
                                    .translation
                                    .angle_between_points(e_transform.translation),
                            ),
                        },
                        ..Default::default()
                    })
                    .insert(RigidBody::KinematicVelocityBased)
                    .insert(CollisionShape::Cuboid {
                        half_extends: Vec3::new(21.0, 7.0, 0.0),
                        border_radius: None,
                    })
                    .insert(CollisionLayers::new(
                        GamePhysicsLayer::EnemyAttack,
                        GamePhysicsLayer::Player,
                    ))
                    .insert(Velocity::from_linear(
                        (p_transform.translation.truncate() - e_transform.translation.truncate())
                            .extend(0.0)
                            .normalize()
                            * 400.0,
                    ))
                    .insert(EnemyProjectile)
                    .insert(DespawnTimer(Timer::from_seconds(3.0, false)));
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
                EnemyAI::Afraid { speed: _ } => {
                    if transform.translation.x > player.translation.x {
                        sprite.flip_x = false;
                    } else {
                        sprite.flip_x = true;
                    }
                }
                _ => (),
            };
            if health.current < enemy.fear_threshold {
                sprite.color = Color::rgb(
                    1.0,
                    0.25 + (health.current.max(0.0) / health.maximum) / 2.0,
                    0.25 + (health.current.max(0.0) / health.maximum) / 2.0,
                );
            } else if let EnemyAI::Afraid { speed: _ } = enemy.ai {
                sprite.color = Color::rgb(1.0, 0.5, 1.0);
            }
        }
    }
}

pub fn check_enemy_player_collision(
    mut collision_events: EventReader<CollisionEvent>,
    mut q_enemies: Query<&mut DamagesPlayer>,
    mut damage_writer: EventWriter<DamagePlayerEvent>,
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
                    damage_writer.send(DamagePlayerEvent(enemy.damage));
                    enemy.is_damaging = true;
                }
                CollisionEvent::Stopped(_d1, _d2) => {
                    enemy.is_damaging = false;
                }
            }
        }
    }
}

pub fn enemy_projectile_damage_player(
    mut commands: Commands,
    mut collision_events: EventReader<CollisionEvent>,
    mut damage_writer: EventWriter<DamagePlayerEvent>,
) {
    fn is_player(layers: CollisionLayers) -> bool {
        layers.contains_group(GamePhysicsLayer::Player)
            && !layers.contains_group(GamePhysicsLayer::EnemyAttack)
    }
    fn is_projectile(layers: CollisionLayers) -> bool {
        layers.contains_group(GamePhysicsLayer::EnemyAttack)
            && !layers.contains_group(GamePhysicsLayer::Player)
    }

    for (evt, ent) in collision_events.iter().filter_map(|event| {
        let (entity_1, entity_2) = event.rigid_body_entities();
        let (layers_1, layers_2) = event.collision_layers();
        if is_projectile(layers_1) && is_player(layers_2) {
            Some((event, entity_1))
        } else if is_projectile(layers_2) && is_player(layers_1) {
            Some((event, entity_2))
        } else {
            None
        }
    }) {
        if let CollisionEvent::Started(_, _) = evt {
            damage_writer.send(DamagePlayerEvent(1.0));
            commands.entity(ent).despawn();
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
            morale.enemies_killed += 1;
            morale.change -= 0.05;
            true
        } else if let EnemyAI::Afraid { speed: _ } = enemy.ai {
            if transform.translation.y <= -SCREEN_HEIGHT * 0.6 {
                if health.current > enemy.fear_threshold {
                    morale.change += 0.05;
                } else {
                    morale.change += 0.15;
                }
                commands.entity(ent).despawn();
                true
            } else {
                false
            }
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
