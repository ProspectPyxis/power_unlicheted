use crate::common::{
    angle_between_points, get_cursor_position, AngledMovement, DespawnTimer, GameSprites,
    MainCamera, Player, Projectile, RectCollider,
};
use bevy::{input::keyboard::KeyCode, prelude::*};

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
        .insert(Player);
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
                        translation: player.translation.clone(),
                        scale: Vec3::new(2.0, 2.0, 0.0),
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .insert(Projectile)
                .insert(AngledMovement {
                    speed: 4.0,
                    angle: angle_between_points(player.translation.truncate(), cursor_pos),
                })
                .insert(DespawnTimer(Timer::from_seconds(1.5, false)))
                .insert(RectCollider::square(32.0));
        }
    }
}
