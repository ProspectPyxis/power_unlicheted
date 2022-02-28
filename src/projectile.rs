use crate::common::{check_squares_collision, Enemy, Projectile, RectCollider};
use bevy::prelude::*;

pub fn check_projectile_collision(
    mut commands: Commands,
    q_projectiles: Query<(&Transform, &RectCollider), With<Projectile>>,
    q_enemies: Query<(Entity, &Transform, &RectCollider), With<Enemy>>,
) {
    for (projectile, proj_collider) in q_projectiles.iter() {
        for (ent, transform, enemy_collider) in q_enemies.iter() {
            if check_squares_collision(enemy_collider, proj_collider, transform, projectile) {
                commands.entity(ent).despawn();
            }
        }
    }
}
