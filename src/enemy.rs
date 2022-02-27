use crate::common::{angle_between_points, move_in_direction, Enemy, EnemyAI, GameSprites, Player};
use bevy::prelude::*;
use std::f32::consts::FRAC_PI_2;

pub fn spawn_enemy(mut commands: Commands, sprites: Res<GameSprites>) {
    commands
        .spawn_bundle(SpriteBundle {
            texture: sprites.soldier.clone(),
            transform: Transform {
                translation: Vec3::new(alea::f32_in_range(-648.0, 648.0), -490.0, 0.0),
                scale: Vec3::new(1.5, 1.5, 0.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Enemy {
            ai: EnemyAI::ChasesPlayer { speed: 2.0 },
        });
}

pub fn update_enemy(
    mut q_enemies: Query<(&Enemy, &mut Transform, &mut Sprite), Without<Player>>,
    q_player: Query<&Transform, With<Player>>,
) {
    let player = q_player.single();
    for (enemy, mut transform, mut sprite) in q_enemies.iter_mut() {
        let current_pos = transform.translation.clone();
        match enemy.ai {
            EnemyAI::ChasesPlayer { speed } => {
                let angle =
                    angle_between_points(current_pos.truncate(), player.translation.truncate());
                move_in_direction(transform, speed, angle);
                if (angle / FRAC_PI_2).ceil() % 4.0 == 2.0
                    || (angle / FRAC_PI_2).ceil() % 4.0 == 3.0
                {
                    sprite.flip_x = true;
                } else {
                    sprite.flip_x = false;
                }
            }
        }
    }
}
