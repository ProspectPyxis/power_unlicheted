use bevy::{
    input::{keyboard::KeyCode, Input},
    prelude::*,
};

#[derive(Component)]
pub struct Player;

pub fn spawn_player(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn_bundle(SpriteBundle {
            texture: asset_server.load("sprites/lich.png"),
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
