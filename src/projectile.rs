use crate::common::{DamagesEnemy, Enemy, GamePhysicsLayer, Health};
use bevy::prelude::*;
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
