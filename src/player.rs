use crate::common::{
    angle_between_points, get_cursor_position, vec3_from_magnitude_angle, DespawnTimer,
    GamePhysicsLayer, GameSprites, MainCamera, Player, Projectile,
};
use bevy::{input::keyboard::KeyCode, prelude::*};
use heron::prelude::*;

pub fn spawn_player(mut commands: Commands, sprites: Res<GameSprites>) {
    commands
        .spawn_bundle(SpriteBundle {
            texture: sprites.lich.clone(),
            transform: Transform {
                scale: Vec3::new(4.0, 4.0, 0.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Player)
        .insert(RigidBody::KinematicPositionBased)
        .insert(CollisionShape::Cuboid {
            half_extends: Vec3::new(32.0, 40.0, 0.0),
            border_radius: None,
        })
        .insert(CollisionLayers::new(
            GamePhysicsLayer::Player,
            GamePhysicsLayer::Enemy,
        ));
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
            let player = q_player.single();
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
                .insert(Velocity::from_linear(vec3_from_magnitude_angle(
                    240.0,
                    angle_between_points(player.translation.truncate(), cursor_pos),
                )))
                .insert(CollisionShape::Sphere { radius: 16.0 })
                .insert(CollisionLayers::new(
                    GamePhysicsLayer::Projectile,
                    GamePhysicsLayer::Enemy,
                ))
                .insert(DespawnTimer(Timer::from_seconds(1.5, false)));
        }
    }
}
