use crate::common::{
    get_cursor_position, DespawnTimer, GamePhysicsLayer, GameSprites, Health, HealthBarUI,
    MainCamera, Player, Projectile, RegeneratesHealth, Vec3Utils,
};
use bevy::{input::keyboard::KeyCode, prelude::*};
use heron::prelude::*;
use std::f32::consts::PI;

pub fn spawn_player(
    mut commands: Commands,
    sprites: Res<GameSprites>,
    asset_server: Res<AssetServer>,
) {
    let player = commands
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
        .insert(CollisionShape::Sphere { radius: 32.0 })
        .insert(CollisionLayers::new(
            GamePhysicsLayer::Player,
            GamePhysicsLayer::Enemy,
        ))
        .insert(Health {
            current: 500.0,
            maximum: 500.0,
        })
        .insert(RegeneratesHealth {
            regen: 1.0,
            tick: Timer::from_seconds(2.0, true),
            is_regenerating: true,
        })
        .id();

    let ui_font = asset_server.load("fonts/OpenSans-Regular.ttf");
    let text_style = TextStyle {
        font: ui_font,
        font_size: 20.0,
        color: Color::GREEN,
    };
    let text_alignment = TextAlignment {
        vertical: VerticalAlign::Center,
        horizontal: HorizontalAlign::Center,
    };
    let health_bar = commands
        .spawn_bundle(Text2dBundle {
            text: Text::with_section("1000", text_style, text_alignment),
            transform: Transform::from_translation(Vec3::new(0.0, -20.0, -1.0)),
            ..Default::default()
        })
        .insert(HealthBarUI)
        .id();

    commands.entity(player).push_children(&[health_bar]);
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
            for i in -1..=1 {
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
                    .insert(Velocity::from_linear(
                        (cursor_pos - player.translation.truncate())
                            .extend(0.0)
                            .normalize()
                            .rotate_2d(PI * i as f32 / 16.0)
                            * 240.0,
                    ))
                    .insert(CollisionShape::Sphere { radius: 16.0 })
                    .insert(CollisionLayers::new(
                        GamePhysicsLayer::Projectile,
                        GamePhysicsLayer::Enemy,
                    ))
                    .insert(DespawnTimer(Timer::from_seconds(1.5, false)));
            }
        }
    }
}

pub fn update_health_display(
    mut q_health_bar: Query<(&Parent, &mut Text), With<HealthBarUI>>,
    q_player: Query<&Health, With<Player>>,
) {
    for (parent, mut text) in q_health_bar.iter_mut() {
        let player = q_player.get(parent.0).unwrap();
        text.sections[0].value = format!("{}", player.current.floor() as i32);
    }
}
