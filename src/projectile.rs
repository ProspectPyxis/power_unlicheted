use crate::common::{
    Animated, DamagesEnemy, DespawnTimer, Enemy, GameAudio, GamePhysicsLayer, GameSprites, Health,
    LightningStrikeBolt, SCREEN_HEIGHT,
};
use bevy::prelude::*;
use bevy_kira_audio::Audio;
use heron::prelude::*;

pub fn check_projectile_collision(
    mut collision_events: EventReader<CollisionEvent>,
    mut q_enemies: Query<&mut Health, With<Enemy>>,
    q_damages: Query<&DamagesEnemy>,
) {
    fn is_projectile(layers: CollisionLayers) -> bool {
        layers.contains_group(GamePhysicsLayer::PlayerAttack)
            && !layers.contains_group(GamePhysicsLayer::Enemy)
    }
    fn is_enemy(layers: CollisionLayers) -> bool {
        layers.contains_group(GamePhysicsLayer::Enemy)
            && !layers.contains_group(GamePhysicsLayer::PlayerAttack)
    }

    for (e_enemy, e_damager) in collision_events
        .iter()
        .filter(|e| e.is_started())
        .filter_map(|event| {
            let (entity_1, entity_2) = event.rigid_body_entities();
            let (layers_1, layers_2) = event.collision_layers();
            if is_enemy(layers_1) && is_projectile(layers_2) {
                Some((entity_1, entity_2))
            } else if is_enemy(layers_2) && is_projectile(layers_1) {
                Some((entity_2, entity_1))
            } else {
                None
            }
        })
    {
        if let Ok(mut enemy) = q_enemies.get_mut(e_enemy) {
            if let Ok(damage) = q_damages.get(e_damager) {
                enemy.current -= damage.damage;
            }
        }
    }
}

pub fn update_lightning_bolt(
    mut commands: Commands,
    mut q_lightning_bolt: Query<(Entity, &LightningStrikeBolt, &mut Transform)>,
    time: Res<Time>,
    sprites: Res<GameSprites>,
    audio: Res<GameAudio>,
    audio_player: Res<Audio>,
) {
    for (ent, bolt, mut transform) in q_lightning_bolt.iter_mut() {
        transform.translation.y -= SCREEN_HEIGHT * 5.0 * time.delta().as_secs_f32();
        if transform.translation.y <= bolt.end_y {
            commands
                .spawn_bundle(SpriteSheetBundle {
                    texture_atlas: sprites.lightning_explosion.clone(),
                    sprite: TextureAtlasSprite {
                        color: Color::rgba(1.0, 1.0, 1.0, 0.5),
                        ..Default::default()
                    },
                    transform: Transform {
                        translation: Vec3::new(transform.translation.x, bolt.end_y, 0.6),
                        scale: Vec3::new(3.0, 3.0, 0.0),
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .insert(Animated {
                    frames: 4,
                    timer: Timer::from_seconds(1.0 / 60.0, true),
                })
                .insert(RigidBody::Sensor)
                .insert(CollisionShape::Sphere { radius: 96.0 })
                .insert(CollisionLayers::new(
                    GamePhysicsLayer::PlayerAttack,
                    GamePhysicsLayer::Enemy,
                ))
                .insert(DamagesEnemy { damage: 3.0 })
                .insert(DespawnTimer(Timer::from_seconds(0.25, false)))
                .with_children(|parent| {
                    parent
                        .spawn()
                        .insert(GlobalTransform::default())
                        .insert(Transform::default())
                        .insert(RigidBody::Sensor)
                        .insert(CollisionShape::Sphere { radius: 112.0 })
                        .insert(CollisionLayers::new(
                            GamePhysicsLayer::PlayerAttack,
                            GamePhysicsLayer::Enemy,
                        ))
                        .insert(DamagesEnemy { damage: 2.0 });
                });

            audio_player.play(audio.lightning_explosion.clone());
            commands.entity(ent).despawn();
        }
    }
}
