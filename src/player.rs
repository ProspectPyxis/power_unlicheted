use crate::common::{
    get_cursor_position, AngledMovement, DespawnTimer, GameSprites, MainCamera, Player, Projectile,
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
        if keyboard_input.pressed(KeyCode::Left) {
            transform.translation.x -= 4.0;
            sprite.flip_x = true;
        }
        if keyboard_input.pressed(KeyCode::Right) {
            transform.translation.x += 4.0;
            sprite.flip_x = false;
        }
        if keyboard_input.pressed(KeyCode::Up) {
            transform.translation.y += 4.0;
        }
        if keyboard_input.pressed(KeyCode::Down) {
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
                    angle: (cursor_pos.y - player.translation.y)
                        .atan2(cursor_pos.x - player.translation.x),
                })
                .insert(DespawnTimer(Timer::from_seconds(1.0, false)));
        }
    }
}
