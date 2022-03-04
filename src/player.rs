use crate::common::{
    get_cursor_position, ChangeSpellEvent, CurrentDay, DamagePlayerEvent, DamagesEnemy,
    DayEndReason, DespawnTimer, EndDayEvent, EnemyMorale, GameAudio, GameFonts, GamePhysicsLayer,
    GameSprites, GameState, Health, InGameUI, InvisTimer, LightningStrikeBolt, MainCamera, Player,
    PlayerSpell, PlayerSpellData, Projectile, SpellCooldowns, Ui, Vec3Utils, SCREEN_HEIGHT,
};
use bevy::{input::keyboard::KeyCode, prelude::*};
use bevy_kira_audio::Audio;
use heron::prelude::*;
use std::f32::consts::PI;

pub fn spawn_player(mut commands: Commands, sprites: Res<GameSprites>) {
    commands
        .spawn_bundle(SpriteBundle {
            texture: sprites.lich.clone(),
            transform: Transform {
                translation: Vec3::new(0.0, 0.0, 0.5),
                scale: Vec3::new(4.0, 4.0, 0.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Player)
        .insert(RigidBody::KinematicPositionBased)
        .insert(CollisionShape::Sphere { radius: 24.0 })
        .insert(CollisionLayers::new(
            GamePhysicsLayer::Player,
            GamePhysicsLayer::Enemy,
        ))
        .insert(Health::full(200.0))
        .insert(PlayerSpellData {
            selected: PlayerSpell::Fireball,
            cooldowns: SpellCooldowns::default(),
        });
}

pub fn spawn_player_ui(mut commands: Commands, sprites: Res<GameSprites>) {
    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: Color::GREEN,
                custom_size: Some(Vec2::new(100.0, 12.0)),
                ..Default::default()
            },
            transform: Transform {
                translation: Vec3::new(0.0, -60.0, 15.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Ui::HealthBarMain)
        .insert(InGameUI);

    commands
        .spawn_bundle(SpriteBundle {
            texture: sprites.spell_icon_fireball.clone(),
            transform: Transform {
                translation: Vec3::new(0.0, 60.0, 15.0),
                scale: Vec3::new(2.0, 2.0, 0.0),
                ..Default::default()
            },
            visibility: Visibility { is_visible: false },
            ..Default::default()
        })
        .insert(Ui::CurrentSpell)
        .insert(InGameUI)
        .insert(InvisTimer(Timer::from_seconds(1.0, false)));
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

#[allow(clippy::too_many_arguments)]
pub fn player_shoot(
    mut commands: Commands,
    sprites: Res<GameSprites>,
    audio: Res<GameAudio>,
    mut q_player: Query<(&Transform, &mut PlayerSpellData), With<Player>>,
    wnds: Res<Windows>,
    q_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    mouse_input: Res<Input<MouseButton>>,
    audio_player: Res<Audio>,
) {
    if mouse_input.pressed(MouseButton::Left) {
        if let Some(cursor_pos) = get_cursor_position(wnds, q_camera) {
            if let Some((player_t, mut spell_data)) = q_player.iter_mut().next() {
                match spell_data.selected {
                    PlayerSpell::Fireball => {
                        if spell_data.cooldowns.fireball.finished() {
                            for i in -1..=1 {
                                commands
                                    .spawn_bundle(SpriteBundle {
                                        texture: sprites.fireball.clone(),
                                        transform: Transform {
                                            translation: player_t.translation,
                                            scale: Vec3::new(2.0, 2.0, 0.0),
                                            ..Default::default()
                                        },
                                        ..Default::default()
                                    })
                                    .insert(Projectile)
                                    .insert(RigidBody::KinematicVelocityBased)
                                    .insert(Velocity::from_linear(
                                        (cursor_pos - player_t.translation.truncate())
                                            .extend(0.0)
                                            .normalize()
                                            .rotate_2d(PI * i as f32 / 16.0)
                                            * 360.0,
                                    ))
                                    .insert(CollisionShape::Sphere { radius: 8.0 })
                                    .insert(CollisionLayers::new(
                                        GamePhysicsLayer::PlayerAttack,
                                        GamePhysicsLayer::Enemy,
                                    ))
                                    .insert(DespawnTimer(Timer::from_seconds(1.5, false)))
                                    .insert(DamagesEnemy { damage: 2.0 })
                                    .with_children(|parent| {
                                        parent
                                            .spawn()
                                            .insert(GlobalTransform::default())
                                            .insert(Transform::default())
                                            .insert(RigidBody::Sensor)
                                            .insert(CollisionShape::Sphere { radius: 16.0 })
                                            .insert(CollisionLayers::new(
                                                GamePhysicsLayer::PlayerAttack,
                                                GamePhysicsLayer::Enemy,
                                            ))
                                            .insert(DamagesEnemy { damage: 1.0 });
                                    });
                            }
                            audio_player.play(audio.fireball.clone());
                            spell_data.cooldowns.fireball.reset();
                        }
                    }
                    PlayerSpell::LightningStrike => {
                        if spell_data.cooldowns.lightning_strike.finished() {
                            commands
                                .spawn_bundle(SpriteBundle {
                                    texture: sprites.lightning_bolt.clone(),
                                    transform: Transform {
                                        translation: Vec3::new(
                                            cursor_pos.x,
                                            cursor_pos.y + SCREEN_HEIGHT + 24.0,
                                            0.1,
                                        ),
                                        scale: Vec3::new(2.0, 2.0, 0.0),
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                })
                                .insert(LightningStrikeBolt {
                                    end_y: cursor_pos.y,
                                });
                            spell_data.cooldowns.lightning_strike.reset();
                        }
                    }
                }
            }
        }
    }
}

pub fn tick_attack_cooldowns(
    mut q_player: Query<&mut PlayerSpellData, With<Player>>,
    time: Res<Time>,
) {
    for mut player in q_player.iter_mut() {
        player.cooldowns.tick_all(time.delta());
    }
}

pub fn switch_active_spell(
    mut q_player: Query<&mut PlayerSpellData, With<Player>>,
    keyboard_input: Res<Input<KeyCode>>,
    mut change_spell: EventWriter<ChangeSpellEvent>,
) {
    if let Some(mut spell_data) = q_player.iter_mut().next() {
        if keyboard_input.just_pressed(KeyCode::E) {
            spell_data.selected = spell_data.selected.next();
            change_spell.send(ChangeSpellEvent(spell_data.selected));
        } else if keyboard_input.just_pressed(KeyCode::Q) {
            spell_data.selected = spell_data.selected.previous();
            change_spell.send(ChangeSpellEvent(spell_data.selected));
        }
    }
}

pub fn register_player_damage(
    mut q_player: Query<&mut Health, With<Player>>,
    mut damages: EventReader<DamagePlayerEvent>,
    mut morale: ResMut<EnemyMorale>,
    mut state: ResMut<State<GameState>>,
    mut day_end_writer: EventWriter<EndDayEvent>,
    audio: Res<GameAudio>,
    audio_player: Res<Audio>,
) {
    if let Some(mut player) = q_player.iter_mut().next() {
        let mut damaged = false;
        for damage in damages.iter() {
            damaged = true;
            player.current -= damage.0;
            morale.change += damage.0 / 10.0;
        }
        if damaged {
            audio_player.play(audio.player_hurt.clone());
        }
        if player.current <= 0.0 {
            day_end_writer.send(EndDayEvent {
                reason: DayEndReason::PlayerDeath,
            });
            state.set(GameState::MoraleStatus).unwrap();
        }
    }
}

pub fn update_health_bar(
    mut q_ui: Query<(&mut Sprite, &mut Transform, &Ui), Without<Player>>,
    q_player: Query<(&Health, &Transform), With<Player>>,
) {
    for (mut sprite, mut h_transform) in q_ui.iter_mut().filter_map(|(s, t, i)| match i {
        Ui::HealthBarMain => Some((s, t)),
        _ => None,
    }) {
        if let Some((health, p_transform)) = q_player.iter().next() {
            sprite.custom_size = Some(Vec2::new(
                (health.current / health.maximum).max(0.0) * 100.0,
                12.0,
            ));
            if health.current <= health.maximum * 0.25 {
                sprite.color = Color::RED;
            } else {
                sprite.color = Color::GREEN;
            }
            h_transform.translation.x = p_transform.translation.x;
            h_transform.translation.y = p_transform.translation.y - 60.0;
        }
    }
}

#[allow(clippy::type_complexity)]
pub fn update_spell_display(
    mut q_ui: Query<
        (
            &mut Handle<Image>,
            &mut Visibility,
            &mut InvisTimer,
            &mut Transform,
            &Ui,
        ),
        Without<Player>,
    >,
    q_player: Query<&Transform, With<Player>>,
    mut change_spell: EventReader<ChangeSpellEvent>,
    sprites: Res<GameSprites>,
) {
    let spell_changed = change_spell.iter().next();
    let player = q_player.iter().next();
    for (texture, mut visibility, mut timer, mut transform) in
        q_ui.iter_mut().filter_map(|(h, v, i, t, u)| match u {
            Ui::CurrentSpell => Some((h, v, i, t)),
            _ => None,
        })
    {
        if let Some(spell) = spell_changed {
            *texture.into_inner() = match spell.0 {
                PlayerSpell::Fireball => sprites.spell_icon_fireball.clone(),
                PlayerSpell::LightningStrike => sprites.spell_icon_lightning.clone(),
            };
            timer.0.reset();
            visibility.is_visible = true;
        }
        if let Some(player) = player {
            transform.translation.x = player.translation.x;
            transform.translation.y = player.translation.y + 60.0;
        }
    }
}

pub fn display_player_controls(
    mut commands: Commands,
    current_day: Res<CurrentDay>,
    fonts: Res<GameFonts>,
) {
    if current_day.0 == 1 {
        commands
            .spawn_bundle(NodeBundle {
                style: Style {
                    size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    flex_direction: FlexDirection::ColumnReverse,
                    ..Default::default()
                },
                color: Color::NONE.into(),
                ..Default::default()
            })
            .insert(DespawnTimer(Timer::from_seconds(7.0, false)))
            .with_children(|parent| {
                parent.spawn_bundle(TextBundle {
                    text: Text {
                        sections: vec![TextSection {
                            value: "WASD: Move, LMB (Hold): Attack, QE: Change Spells".to_string(),
                            style: TextStyle {
                                font: fonts.main.clone(),
                                font_size: 32.0,
                                color: Color::WHITE,
                            },
                        }],
                        alignment: TextAlignment {
                            horizontal: HorizontalAlign::Center,
                            vertical: VerticalAlign::Center,
                        },
                    },
                    style: Style {
                        margin: Rect {
                            top: Val::Px(SCREEN_HEIGHT * 0.3),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    ..Default::default()
                });
            });
    }
}
