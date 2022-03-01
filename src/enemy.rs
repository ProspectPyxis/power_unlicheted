use crate::common::{
    angle_between_points, vec3_from_magnitude_angle, Enemy, EnemyAI, GamePhysicsLayer, GameSprites,
    Player,
};
use bevy::prelude::*;
use heron::prelude::*;
use std::f32::consts::{FRAC_PI_2, PI};

pub fn spawn_enemy(
    mut commands: Commands,
    sprites: Res<GameSprites>,
    q_player: Query<&Transform, With<Player>>,
) {
    let player = q_player.single();
    let pos = Vec3::new(alea::f32_in_range(-648.0, 648.0), -490.0, 0.0);
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
        .insert(CollisionShape::Cuboid {
            half_extends: Vec3::new(11.25, 15.0, 0.0),
            border_radius: None,
        })
        .insert(Velocity::from_linear(vec3_from_magnitude_angle(
            120.0,
            angle_between_points(pos.truncate(), player.translation.truncate()),
        )))
        .insert(RotationConstraints::lock())
        .insert(
            CollisionLayers::none()
                .with_group(GamePhysicsLayer::Enemy)
                .with_masks(&[GamePhysicsLayer::Enemy, GamePhysicsLayer::Projectile]),
        );
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
                let angle =
                    angle_between_points(current_pos.truncate(), player.translation.truncate());
                velocity.linear = vec3_from_magnitude_angle(speed, angle);
                if ((angle + FRAC_PI_2) / PI).rem_euclid(2.0).floor() == 1.0 {
                    sprite.flip_x = true;
                } else {
                    sprite.flip_x = false;
                }
            }
        }
    }
}
