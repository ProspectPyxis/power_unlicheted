use crate::common::GamePhysicsLayer;
use bevy::prelude::*;
use heron::prelude::*;

pub fn check_projectile_collision(
    mut commands: Commands,
    mut collision_events: EventReader<CollisionEvent>,
) {
    fn is_projectile(layers: CollisionLayers) -> bool {
        layers.contains_group(GamePhysicsLayer::PlayerAttack)
            && !layers.contains_group(GamePhysicsLayer::Enemy)
    }
    fn is_enemy(layers: CollisionLayers) -> bool {
        layers.contains_group(GamePhysicsLayer::Enemy)
            && !layers.contains_group(GamePhysicsLayer::PlayerAttack)
    }

    for ent in collision_events
        .iter()
        .filter(|e| e.is_started())
        .filter_map(|event| {
            let (entity_1, entity_2) = event.rigid_body_entities();
            let (layers_1, layers_2) = event.collision_layers();
            if is_enemy(layers_1) && is_projectile(layers_2) {
                Some(entity_1)
            } else if is_enemy(layers_2) && is_projectile(layers_1) {
                Some(entity_2)
            } else {
                None
            }
        })
    {
        commands.entity(ent).despawn();
    }
}
